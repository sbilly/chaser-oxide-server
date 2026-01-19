"""
Chaser-Oxide Python 客户端 - 基础示例

演示如何使用 Python 客户端进行基本的浏览器自动化操作。

依赖安装:
    pip install grpcio grpcio-tools

使用方法:
    # 从项目根目录运行
    cd docs/examples/python
    python basic_client.py

或者将 chaser 包安装到 Python 环境:
    pip install -e .
"""

import grpc
import logging
import sys
from typing import Generator
import time

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

# 添加当前目录到路径以导入 chaser 包
sys.path.insert(0, '.')

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
    event_pb2,
    event_pb2_grpc,
)


class ChaserOxideClient:
    """Chaser-Oxide gRPC 客户端封装"""

    def __init__(self, host: str = "localhost:50051"):
        """初始化客户端连接

        Args:
            host: gRPC 服务器地址，格式为 "host:port"
        """
        self.channel = grpc.insecure_channel(host)
        self.browser = browser_pb2_grpc.BrowserServiceStub(self.channel)
        self.page = page_pb2_grpc.PageServiceStub(self.channel)
        self.element = element_pb2_grpc.ElementServiceStub(self.channel)
        self.profile = profile_pb2_grpc.ProfileServiceStub(self.channel)
        self.events = event_pb2_grpc.EventServiceStub(self.channel)

    def close(self):
        """关闭客户端连接"""
        self.channel.close()


# ============================================================================
# 错误处理工具
# ============================================================================

# 错误代码映射表
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
    """检查响应中的错误字段，记录并返回是否有错误

    Args:
        response: gRPC 响应对象
        operation: 操作名称（用于日志）

    Returns:
        bool: True 表示有错误，False 表示无错误
    """
    if response.HasField('error'):
        error_code = response.error.code
        error_message = response.error.message
        error_name = ERROR_CODE_NAMES.get(error_code, f"CODE_{error_code}")

        logger.error(f"{operation} 失败: [{error_name}] {error_message}")
        logger.error(f"  错误代码: {error_code}")
        logger.debug(f"  完整错误响应: {response.error}")
        return True
    return False


def handle_errors(func):
    """错误处理装饰器，提供统一的错误日志和优雅退出

    Args:
        func: 被装饰的函数

    Returns:
        包装后的函数，带有统一的错误处理逻辑
    """
    def wrapper(*args, **kwargs):
        func_name = func.__name__
        logger.info(f"开始执行: {func_name}")
        try:
            result = func(*args, **kwargs)
            logger.info(f"成功完成: {func_name}")
            return result
        except grpc.RpcError as e:
            logger.error(f"gRPC 错误: {e.code().name} - {e.details()}")
            logger.error(f"  操作: {func_name}")
            logger.error(f"  gRPC 状态码: {e.code().value}")
            logger.debug(f"  调试信息: {e}")
            logger.debug("堆栈跟踪:", exc_info=True)
            sys.exit(1)
        except KeyboardInterrupt:
            logger.warning(f"用户中断: {func_name}")
            sys.exit(130)
        except Exception as e:
            logger.error(f"未预期错误: {type(e).__name__}")
            logger.error(f"  消息: {str(e)}")
            logger.error(f"  操作: {func_name}")
            logger.debug("堆栈跟踪:", exc_info=True)
            sys.exit(1)
    return wrapper


@handle_errors
def example_basic_navigation():
    """基础导航示例"""
    logger.info("=" * 60)
    logger.info("基础导航示例")
    logger.info("=" * 60)

    client = ChaserOxideClient()
    browser_id = None
    page_id = None

    try:
        # 1. 启动浏览器
        logger.info("1. 启动浏览器...")
        launch_request = browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True,  # 无头模式
                window_width=1920,
                window_height=1080,
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            )
        )
        launch_response = client.browser.Launch(launch_request)

        if check_error(launch_response, "启动浏览器"):
            sys.exit(1)

        browser_id = launch_response.browser_info.browser_id
        logger.info(f"   浏览器已启动: {browser_id}")

        # 2. 创建新页面
        logger.info("2. 创建新页面...")
        create_page_request = page_pb2.CreatePageRequest(
            browser_id=browser_id,
            url="about:blank"
        )
        create_page_response = client.page.CreatePage(create_page_request)

        if check_error(create_page_response, "创建页面"):
            sys.exit(1)

        page_id = create_page_response.page_info.page_id
        logger.info(f"   页面已创建: {page_id}")

        # 3. 导航到 URL
        logger.info("3. 导航到 example.com...")
        navigate_request = page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com",
            options=common_pb2.NavigationOptions(
                timeout=30000,
                wait_until=common_pb2.NavigationOptions.LOAD_STATE_NETWORK_IDLE
            )
        )
        navigate_response = client.page.Navigate(navigate_request)

        if check_error(navigate_response, "导航"):
            sys.exit(1)

        logger.info(f"   导航成功: {navigate_response.result.url}")
        logger.info(f"   状态码: {navigate_response.result.status_code}")

        # 4. 获取页面标题
        logger.info("4. 获取页面内容...")
        snapshot_request = page_pb2.GetSnapshotRequest(page_id=page_id)
        snapshot_response = client.page.GetSnapshot(snapshot_request)

        if check_error(snapshot_response, "获取快照"):
            logger.warning("   跳过快照获取")
        else:
            logger.info(f"   页面标题: {snapshot_response.snapshot.title}")

        # 5. 截图
        logger.info("5. 截取页面截图...")
        screenshot_request = page_pb2.ScreenshotRequest(
            page_id=page_id,
            options=common_pb2.ScreenshotOptions(
                format=common_pb2.ScreenshotOptions.FORMAT_PNG,
                full_page=True
            )
        )
        screenshot_response = client.page.Screenshot(screenshot_request)

        if check_error(screenshot_response, "截图"):
            logger.warning("   跳过截图保存")
        else:
            logger.info(f"   截图成功: {len(screenshot_response.result.data)} bytes")
            # 保存截图
            with open("screenshot.png", "wb") as f:
                f.write(screenshot_response.result.data)
            logger.info(f"   已保存到: screenshot.png")

        # 6. 执行 JavaScript
        logger.info("6. 执行 JavaScript...")
        evaluate_request = page_pb2.EvaluateRequest(
            page_id=page_id,
            expression="document.title",
            await_promise=True
        )
        evaluate_response = client.page.Evaluate(evaluate_request)

        if check_error(evaluate_response, "执行 JavaScript"):
            logger.warning("   跳过 JavaScript 执行结果")
        else:
            logger.info(f"   执行结果: {evaluate_response.result.string_value}")

    finally:
        # 清理资源（无论是否出错都会执行）
        if page_id:
            try:
                client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
                logger.info("页面已关闭")
            except Exception as e:
                logger.debug(f"关闭页面时出错（已忽略）: {e}")

        if browser_id:
            try:
                client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))
                logger.info("浏览器已关闭")
            except Exception as e:
                logger.debug(f"关闭浏览器时出错（已忽略）: {e}")

        client.close()
        logger.info("客户端连接已关闭")


@handle_errors
def example_element_interaction():
    """元素交互示例"""
    logger.info("\n" + "=" * 60)
    logger.info("元素交互示例")
    logger.info("=" * 60)

    client = ChaserOxideClient()
    browser_id = None
    page_id = None

    try:
        # 启动浏览器和页面
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        if check_error(launch_response, "启动浏览器"):
            sys.exit(1)
        browser_id = launch_response.browser_info.browser_id
        logger.info(f"浏览器已启动: {browser_id}")

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        if check_error(page_response, "创建页面"):
            sys.exit(1)
        page_id = page_response.page_info.page_id
        logger.info(f"页面已创建: {page_id}")

        # 导航到测试页面
        navigate_response = client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))
        if check_error(navigate_response, "导航"):
            sys.exit(1)

        # 查找元素
        logger.info("1. 查找 h1 元素...")
        find_request = element_pb2.FindElementRequest(
            page_id=page_id,
            selector_type=common_pb2.SELECTOR_TYPE_CSS,
            selector="h1"
        )
        find_response = client.element.FindElement(find_request)

        if check_error(find_response, "查找元素"):
            sys.exit(1)

        element = find_response.element
        logger.info(f"   找到元素: {element.element_id}")

        # 获取元素文本
        logger.info("2. 获取元素文本...")
        text_request = element_pb2.GetTextRequest(element=element)
        text_response = client.element.GetText(text_request)

        if not text_response.HasField('error'):
            logger.info(f"   文本内容: {text_response.text.text}")

        # 获取元素属性
        logger.info("3. 获取元素属性...")
        attr_request = element_pb2.GetAttributeRequest(
            element=element,
            name="class"
        )
        attr_response = client.element.GetAttribute(attr_request)

        if not attr_response.HasField('error'):
            logger.info(f"   class 属性: {attr_response.value.value}")

    finally:
        # 清理资源（无论是否出错都会执行）
        if page_id:
            try:
                client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
                logger.info("页面已关闭")
            except Exception as e:
                logger.debug(f"关闭页面时出错（已忽略）: {e}")

        if browser_id:
            try:
                client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))
                logger.info("浏览器已关闭")
            except Exception as e:
                logger.debug(f"关闭浏览器时出错（已忽略）: {e}")

        client.close()
        logger.info("客户端连接已关闭")


@handle_errors
def example_event_subscription():
    """事件订阅示例"""
    logger.info("\n" + "=" * 60)
    logger.info("事件订阅示例")
    logger.info("=" * 60)

    client = ChaserOxideClient()

    # 在 try 块外部声明变量，确保 finally 块可以访问
    browser_id = None
    page_id = None

    try:
        # 启动浏览器和页面
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        if check_error(launch_response, "启动浏览器"):
            sys.exit(1)
        browser_id = launch_response.browser_info.browser_id
        logger.info(f"浏览器已启动: {browser_id}")

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        if check_error(page_response, "创建页面"):
            sys.exit(1)
        page_id = page_response.page_info.page_id
        logger.info(f"页面已创建: {page_id}")

        # 定义事件流生成器
        def event_generator() -> Generator[event_pb2.SubscribeRequest, None, None]:
            # 订阅事件
            yield event_pb2.SubscribeRequest(
                action=event_pb2.SubscribeRequest.ACTION_SUBSCRIBE,
                subscription=event_pb2.Subscription(
                    page_id=page_id,
                    event_types=[
                        event_pb2.EVENT_TYPE_PAGE_LOADED,
                        event_pb2.EVENT_TYPE_CONSOLE_LOG,
                        event_pb2.EVENT_TYPE_RESPONSE_RECEIVED
                    ]
                )
            )
            # 保持连接
            while True:
                time.sleep(1)
                yield event_pb2.SubscribeRequest(action=event_pb2.SubscribeRequest.ACTION_PING)

        logger.info("1. 订阅页面事件...")

        # 在后台线程中处理事件
        event_count = 0

        try:
            # 设置较短的超时用于演示
            for event in client.events.Subscribe(event_generator()):
                event_count += 1

                logger.info(f"\n2. 收到事件 #{event_count}:")
                logger.info(f"   类型: {event_pb2.EventType.Name(event.metadata.type)}")
                logger.info(f"   时间戳: {event.metadata.timestamp}")

                # 解析不同类型的事件
                if event.metadata.type == event_pb2.EVENT_TYPE_PAGE_LOADED:
                    logger.info(f"   URL: {event.page_event.url}")
                    logger.info(f"   标题: {event.page_event.title}")

                elif event.metadata.type == event_pb2.EVENT_TYPE_CONSOLE_LOG:
                    logger.info(f"   日志级别: {event_pb2.ConsoleEvent.LogLevel.Name(event.console_event.level)}")
                    logger.info(f"   内容: {event.console_event.args}")

                elif event.metadata.type == event_pb2.EVENT_TYPE_RESPONSE_RECEIVED:
                    logger.info(f"   URL: {event.network_event.url}")
                    logger.info(f"   状态码: {event.network_event.status_code}")

                # 收到 5 个事件后退出
                if event_count >= 5:
                    break

        except grpc.RpcError as e:
            logger.error(f"gRPC 错误: {e.code().name} - {e.details()}")
            sys.exit(1)

    finally:
        # 清理资源（无论是否出错都会执行）
        if page_id:
            try:
                client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
                logger.info("页面已关闭")
            except Exception as e:
                logger.debug(f"关闭页面时出错（已忽略）: {e}")

        if browser_id:
            try:
                client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))
                logger.info("浏览器已关闭")
            except Exception as e:
                logger.debug(f"关闭浏览器时出错（已忽略）: {e}")

        client.close()
        logger.info("客户端连接已关闭")


@handle_errors
def example_stealth_browsing():
    """隐身浏览示例"""
    logger.info("\n" + "=" * 60)
    logger.info("隐身浏览示例")
    logger.info("=" * 60)

    client = ChaserOxideClient()

    # 在 try 块外部声明变量，确保 finally 块可以访问
    profile_id = None
    browser_id = None
    page_id = None

    try:
        # 1. 创建 Windows 指纹配置
        logger.info("1. 创建 Windows 指纹配置...")
        profile_request = profile_pb2.CreateProfileRequest(
            type=profile_pb2.PROFILE_TYPE_WINDOWS
        )
        profile_response = client.profile.CreateProfile(profile_request)

        if check_error(profile_response, "创建配置"):
            sys.exit(1)

        profile_id = profile_response.profile.profile_id
        logger.info(f"   配置已创建: {profile_id}")
        logger.info(f"   User-Agent: {profile_response.profile.fingerprint.headers.user_agent}")

        # 2. 启动浏览器
        logger.info("2. 启动浏览器...")
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True,
                user_agent=profile_response.profile.fingerprint.headers.user_agent
            )
        ))
        if check_error(launch_response, "启动浏览器"):
            sys.exit(1)
        browser_id = launch_response.browser_info.browser_id

        # 3. 创建页面
        logger.info("3. 创建页面...")
        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        if check_error(page_response, "创建页面"):
            sys.exit(1)
        page_id = page_response.page_info.page_id

        # 4. 应用指纹配置
        logger.info("4. 应用指纹配置...")
        apply_request = profile_pb2.ApplyProfileRequest(
            page_id=page_id,
            profile_id=profile_id
        )
        apply_response = client.profile.ApplyProfile(apply_request)

        if check_error(apply_response, "应用配置"):
            logger.warning("   跳过配置应用")
        else:
            logger.info(f"   配置已应用")
            logger.info(f"   应用特性: {apply_response.result.applied_features}")

        # 5. 访问测试页面并检查指纹
        logger.info("5. 检查浏览器指纹...")
        navigate_response = client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))
        if check_error(navigate_response, "导航"):
            sys.exit(1)

        # 执行 JavaScript 检查指纹
        check_script = """
        ({
            userAgent: navigator.userAgent,
            platform: navigator.platform,
            vendor: navigator.vendor,
            hardwareConcurrency: navigator.hardwareConcurrency,
            deviceMemory: navigator.deviceMemory,
            language: navigator.language,
            screen: {
                width: screen.width,
                height: screen.height,
                colorDepth: screen.colorDepth
            }
        })
        """

        eval_response = client.page.Evaluate(page_pb2.EvaluateRequest(
            page_id=page_id,
            expression=check_script
        ))

        if not eval_response.HasField('error'):
            logger.info(f"   检测结果:")
            # 解析 JSON 结果
            import json
            fingerprint = json.loads(eval_response.result.string_value)
            for key, value in fingerprint.items():
                logger.info(f"   {key}: {value}")

    finally:
        # 清理资源（无论是否出错都会执行）
        if page_id:
            try:
                client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
                logger.info("页面已关闭")
            except Exception as e:
                logger.debug(f"关闭页面时出错（已忽略）: {e}")

        if browser_id:
            try:
                client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))
                logger.info("浏览器已关闭")
            except Exception as e:
                logger.debug(f"关闭浏览器时出错（已忽略）: {e}")

        if profile_id:
            try:
                client.profile.DeleteProfile(profile_pb2.DeleteProfileRequest(profile_id=profile_id))
                logger.info("配置已删除")
            except Exception as e:
                logger.debug(f"删除配置时出错（已忽略）: {e}")

        client.close()
        logger.info("客户端连接已关闭")


def main():
    """主函数：运行所有示例"""
    logger.info("=" * 60)
    logger.info("Chaser-Oxide Python 客户端示例")
    logger.info("=" * 60)

    examples = [
        ("基础导航", example_basic_navigation),
        ("元素交互", example_element_interaction),
        ("事件订阅", example_event_subscription),
        ("隐身浏览", example_stealth_browsing),
    ]

    for name, func in examples:
        logger.info(f"\n运行示例: {name}")
        logger.info("-" * 60)
        try:
            func()
            logger.info(f"✓ 示例 '{name}' 完成")
        except SystemExit as e:
            # handle_errors 装饰器调用 sys.exit() 时会在这里被捕获
            if e.code != 0:
                logger.error(f"✗ 示例 '{name}' 失败，退出代码: {e.code}")
                sys.exit(e.code)
        except Exception as e:
            logger.error(f"✗ 示例 '{name}' 发生未捕获异常: {type(e).__name__}")
            logger.error(f"  消息: {str(e)}")
            sys.exit(1)

    logger.info("\n" + "=" * 60)
    logger.info("所有示例执行完成！")
    logger.info("=" * 60)


if __name__ == "__main__":
    main()
