"""
Chaser-Oxide Google 搜索爬虫

通过 chaser-oxide-server 实现 Google 搜索自动化，支持代理配置。

依赖安装:
    pip install grpcio grpcio-tools

使用方法:
    # 从项目根目录运行
    cd examples/google_search_with_proxy
    python google_search.py --query "搜索关键词" --max-results 100

或者将 chaser 包安装到 Python 环境:
    pip install -e ../../../docs/examples/python
"""

import argparse
import csv
import json
import logging
import sys
import time
import grpc
from typing import List, Dict, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

# 添加示例目录到路径以导入 chaser 包
sys.path.insert(0, '../../docs/examples/python')

# 导入生成的 gRPC 代码
from chaser.oxide.v1 import (
    common_pb2,
    browser_pb2,
    browser_pb2_grpc,
    page_pb2,
    page_pb2_grpc,
    element_pb2,
    element_pb2_grpc,
    profile_pb2,
    profile_pb2_grpc,
)

from config import ProxyConfig


# Error code mapping
ERROR_CODE_NAMES = {
    0: "UNKNOWN",
    1: "INVALID_ARGUMENT",
    2: "NOT_FOUND",
    3: "ALREADY_EXISTS",
    4: "PERMISSION_DENIED",
    5: "RESOURCE_EXHAUSTED",
    6: "FAILED_PRECONDITION",
    7: "ABORTED",
    8: "OUT_OF_RANGE",
    9: "INTERNAL",
    10: "BROWSER_CLOSED",
    11: "PAGE_CLOSED",
    12: "ELEMENT_NOT_FOUND",
    13: "NAVIGATION_FAILED",
    14: "TIMEOUT",
    15: "EVALUATION_FAILED",
}


def check_error(response, operation: str) -> bool:
    """Check for errors in gRPC response and log them

    Args:
        response: gRPC response object with error field
        operation: Operation name for logging

    Returns:
        True if error exists, False otherwise
    """
    if response.HasField('error'):
        error_code = response.error.code
        error_message = response.error.message
        error_name = ERROR_CODE_NAMES.get(error_code, f"CODE_{error_code}")

        logger.error(f"{operation} failed: [{error_name}] {error_message}")
        logger.error(f"  Error code: {error_code}")
        logger.debug(f"  Full error response: {response.error}")
        return True
    return False


class GoogleSearchScraper:
    """Google 搜索爬虫 - 使用 chaser-oxide-server 进行搜索"""

    def __init__(self, host: str = "localhost:50051", proxy_config: Optional[ProxyConfig] = None):
        """初始化爬虫

        Args:
            host: chaser-oxide 服务器地址
            proxy_config: 代理配置，如果为 None 则从环境变量加载
        """
        self.channel = grpc.insecure_channel(host)
        self.browser = browser_pb2_grpc.BrowserServiceStub(self.channel)
        self.page = page_pb2_grpc.PageServiceStub(self.channel)
        self.element = element_pb2_grpc.ElementServiceStub(self.channel)
        self.profile = profile_pb2_grpc.ProfileServiceStub(self.channel)
        self.proxy_config = proxy_config or ProxyConfig.from_env()

        self.browser_id = None
        self.page_id = None
        self.current_user_agent = None  # 用于存储 profile 的 user_agent
        self.current_query = None  # 用于存储当前搜索关键词，用于分页

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """退出上下文时自动关闭资源

        无论是否发生异常，都会调用 close() 方法清理资源。
        返回 False 让异常继续传播（如果有的话）。
        """
        logger.info("上下文管理器退出，开始清理资源...")
        logger.debug(f"异常信息: exc_type={exc_type}, exc_val={exc_val}")

        try:
            self.close()
        except Exception as e:
            logger.error(f"资源清理过程中发生错误: {type(e).__name__} - {str(e)}")
            logger.debug("堆栈跟踪:", exc_info=True)

        logger.info("上下文管理器退出完成")
        return False  # 返回 False 让异常继续传播

    def _take_screenshot(self, filename: str) -> bool:
        """截取当前页面截图

        Args:
            filename: 保存文件名

        Returns:
            bool: 成功返回 True
        """
        import os
        os.makedirs('logs', exist_ok=True)
        filepath = f'logs/{filename}'

        try:
            # Use ScreenshotOptions instead of path parameter
            request = page_pb2.ScreenshotRequest(
                page_id=self.page_id,
                options=common_pb2.ScreenshotOptions(
                    format=common_pb2.ScreenshotOptions.FORMAT_PNG
                )
            )
            response = self.page.Screenshot(request)

            if check_error(response, "截图"):
                return False

            # Write binary data to file
            if response.HasField('result'):
                with open(filepath, 'wb') as f:
                    f.write(response.result.data)
                logger.info(f"截图已保存: {filepath}")
                return True
            return False
        except Exception as e:
            logger.warning(f"截图失败: {e}")
            return False

    def _setup_stealth_profile(self) -> Optional[str]:
        """配置反检测指纹 - 使用高熵值随机化

        使用 RandomizeProfile 创建持久化的反检测配置。
        高熵值 (0.95) 最大化随机性以绕过 Google CAPTCHA。

        Returns:
            str: profile_id，失败返回 None
        """
        try:
            # 使用 RandomizeProfile 创建持久化配置
            request = profile_pb2.RandomizeProfileRequest(
                type=profile_pb2.PROFILE_TYPE_WINDOWS,
                options=profile_pb2.RandomizationOptions(
                    randomize_screen=True,
                    randomize_timezone=True,
                    randomize_language=True,
                    randomize_webgl=True,
                    entropy=0.95  # 高熵值，最大化随机性
                )
            )

            response = self.profile.RandomizeProfile(request)

            if response.HasField('error'):
                logger.warning(f"创建反检测配置失败: {response.error.message}")
                return None

            profile_id = response.profile.profile_id
            logger.info(f"反检测配置已创建: {profile_id}")
            logger.debug(f"  User-Agent: {response.profile.fingerprint.headers.user_agent}")
            logger.debug(f"  平台: {response.profile.fingerprint.navigator.platform}")
            logger.debug(f"  CPU 核心: {response.profile.fingerprint.hardware.cpu_cores}")
            logger.debug(f"  内存: {response.profile.fingerprint.hardware.device_memory}GB")
            logger.debug(f"  GPU: {response.profile.fingerprint.hardware.gpu_vendor} - {response.profile.fingerprint.hardware.gpu_renderer}")

            # 更新浏览器 user_agent 以匹配 profile
            self.current_user_agent = response.profile.fingerprint.headers.user_agent

            return profile_id

        except Exception as e:
            logger.warning(f"创建反检测配置异常: {e}")
            return None

    def launch_browser(self) -> bool:
        """启动浏览器并配置代理

        Returns:
            bool: 成功返回 True，失败返回 False
        """
        logger.info("启动浏览器...")

        try:
            proxy_server = self.proxy_config.to_chaser_proxy()
            proxy_bypass = self.proxy_config.to_chaser_bypass_list()

            launch_request = browser_pb2.LaunchRequest(
                options=common_pb2.BrowserOptions(
                    headless=True,
                    window_width=1920,
                    window_height=1080,
                    proxy_server=proxy_server,
                    proxy_bypass_list=proxy_bypass,
                    user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
                    ignore_certificate_errors=True,  # 代理可能使用自签名证书
                    args=[  # 反检测参数 + Docker 环境优化
                        "--disable-blink-features=AutomationControlled",
                        "--disable-dev-shm-usage",
                        "--no-sandbox",
                        "--disable-infobars",  # 禁用信息栏
                        "--disable-extensions",  # 禁用扩展
                        "--disable-gpu",  # Docker 环境推荐
                        "--disable-software-rasterizer",
                        "--disable-background-timer-throttling",
                        "--disable-backgrounding-occluded-windows",
                        "--disable-renderer-backgrounding",
                        "--disable-features=IsolateOrigins,site-per-process",
                        "--window-size=1920,1080"  # 固定窗口大小
                    ]
                )
            )

            launch_response = self.browser.Launch(launch_request)

            if check_error(launch_response, "启动浏览器"):
                return False

            self.browser_id = launch_response.browser_info.browser_id
            logger.info(f"浏览器已启动: {self.browser_id}")

            if proxy_server:
                logger.info(f"  使用代理: {proxy_server}")
            if proxy_bypass:
                logger.info(f"  绕过列表: {proxy_bypass}")

            # 创建页面
            create_page_request = page_pb2.CreatePageRequest(
                browser_id=self.browser_id,
                url="about:blank"
            )

            create_page_response = self.page.CreatePage(create_page_request)

            if check_error(create_page_response, "创建页面"):
                return False

            self.page_id = create_page_response.page_info.page_id
            logger.info(f"页面已创建: {self.page_id}")

            # 应用反检测配置
            profile_id = self._setup_stealth_profile()
            if profile_id:
                try:
                    apply_request = profile_pb2.ApplyProfileRequest(
                        page_id=self.page_id,
                        profile_id=profile_id,
                        override_existing=True
                    )
                    apply_response = self.profile.ApplyProfile(apply_request)

                    if not apply_response.HasField('error'):
                        logger.info(f"反检测配置已应用: {', '.join(apply_response.result.applied_features)}")
                    else:
                        logger.warning(f"应用反检测配置失败: {apply_response.error.message}")
                except Exception as e:
                    logger.warning(f"应用反检测配置异常: {e}")

            return True

        except grpc.RpcError as e:
            logger.error(f"gRPC 错误: {e.code().name} - {e.details()}")
            logger.debug("堆栈跟踪:", exc_info=True)
            return False

    def search(self, query: str, max_results: int = 100) -> List[Dict]:
        """执行 Google 搜索

        Args:
            query: 搜索关键词
            max_results: 最大结果数

        Returns:
            搜索结果列表，每个结果包含 rank, title, url, display_url
        """
        logger.info(f"开始搜索: {query}")
        self.current_query = query  # 保存查询用于分页

        try:
            # 直接构造搜索 URL（绕过表单提交，更可靠）
            import urllib.parse
            encoded_query = urllib.parse.quote(query)
            search_url = f"https://www.google.com/search?q={encoded_query}&num=10"

            navigate_request = page_pb2.NavigateRequest(
                page_id=self.page_id,
                url=search_url,
                options=common_pb2.NavigationOptions(
                    timeout=30000,
                    wait_until=common_pb2.NavigationOptions.LOAD_STATE_NETWORK_IDLE
                )
            )

            navigate_response = self.page.Navigate(navigate_request)

            if check_error(navigate_response, "导航到搜索结果页"):
                return []

            logger.info(f"已导航到搜索结果页: {search_url}")

            # 等待页面加载
            time.sleep(3)

            # 截图用于诊断
            self._take_screenshot('after_search.png')

            # 诊断：检查页面状态 - 增强版
            diagnostic_script = """
(() => {
    // 检查是否是 CAPTCHA 页面
    const hasCaptcha = !!(
        document.querySelector('.recaptcha-checkbox') ||
        document.querySelector('#captcha') ||
        document.querySelector('[data-recaptcha]') ||
        document.querySelector('iframe[src*="recaptcha"]') ||
        document.querySelector('form[action*="recaptcha"]')
    );

    // 检查 CAPTCHA 类型
    let captchaType = 'none';
    if (hasCaptcha) {
        if (document.querySelector('.recaptcha-checkbox')) {
            captchaType = 'recaptcha_v2';
        } else if (document.querySelector('#captcha')) {
            captchaType = 'custom_captcha';
        } else if (document.querySelector('iframe[src*="recaptcha"]')) {
            captchaType = 'recaptcha_iframe';
        }
    }

    // 检查是否是 "Unusual traffic" 页面
    const isUnusualTraffic = document.body.textContent.includes('unusual traffic') ||
                             document.body.textContent.includes('captcha') ||
                             document.body.textContent.includes('verify you are human');

    // 检查是否有搜索结果容器
    const searchContainers = document.querySelectorAll('div.g, div[data-hveid], div.tF2Cxc');
    const hasResults = searchContainers.length > 0;

    // 检查 URL
    const currentUrl = window.location.href;

    // 检查页面标题
    const pageTitle = document.title;

    // 检查是否有 Google 搜索框（确认在正确的页面）
    const hasSearchBox = !!document.querySelector('textarea[name="q"], input[name="q"]');

    return JSON.stringify({
        has_captcha: hasCaptcha,
        captcha_type: captchaType,
        is_unusual_traffic: isUnusualTraffic,
        has_results: hasResults,
        result_containers: searchContainers.length,
        current_url: currentUrl,
        page_title: pageTitle,
        has_search_box: hasSearchBox
    });
})()
"""

            diag_request = page_pb2.EvaluateRequest(
                page_id=self.page_id,
                expression=diagnostic_script,
                await_promise=False
            )

            diag_response = self.page.Evaluate(diag_request)

            if not diag_response.HasField('error'):
                diag_result = json.loads(diag_response.result.string_value)
                logger.info(f"页面诊断:")
                logger.info(f"  URL: {diag_result.get('current_url')}")
                logger.info(f"  标题: {diag_result.get('page_title')}")
                logger.info(f"  有搜索框: {diag_result.get('has_search_box')}")
                logger.info(f"  有 CAPTCHA: {diag_result.get('has_captcha')}")
                logger.info(f"  结果容器数: {diag_result.get('result_containers')}")

                # CAPTCHA 检测和处理
                if diag_result.get('has_captcha') or diag_result.get('is_unusual_traffic'):
                    logger.error("=" * 60)
                    logger.error("⚠️  Google 已触发 CAPTCHA 验证")
                    logger.error(f"   CAPTCHA 类型: {diag_result.get('captcha_type')}")
                    logger.error(f"   异常流量: {diag_result.get('is_unusual_traffic')}")
                    logger.error("")
                    logger.error("   可能原因:")
                    logger.error("   1. IP 地址被 Google 标记为可疑")
                    logger.error("   2. 请求频率过高")
                    logger.error("   3. 自动化行为被检测")
                    logger.error("")
                    logger.error("   建议解决方案:")
                    logger.error("   1. 更换代理 IP（推荐使用住宅代理）")
                    logger.error("   2. 降低请求频率，增加延迟")
                    logger.error("   3. 使用 CAPTCHA 解决服务（如 2Captcha）")
                    logger.error("=" * 60)
                    return []

            logger.debug(f"开始提取搜索结果...")

            # 提取搜索结果
            all_results = []
            page_num = 0

            while len(all_results) < max_results:
                # 通过 JavaScript 提取当前页结果
                results = self._extract_results_from_page()

                # 添加排名
                for i, result in enumerate(results, start=len(all_results) + 1):
                    result['rank'] = i

                all_results.extend(results)

                logger.info(f"第 {page_num + 1} 页: 提取到 {len(results)} 个结果")

                if len(all_results) >= max_results:
                    break

                # 检查是否有下一页
                if not self._has_next_page():
                    logger.info("没有更多结果页")
                    break

                # 点击下一页
                if not self._click_next_page():
                    logger.warning("无法点击下一页")
                    break

                page_num += 1
                time.sleep(2)  # 页面间延迟

            return all_results[:max_results]

        except grpc.RpcError as e:
            logger.error(f"搜索过程中发生 gRPC 错误: {e.code().name}")
            logger.debug("堆栈跟踪:", exc_info=True)
            return []
        except Exception as e:
            logger.error(f"搜索过程中发生未预期错误: {type(e).__name__} - {str(e)}")
            logger.debug("堆栈跟踪:", exc_info=True)
            return []

    def _extract_results_from_page(self) -> List[Dict]:
        """使用 JavaScript 从当前页面提取搜索结果

        Returns:
            搜索结果列表
        """
        script = """
        (() => {
            const results = [];
            const debug = {selectors_tested: [], found_elements: 0};

            // 尝试多种选择器模式（2026年1月更新）
            const selectorPatterns = [
                'div.g',                                    // 经典选择器
                'div[data-hveid]',                          // 带属性的容器
                'div.tF2Cxc',                               // 现代选择器
                'div.yuRUbf',                               // 结果容器
                'div[lang]',                                // 带语言属性
                'div[data-hveid] h3',                       // 直接找标题
                'div.MjjYud',                               // 2025新容器
                'div[data-hveid].g',                        // 组合
                'div[jscontroller]',                        // 新结构
                'div[data-tsn]',                            // tsn属性
            ];

            for (const selector of selectorPatterns) {
                const containers = document.querySelectorAll(selector);
                debug.selectors_tested.push({selector: selector, count: containers.length});

                if (containers.length > 0) {
                    // 根据选择器类型调整查找策略
                    containers.forEach(container => {
                        try {
                            let titleElement = null;
                            let linkElement = null;

                            // 策略1: 容器是结果块
                            if (container.querySelector) {
                                titleElement = container.querySelector('h3');
                                linkElement = container.querySelector('a');

                                // 策略2: 容器本身是标题（如 div[data-hveid] h3）
                                if (!titleElement && container.tagName === 'H3') {
                                    titleElement = container;
                                    linkElement = container.parentElement?.querySelector('a');
                                }
                            }

                            if (titleElement && linkElement) {
                                const rawUrl = linkElement.href;
                                let finalUrl = rawUrl;

                                // 处理 Google 重定向
                                if (rawUrl.includes('/url?q=')) {
                                    const urlMatch = rawUrl.match(/[?&]q=([^&]+)/);
                                    if (urlMatch) {
                                        finalUrl = decodeURIComponent(urlMatch[1]);
                                    }
                                }

                                const title = titleElement.textContent?.trim() || '';
                                if (title) {  // 只添加有标题的结果
                                    results.push({
                                        title: title,
                                        url: finalUrl,
                                        display_url: linkElement.href
                                    });
                                }
                            }
                        } catch (e) {
                            console.error('Error parsing result:', e);
                        }
                    });

                    debug.found_elements = results.length;
                    if (results.length > 0) {
                        break;  // 找到结果就停止尝试其他选择器
                    }
                }
            }

            // 返回结果和调试信息
            return JSON.stringify({results: results, debug: debug});
        })()
        """

        evaluate_request = page_pb2.EvaluateRequest(
            page_id=self.page_id,
            expression=script,
            await_promise=True
        )

        evaluate_response = self.page.Evaluate(evaluate_request)

        if evaluate_response.HasField('error'):
            logger.warning(f"提取结果失败: {evaluate_response.error.message}")
            return []

        try:
            response = json.loads(evaluate_response.result.string_value)
            # 新格式包含 {results: [...], debug: {...}}
            if isinstance(response, dict) and 'results' in response:
                debug_info = response.get('debug', {})
                selectors_tested = debug_info.get('selectors_tested', [])
                found_elements = debug_info.get('found_elements', 0)

                # 只在找不到结果时打印调试信息
                if found_elements == 0:
                    logger.debug(f"选择器测试结果:")
                    for s in selectors_tested:
                        logger.debug(f"  '{s['selector']}': {s['count']} 个元素")

                return response['results']
            # 旧格式直接返回数组（向后兼容）
            elif isinstance(response, list):
                return response
            else:
                logger.warning(f"未知的响应格式")
                return []
        except json.JSONDecodeError as e:
            logger.warning(f"解析结果失败: {e}")
            return []

    def _has_next_page(self) -> bool:
        """检查是否有下一页

        Returns:
            bool: 有下一页返回 True
        """
        script = """
        !!(
            document.querySelector('#pnnext') ||
            document.querySelector('a[aria-label="Next"]') ||
            document.querySelector('span.YyVfkd')
        )
        """

        evaluate_request = page_pb2.EvaluateRequest(
            page_id=self.page_id,
            expression=script,
            await_promise=False
        )

        evaluate_response = self.page.Evaluate(evaluate_request)

        if evaluate_response.HasField('error'):
            return False

        return evaluate_response.result.bool_value

    def _click_next_page(self) -> bool:
        """点击下一页按钮

        Returns:
            bool: 成功点击返回 True
        """
        # 尝试多种选择器
        selectors = [
            'a#pnnext',
            'a[aria-label="Next"]',
            'span.YyVfkd'
        ]

        for selector in selectors:
            try:
                find_request = element_pb2.FindElementRequest(
                    page_id=self.page_id,
                    selector_type=common_pb2.SELECTOR_TYPE_CSS,
                    selector=selector,
                    timeout=1000
                )

                find_response = self.element.FindElement(find_request)

                if not find_response.HasField('error'):
                    click_request = element_pb2.ClickRequest(
                        element=find_response.element
                    )

                    click_response = self.element.Click(click_request)

                    if not click_response.HasField('error'):
                        return True
            except Exception:
                continue

        return False

    def save_to_csv(self, results: List[Dict], filename: str):
        """保存结果到 CSV 文件

        Args:
            results: 搜索结果列表
            filename: 输出文件名
        """
        if not results:
            logger.info("没有结果可保存")
            return

        fieldnames = ['rank', 'title', 'url', 'display_url']

        with open(filename, 'w', newline='', encoding='utf-8-sig') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()

            for result in results:
                writer.writerow({
                    'rank': result.get('rank', ''),
                    'title': result.get('title', ''),
                    'url': result.get('url', ''),
                    'display_url': result.get('display_url', '')
                })

        logger.info(f"结果已保存到: {filename}")

    def close(self):
        """关闭浏览器和页面"""
        logger.info("=" * 50)
        logger.info("开始清理资源...")
        logger.info(f"当前状态 - browser_id: {self.browser_id}, page_id: {self.page_id}")

        # 先关闭页面
        if self.page_id:
            try:
                logger.info(f"正在关闭页面: {self.page_id}")
                self.page.ClosePage(page_pb2.ClosePageRequest(page_id=self.page_id))
                logger.info("✓ 页面已关闭")
            except grpc.RpcError as e:
                logger.warning(f"关闭页面时出错 (gRPC): [{e.code().name}] {e.details()}")
            except Exception as e:
                logger.warning(f"关闭页面时出错: {type(e).__name__} - {str(e)}")
        else:
            logger.info("页面未创建，跳过关闭")

        # 再关闭浏览器
        if self.browser_id:
            try:
                logger.info(f"正在关闭浏览器: {self.browser_id}")
                self.browser.Close(browser_pb2.CloseRequest(browser_id=self.browser_id))
                logger.info("✓ 浏览器已关闭")
            except grpc.RpcError as e:
                logger.warning(f"关闭浏览器时出错 (gRPC): [{e.code().name}] {e.details()}")
            except Exception as e:
                logger.warning(f"关闭浏览器时出错: {type(e).__name__} - {str(e)}")
        else:
            logger.info("浏览器未创建，跳过关闭")

        # 最后关闭通道
        try:
            logger.info("正在关闭 gRPC 通道")
            self.channel.close()
            logger.info("✓ gRPC 通道已关闭")
        except Exception as e:
            logger.warning(f"关闭通道时出错: {type(e).__name__} - {str(e)}")

        logger.info("=" * 50)
        logger.info("资源清理完成")


def main():
    """主函数 - 命令行接口"""
    parser = argparse.ArgumentParser(
        description='Google 搜索爬虫 - 基于 chaser-oxide-server',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
示例:
  # 基本使用
  python google_search.py

  # 自定义搜索词
  python google_search.py --query "Python 教程"

  # 指定结果数量
  python google_search.py --max-results 50

  # 使用代理
  python google_search.py --proxy http://127.0.0.1:8118

  # 完整参数
  python google_search.py --query "机器学习" --max-results 100 --output ml_results.csv
        '''
    )

    parser.add_argument(
        '--query', '-q',
        type=str,
        default='通过图片定位地理位置',
        help='搜索关键词 (默认: "通过图片定位地理位置")'
    )

    parser.add_argument(
        '--max-results', '-n',
        type=int,
        default=100,
        help='最大结果数 (默认: 100)'
    )

    parser.add_argument(
        '--output', '-o',
        type=str,
        default='search_results.csv',
        help='输出 CSV 文件 (默认: search_results.csv)'
    )

    parser.add_argument(
        '--host',
        type=str,
        default='localhost:50051',
        help='chaser-oxide 服务器地址 (默认: localhost:50051)'
    )

    parser.add_argument(
        '--proxy',
        type=str,
        help='代理服务器 (覆盖环境变量, 格式: http://host:port 或 socks5://host:port)'
    )

    args = parser.parse_args()

    # 处理代理配置
    if args.proxy:
        proxy_config = ProxyConfig(
            http_proxy=args.proxy,
            https_proxy=args.proxy
        )
    else:
        proxy_config = ProxyConfig.from_env()

    logger.info("=" * 60)
    logger.info("Google 搜索爬虫")
    logger.info("=" * 60)
    logger.info(f"搜索关键词: {args.query}")
    logger.info(f"最大结果数: {args.max_results}")
    logger.info(f"输出文件: {args.output}")

    if proxy_config.is_active():
        logger.info(f"代理配置: {proxy_config}")

    logger.info("=" * 60)

    scraper = GoogleSearchScraper(
        host=args.host,
        proxy_config=proxy_config
    )

    try:
        # 启动浏览器
        if not scraper.launch_browser():
            logger.error("浏览器启动失败，程序退出")
            return 1

        # 执行搜索
        logger.info(f"开始搜索: {args.query}")
        results = scraper.search(args.query, args.max_results)

        if not results:
            logger.warning("未获取到搜索结果")

        # 保存结果
        scraper.save_to_csv(results, args.output)

        logger.info("=" * 60)
        logger.info(f"搜索完成! 共获取 {len(results)} 个结果")
        logger.info(f"结果已保存到: {args.output}")
        logger.info("=" * 60)

        return 0

    except grpc.RpcError as e:
        logger.error(f"gRPC 错误: {e.code().name} - {e.details()}")
        logger.error(f"  gRPC 状态码: {e.code().value}")
        logger.debug("堆栈跟踪:", exc_info=True)
        return 1
    except KeyboardInterrupt:
        logger.warning("用户中断")
        return 130
    except Exception as e:
        logger.error(f"未预期错误: {type(e).__name__}")
        logger.error(f"  消息: {str(e)}")
        logger.debug("堆栈跟踪:", exc_info=True)
        return 1
    finally:
        # 确保资源无论如何都会被清理
        logger.info("主函数 finally 块：确保清理资源")
        if scraper:
            try:
                scraper.close()
            except Exception as e:
                logger.error(f"清理资源时发生错误: {type(e).__name__} - {str(e)}")
                logger.debug("堆栈跟踪:", exc_info=True)


if __name__ == "__main__":
    sys.exit(main())
