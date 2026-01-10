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
import sys
from typing import Generator
import time

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


def example_basic_navigation():
    """基础导航示例"""
    print("=" * 60)
    print("基础导航示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 1. 启动浏览器
        print("\n1. 启动浏览器...")
        launch_request = browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True,  # 无头模式
                window_width=1920,
                window_height=1080,
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            )
        )
        launch_response = client.browser.Launch(launch_request)

        if launch_response.HasField('error'):
            print(f"   启动失败: {launch_response.error.message}")
            return

        browser_id = launch_response.browser_info.browser_id
        print(f"   浏览器已启动: {browser_id}")

        # 2. 创建新页面
        print("\n2. 创建新页面...")
        create_page_request = page_pb2.CreatePageRequest(
            browser_id=browser_id,
            url="about:blank"
        )
        create_page_response = client.page.CreatePage(create_page_request)

        if create_page_response.HasField('error'):
            print(f"   创建页面失败: {create_page_response.error.message}")
            return

        page_id = create_page_response.page_info.page_id
        print(f"   页面已创建: {page_id}")

        # 3. 导航到 URL
        print("\n3. 导航到 example.com...")
        navigate_request = page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com",
            options=common_pb2.NavigationOptions(
                timeout=30000,
                wait_until=common_pb2.NavigationOptions.LOAD_STATE_NETWORK_IDLE
            )
        )
        navigate_response = client.page.Navigate(navigate_request)

        if navigate_response.HasField('error'):
            print(f"   导航失败: {navigate_response.error.message}")
            return

        print(f"   导航成功: {navigate_response.result.url}")
        print(f"   状态码: {navigate_response.result.status_code}")

        # 4. 获取页面标题
        print("\n4. 获取页面内容...")
        snapshot_request = page_pb2.GetSnapshotRequest(page_id=page_id)
        snapshot_response = client.page.GetSnapshot(snapshot_request)

        if snapshot_response.HasField('error'):
            print(f"   获取快照失败: {snapshot_response.error.message}")
        else:
            print(f"   页面标题: {snapshot_response.snapshot.title}")

        # 5. 截图
        print("\n5. 截取页面截图...")
        screenshot_request = page_pb2.ScreenshotRequest(
            page_id=page_id,
            options=common_pb2.ScreenshotOptions(
                format=common_pb2.ScreenshotOptions.FORMAT_PNG,
                full_page=True
            )
        )
        screenshot_response = client.page.Screenshot(screenshot_request)

        if screenshot_response.HasField('error'):
            print(f"   截图失败: {screenshot_response.error.message}")
        else:
            print(f"   截图成功: {len(screenshot_response.result.data)} bytes")
            # 保存截图
            with open("screenshot.png", "wb") as f:
                f.write(screenshot_response.result.data)
            print(f"   已保存到: screenshot.png")

        # 6. 执行 JavaScript
        print("\n6. 执行 JavaScript...")
        evaluate_request = page_pb2.EvaluateRequest(
            page_id=page_id,
            expression="document.title",
            await_promise=True
        )
        evaluate_response = client.page.Evaluate(evaluate_request)

        if evaluate_response.HasField('error'):
            print(f"   执行失败: {evaluate_response.error.message}")
        else:
            print(f"   执行结果: {evaluate_response.result.string_value}")

        # 7. 清理资源
        print("\n7. 清理资源...")
        close_page_request = page_pb2.ClosePageRequest(page_id=page_id)
        client.page.ClosePage(close_page_request)

        close_browser_request = browser_pb2.CloseRequest(browser_id=browser_id)
        client.browser.Close(close_browser_request)

        print("   资源已清理")

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_element_interaction():
    """元素交互示例"""
    print("\n" + "=" * 60)
    print("元素交互示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 启动浏览器和页面
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        browser_id = launch_response.browser_info.browser_id

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

        # 导航到测试页面
        client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))

        # 查找元素
        print("\n1. 查找 h1 元素...")
        find_request = element_pb2.FindElementRequest(
            page_id=page_id,
            selector_type=common_pb2.SELECTOR_TYPE_CSS,
            selector="h1"
        )
        find_response = client.element.FindElement(find_request)

        if find_response.HasField('error'):
            print(f"   查找失败: {find_response.error.message}")
            return

        element = find_response.element
        print(f"   找到元素: {element.element_id}")

        # 获取元素文本
        print("\n2. 获取元素文本...")
        text_request = element_pb2.GetTextRequest(element=element)
        text_response = client.element.GetText(text_request)

        if not text_response.HasField('error'):
            print(f"   文本内容: {text_response.text.text}")

        # 获取元素属性
        print("\n3. 获取元素属性...")
        attr_request = element_pb2.GetAttributeRequest(
            element=element,
            name="class"
        )
        attr_response = client.element.GetAttribute(attr_request)

        if not attr_response.HasField('error'):
            print(f"   class 属性: {attr_response.value.value}")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_event_subscription():
    """事件订阅示例"""
    print("\n" + "=" * 60)
    print("事件订阅示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 启动浏览器和页面
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        browser_id = launch_response.browser_info.browser_id

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

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

        print("\n1. 订阅页面事件...")

        # 在后台线程中处理事件
        event_count = 0

        try:
            # 设置较短的超时用于演示
            for event in client.events.Subscribe(event_generator()):
                event_count += 1

                print(f"\n2. 收到事件 #{event_count}:")
                print(f"   类型: {event_pb2.EventType.Name(event.metadata.type)}")
                print(f"   时间戳: {event.metadata.timestamp}")

                # 解析不同类型的事件
                if event.metadata.type == event_pb2.EVENT_TYPE_PAGE_LOADED:
                    print(f"   URL: {event.page_event.url}")
                    print(f"   标题: {event.page_event.title}")

                elif event.metadata.type == event_pb2.EVENT_TYPE_CONSOLE_LOG:
                    print(f"   日志级别: {event_pb2.ConsoleEvent.LogLevel.Name(event.console_event.level)}")
                    print(f"   内容: {event.console_event.args}")

                elif event.metadata.type == event_pb2.EVENT_TYPE_RESPONSE_RECEIVED:
                    print(f"   URL: {event.network_event.url}")
                    print(f"   状态码: {event.network_event.status_code}")

                # 收到 5 个事件后退出
                if event_count >= 5:
                    break

        except grpc.RpcError as e:
            print(f"\nRPC 错误: {e.code()} - {e.details()}")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_stealth_browsing():
    """隐身浏览示例"""
    print("\n" + "=" * 60)
    print("隐身浏览示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 1. 创建 Windows 指纹配置
        print("\n1. 创建 Windows 指纹配置...")
        profile_request = profile_pb2.CreateProfileRequest(
            type=profile_pb2.PROFILE_TYPE_WINDOWS
        )
        profile_response = client.profile.CreateProfile(profile_request)

        if profile_response.HasField('error'):
            print(f"   创建配置失败: {profile_response.error.message}")
            return

        profile_id = profile_response.profile.profile_id
        print(f"   配置已创建: {profile_id}")
        print(f"   User-Agent: {profile_response.profile.fingerprint.headers.user_agent}")

        # 2. 启动浏览器
        print("\n2. 启动浏览器...")
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True,
                user_agent=profile_response.profile.fingerprint.headers.user_agent
            )
        ))
        browser_id = launch_response.browser_info.browser_id

        # 3. 创建页面
        print("\n3. 创建页面...")
        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

        # 4. 应用指纹配置
        print("\n4. 应用指纹配置...")
        apply_request = profile_pb2.ApplyProfileRequest(
            page_id=page_id,
            profile_id=profile_id
        )
        apply_response = client.profile.ApplyProfile(apply_request)

        if apply_response.HasField('error'):
            print(f"   应用配置失败: {apply_response.error.message}")
        else:
            print(f"   配置已应用")
            print(f"   应用特性: {apply_response.result.applied_features}")

        # 5. 访问测试页面并检查指纹
        print("\n5. 检查浏览器指纹...")
        client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))

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
            print(f"   检测结果:")
            # 解析 JSON 结果
            import json
            fingerprint = json.loads(eval_response.result.string_value)
            for key, value in fingerprint.items():
                print(f"   {key}: {value}")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


if __name__ == "__main__":
    # 运行所有示例
    example_basic_navigation()
    example_element_interaction()
    example_event_subscription()
    example_stealth_browsing()

    print("\n" + "=" * 60)
    print("所有示例执行完成！")
    print("=" * 60)
