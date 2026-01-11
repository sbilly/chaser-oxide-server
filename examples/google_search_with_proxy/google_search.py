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
import sys
import time
import grpc
from typing import List, Dict, Optional

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
)

from config import ProxyConfig


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
        self.proxy_config = proxy_config or ProxyConfig.from_env()

        self.browser_id = None
        self.page_id = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def launch_browser(self) -> bool:
        """启动浏览器并配置代理

        Returns:
            bool: 成功返回 True，失败返回 False
        """
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
                ignore_certificate_errors=True  # 代理可能使用自签名证书
            )
        )

        launch_response = self.browser.Launch(launch_request)

        if launch_response.HasField('error'):
            print(f"启动浏览器失败: {launch_response.error.message}")
            return False

        self.browser_id = launch_response.browser_info.browser_id
        print(f"浏览器已启动: {self.browser_id}")

        if proxy_server:
            print(f"  使用代理: {proxy_server}")
        if proxy_bypass:
            print(f"  绕过列表: {proxy_bypass}")

        # 创建页面
        create_page_request = page_pb2.CreatePageRequest(
            browser_id=self.browser_id,
            url="about:blank"
        )

        create_page_response = self.page.CreatePage(create_page_request)

        if create_page_response.HasField('error'):
            print(f"创建页面失败: {create_page_response.error.message}")
            return False

        self.page_id = create_page_response.page_info.page_id
        print(f"页面已创建: {self.page_id}")

        return True

    def search(self, query: str, max_results: int = 100) -> List[Dict]:
        """执行 Google 搜索

        Args:
            query: 搜索关键词
            max_results: 最大结果数

        Returns:
            搜索结果列表，每个结果包含 rank, title, url, display_url
        """
        # 导航到 Google
        navigate_request = page_pb2.NavigateRequest(
            page_id=self.page_id,
            url="https://www.google.com",
            options=common_pb2.NavigationOptions(
                timeout=30000,
                wait_until=common_pb2.NavigationOptions.LOAD_STATE_NETWORK_IDLE
            )
        )

        navigate_response = self.page.Navigate(navigate_request)

        if navigate_response.HasField('error'):
            raise Exception(f"导航失败: {navigate_response.error.message}")

        print(f"已导航到: https://www.google.com")

        # 等待页面加载
        time.sleep(2)

        # 查找搜索框
        find_request = element_pb2.FindElementRequest(
            page_id=self.page_id,
            selector_type=common_pb2.SELECTOR_TYPE_CSS,
            selector="textarea[name='q']",
            wait_for_visible=True,
            timeout=10000
        )

        find_response = self.element.FindElement(find_request)

        if find_response.HasField('error'):
            raise Exception(f"查找搜索框失败: {find_response.error.message}")

        search_box = find_response.element

        # 输入搜索词
        type_request = element_pb2.TypeRequest(
            element=search_box,
            text=query,
            clear_first=True
        )

        type_response = self.element.Type(type_request)

        if type_response.HasField('error'):
            raise Exception(f"输入搜索词失败: {type_response.error.message}")

        print(f"已输入搜索词: {query}")

        # 使用 JavaScript 提交搜索表单（比 PressKey 更可靠）
        submit_script = """
(() => {
    // 方法1: 提交表单
    const form = document.querySelector('form');
    if (form) {
        form.submit();
        return {method: 'form_submit', success: true};
    }

    // 方法2: 触发搜索按钮点击
    const submitButton = document.querySelector('input[type="submit"]');
    if (submitButton) {
        submitButton.click();
        return {method: 'button_click', success: true};
    }

    // 方法3: 直接构造 URL 并导航
    const searchInput = document.querySelector('textarea[name="q"], input[name="q"]');
    if (searchInput && searchInput.value) {
        const searchUrl = 'https://www.google.com/search?q=' + encodeURIComponent(searchInput.value);
        window.location.href = searchUrl;
        return {method: 'direct_navigation', success: true};
    }

    return {method: 'none', success: false, error: 'No submission method found'};
})()
"""

        print(f"  [DEBUG] 准备提交搜索表单...")

        evaluate_request = page_pb2.EvaluateRequest(
            page_id=self.page_id,
            expression=submit_script,
            await_promise=True
        )

        evaluate_response = self.page.Evaluate(evaluate_request)

        if evaluate_response.HasField('error'):
            print(f"  [DEBUG] JavaScript 提交失败: {evaluate_response.error.message}")
            # 回退到 PressKey 方法
            print(f"  [DEBUG] 回退使用 PressKey 方法...")
            press_key_request = element_pb2.PressKeyRequest(
                element=search_box,
                key="Enter"
            )
            press_key_response = self.element.PressKey(press_key_request)

            if press_key_response.HasField('error'):
                raise Exception(f"提交搜索失败: {press_key_response.error.message}")
        else:
            result = json.loads(evaluate_response.result.string_value)
            print(f"  [DEBUG] 提交方法: {result.get('method')}, 成功: {result.get('success')}")

        print(f"  [DEBUG] 等待搜索结果加载...")
        time.sleep(3)
        print(f"  [DEBUG] 开始提取搜索结果...")

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

            print(f"第 {page_num + 1} 页: 提取到 {len(results)} 个结果")

            if len(all_results) >= max_results:
                break

            # 检查是否有下一页
            if not self._has_next_page():
                print("没有更多结果页")
                break

            # 点击下一页
            if not self._click_next_page():
                print("无法点击下一页")
                break

            page_num += 1
            time.sleep(2)  # 页面间延迟

        return all_results[:max_results]

    def _extract_results_from_page(self) -> List[Dict]:
        """使用 JavaScript 从当前页面提取搜索结果

        Returns:
            搜索结果列表
        """
        script = """
        (() => {
            const results = [];
            const debug = {selectors_tested: [], found_elements: 0};

            // 尝试多种选择器模式
            const selectorPatterns = [
                'div.g',                                    // 经典选择器
                'div[data-hveid]',                          // 带属性的容器
                'div.tF2Cxc',                               // 现代选择器
                'div.yuRUbf',                               // 结果容器
                'div[lang]',                                // 带语言属性
                'div[data-hveid] h3',                       // 直接找标题
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
            print(f"  警告: 提取结果失败: {evaluate_response.error.message}")
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
                    print(f"  [DEBUG] 选择器测试结果:")
                    for s in selectors_tested:
                        print(f"           '{s['selector']}': {s['count']} 个元素")

                return response['results']
            # 旧格式直接返回数组（向后兼容）
            elif isinstance(response, list):
                return response
            else:
                print(f"  警告: 未知的响应格式")
                return []
        except json.JSONDecodeError as e:
            print(f"  警告: 解析结果失败: {e}")
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
            print("没有结果可保存")
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

        print(f"结果已保存到: {filename}")

    def close(self):
        """关闭浏览器和页面"""
        if self.page_id:
            try:
                self.page.ClosePage(page_pb2.ClosePageRequest(page_id=self.page_id))
                print("页面已关闭")
            except Exception:
                pass

        if self.browser_id:
            try:
                self.browser.Close(browser_pb2.CloseRequest(browser_id=self.browser_id))
                print("浏览器已关闭")
            except Exception:
                pass

        self.channel.close()


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

    print("=" * 60)
    print("Google 搜索爬虫")
    print("=" * 60)
    print(f"搜索关键词: {args.query}")
    print(f"最大结果数: {args.max_results}")
    print(f"输出文件: {args.output}")

    if proxy_config.is_active():
        print(f"代理配置: {proxy_config}")

    print("=" * 60)

    try:
        with GoogleSearchScraper(
            host=args.host,
            proxy_config=proxy_config
        ) as scraper:
            # 启动浏览器
            if not scraper.launch_browser():
                print("\n启动浏览器失败")
                return 1

            # 执行搜索
            print(f"\n开始搜索: {args.query}")
            results = scraper.search(args.query, args.max_results)

            # 保存结果
            scraper.save_to_csv(results, args.output)

            print("\n" + "=" * 60)
            print(f"搜索完成! 共获取 {len(results)} 个结果")
            print(f"结果已保存到: {args.output}")
            print("=" * 60)

            return 0

    except KeyboardInterrupt:
        print("\n\n用户中断")
        return 130
    except Exception as e:
        print(f"\n错误: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
