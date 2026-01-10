"""
Chaser-Oxide Python 客户端 - 隐身功能示例

演示高级隐身功能，包括指纹配置、人类行为模拟和反检测技术。

依赖安装:
    pip install grpcio grpcio-tools

使用方法:
    # 从项目根目录运行
    cd docs/examples/python
    python stealth_client.py

或者将 chaser 包安装到 Python 环境:
    pip install -e .
"""

import grpc
import sys
from typing import Generator
import time
import json

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


def example_custom_profile():
    """自定义指纹配置示例"""
    print("=" * 60)
    print("自定义指纹配置示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 1. 创建自定义配置
        print("\n1. 创建自定义指纹配置...")
        custom_request = profile_pb2.CreateCustomProfileRequest(
            profile_name="my_stealth_profile",
            template=profile_pb2.PROFILE_TYPE_WINDOWS,
            options=profile_pb2.CustomProfileOptions(
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                platform="Win32",
                screen_width=1920,
                screen_height=1080,
                device_pixel_ratio=1.0,
                cpu_cores=8,
                device_memory=16,
                locale="zh-CN",
                languages=["zh-CN", "zh", "en-US", "en"],
                timezone="Asia/Shanghai",
                webgl_vendor="Intel Inc.",
                webgl_renderer="Intel(R) UHD Graphics 630",
                navigator_vendor="Google Inc.",
                navigator_product="Gecko"
            ),
            profile_options=profile_pb2.ProfileOptions(
                inject_navigator=True,
                inject_screen=True,
                inject_webgl=True,
                inject_canvas=True,
                inject_audio=True,
                neutralize_utility_world=True,
                use_isolated_world=True,
                randomize_metrics=True,
                prevent_detection=True
            )
        )

        custom_response = client.profile.CreateCustomProfile(custom_request)

        if custom_response.HasField('error'):
            print(f"   创建配置失败: {custom_response.error.message}")
            return

        profile = custom_response.profile
        print(f"   配置已创建: {profile.profile_id}")
        print(f"   User-Agent: {profile.fingerprint.headers.user_agent}")
        print(f"   平台: {profile.fingerprint.navigator.platform}")
        print(f"   时区: {profile.timezone}")

        # 2. 启动浏览器并应用配置
        print("\n2. 启动浏览器...")
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(
                headless=True,
                user_agent=profile.fingerprint.headers.user_agent
            )
        ))
        browser_id = launch_response.browser_info.browser_id

        print("\n3. 创建页面...")
        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

        # 3. 应用指纹配置
        print("\n4. 应用指纹配置...")
        apply_request = profile_pb2.ApplyProfileRequest(
            page_id=page_id,
            profile_id=profile.profile_id,
            override_existing=True
        )
        apply_response = client.profile.ApplyProfile(apply_request)

        if apply_response.HasField('error'):
            print(f"   应用配置失败: {apply_response.error.message}")
        else:
            print(f"   配置已应用")
            print(f"   应用的特性: {', '.join(apply_response.result.applied_features)}")

        # 4. 验证指纹
        print("\n5. 验证浏览器指纹...")
        client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))

        # 执行指纹检测脚本
        detection_script = """
        ({
            // 基本 navigator 属性
            userAgent: navigator.userAgent,
            platform: navigator.platform,
            vendor: navigator.vendor,
            language: navigator.language,
            languages: navigator.languages,

            // 硬件信息
            hardwareConcurrency: navigator.hardwareConcurrency,
            deviceMemory: navigator.deviceMemory,

            // 屏幕信息
            screen: {
                width: screen.width,
                height: screen.height,
                availWidth: screen.availWidth,
                availHeight: screen.availHeight,
                colorDepth: screen.colorDepth,
                pixelDepth: screen.pixelDepth
            },

            // WebGL 指纹
            webgl: (() => {
                const canvas = document.createElement('canvas');
                const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
                if (!gl) return null;
                const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
                return {
                    vendor: debugInfo ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) : 'unknown',
                    renderer: debugInfo ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) : 'unknown'
                };
            })(),

            // Canvas 指纹
            canvas: (() => {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                ctx.textBaseline = 'top';
                ctx.font = '14px Arial';
                ctx.fillText('Hello, World!', 2, 2);
                return canvas.toDataURL().substring(0, 50) + '...';
            })(),

            // 检测自动化
            automation: {
                webdriver: navigator.webdriver,
                chrome: window.chrome ? {
                    runtime: window.chrome.runtime ? true : false
                } : null,
                permissions: navigator.permissions ? true : false
            }
        })
        """

        eval_response = client.page.Evaluate(page_pb2.EvaluateRequest(
            page_id=page_id,
            expression=detection_script
        ))

        if not eval_response.HasField('error'):
            print(f"   指纹检测结果:")
            fingerprint = json.loads(eval_response.result.string_value)

            # 显示关键指纹信息
            print(f"   User-Agent: {fingerprint['userAgent']}")
            print(f"   平台: {fingerprint['platform']}")
            print(f"   语言: {fingerprint['language']}")
            print(f"   CPU 核心数: {fingerprint['hardwareConcurrency']}")
            print(f"   设备内存: {fingerprint['deviceMemory']}GB")
            print(f"   屏幕分辨率: {fingerprint['screen']['width']}x{fingerprint['screen']['height']}")
            print(f"   WebGL Vendor: {fingerprint['webgl']['vendor']}")
            print(f"   WebGL Renderer: {fingerprint['webgl']['renderer']}")
            print(f"   Canvas 指纹: {fingerprint['canvas'][:50]}...")
            print(f"   Webdriver: {fingerprint['automation']['webdriver']}")

            # 评估反检测效果
            print(f"\n   反检测评估:")
            score = 0
            if fingerprint['automation']['webdriver'] == False:
                print(f"   ✓ Webdriver 属性已隐藏")
                score += 1
            if fingerprint['hardwareConcurrency'] >= 4:
                print(f"   ✓ CPU 核心数看起来真实")
                score += 1
            if fingerprint['deviceMemory'] >= 8:
                print(f"   ✓ 设备内存看起来真实")
                score += 1
            print(f"   总体评分: {score}/3")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_human_behavior():
    """人类行为模拟示例"""
    print("\n" + "=" * 60)
    print("人类行为模拟示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 启动浏览器
        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        browser_id = launch_response.browser_info.browser_id

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

        # 导航到测试页面
        print("\n1. 导航到测试页面...")
        client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))

        # 查找元素
        print("\n2. 查找元素...")
        find_response = client.element.FindElement(element_pb2.FindElementRequest(
            page_id=page_id,
            selector_type=common_pb2.SELECTOR_TYPE_CSS,
            selector="h1"
        ))

        if find_response.HasField('error'):
            print(f"   查找失败: {find_response.error.message}")
            return

        element = find_response.element
        print(f"   找到元素: {element.element_id}")

        # 人类式移动和点击
        print("\n3. 模拟人类鼠标移动和点击...")
        print("   使用贝塞尔曲线轨迹，速度变化，随机偏移")

        hover_request = element_pb2.HoverRequest(
            element=element,
            human_like=True,
            movement_duration=500,  # 500ms 移动时间
            bezier_curve=True       # 使用贝塞尔曲线
        )
        hover_response = client.element.Hover(hover_request)

        if not hover_response.HasField('error'):
            print(f"   鼠标悬停成功")

        time.sleep(0.3)  # 人类反应延迟

        click_request = element_pb2.ClickRequest(
            element=element,
            human_like=True,
            movement_duration=300,
            random_offset=True,     # 随机偏移
            bezier_curve=True
        )
        click_response = client.element.Click(click_request)

        if not click_response.HasField('error'):
            print(f"   点击成功")

        # 人类式输入
        print("\n4. 模拟人类输入...")
        print("   包含打字速度变化，随机停顿，偶尔回删")

        # 创建输入框（示例）
        # 实际使用时需要找到真实的输入框
        type_request = element_pb2.TypeRequest(
            element=element,
            text="Hello, World!",
            human_like=True,
            typing_speed=150,        # 平均打字速度 150ms/字符
            random_delays=True,      # 随机停顿
            occasional_backspace=True  # 偶尔回删
        )

        # 注意：这个请求可能会失败，因为 example.com 没有输入框
        # 这里仅作为演示 API 用法
        print(f"   文本: {type_request.text}")
        print(f"   打字速度: {type_request.typing_speed}ms/字符")
        print(f"   随机停顿: {type_request.random_delays}")
        print(f"   偶尔回删: {type_request.occasional_backspace}")

        # 拖拽示例
        print("\n5. 模拟人类拖拽...")
        print("   使用贝塞尔曲线轨迹，速度变化")

        # 查找可拖拽元素（示例）
        drag_request = element_pb2.DragAndDropRequest(
            source_element=element,
            target_element=element,  # 实际使用时应该是不同的元素
            human_like=True,
            movement_duration=800,
            bezier_curve=True,
            random_path=True         # 随机路径
        )

        print(f"   拖拽持续时间: {drag_request.movement_duration}ms")
        print(f"   贝塞尔曲线: {drag_request.bezier_curve}")
        print(f"   随机路径: {drag_request.random_path}")

        print("\n   人类行为模拟完成!")
        print("   提示: 所有交互都包含:")
        print("   - 贝塞尔曲线鼠标轨迹")
        print("   - 速度变化和随机停顿")
        print("   - 模拟人类反应时间")
        print("   - 随机偏移和抖动")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_randomized_profiles():
    """随机化指纹配置示例"""
    print("\n" + "=" * 60)
    print("随机化指纹配置示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 创建多个随机化配置
        print("\n1. 创建多个随机化 Windows 配置...")

        profiles = []
        for i in range(3):
            print(f"\n   生成配置 #{i + 1}...")

            randomize_request = profile_pb2.RandomizeProfileRequest(
                type=profile_pb2.PROFILE_TYPE_WINDOWS,
                options=profile_pb2.RandomizationOptions(
                    randomize_screen=True,
                    randomize_timezone=True,
                    randomize_language=True,
                    randomize_webgl=True,
                    entropy=0.8  # 80% 随机化程度
                )
            )

            randomize_response = client.profile.RandomizeProfile(randomize_request)

            if randomize_response.HasField('error'):
                print(f"   生成失败: {randomize_response.error.message}")
                continue

            profile = randomize_response.profile
            profiles.append(profile)

            print(f"   配置 ID: {profile.profile_id}")
            print(f"   User-Agent: {profile.fingerprint.headers.user_agent[:50]}...")
            print(f"   屏幕分辨率: {profile.fingerprint.screen.width}x{profile.fingerprint.screen.height}")
            print(f"   时区: {profile.timezone}")
            print(f"   语言: {', '.join(profile.languages[:2])}")

        print(f"\n   成功生成 {len(profiles)} 个随机配置")

        # 比较配置差异
        print("\n2. 比较配置差异...")
        if len(profiles) >= 2:
            p1, p2 = profiles[0], profiles[1]

            print(f"   配置 1 vs 配置 2:")
            print(f"   屏幕分辨率: {p1.fingerprint.screen.width}x{p1.fingerprint.screen.height} vs "
                  f"{p2.fingerprint.screen.width}x{p2.fingerprint.screen.height}")
            print(f"   时区: {p1.timezone} vs {p2.timezone}")
            print(f"   CPU 核心: {p1.fingerprint.hardware.cpu_cores} vs {p2.fingerprint.hardware.cpu_cores}")
            print(f"   设备内存: {p1.fingerprint.hardware.device_memory}GB vs "
                  f"{p2.fingerprint.hardware.device_memory}GB")

            if p1.fingerprint.screen.width != p2.fingerprint.screen.width:
                print(f"   ✓ 屏幕分辨率不同")
            if p1.timezone != p2.timezone:
                print(f"   ✓ 时区不同")
            if p1.fingerprint.hardware.cpu_cores != p2.fingerprint.hardware.cpu_cores:
                print(f"   ✓ CPU 核心数不同")

        # 使用随机化配置
        print("\n3. 使用随机化配置...")
        if profiles:
            profile = profiles[0]

            launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
                options=common_pb2.BrowserOptions(
                    headless=True,
                    user_agent=profile.fingerprint.headers.user_agent
                )
            ))
            browser_id = launch_response.browser_info.browser_id

            page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
                browser_id=browser_id
            ))
            page_id = page_response.page_info.page_id

            apply_response = client.profile.ApplyProfile(profile_pb2.ApplyProfileRequest(
                page_id=page_id,
                profile_id=profile.profile_id
            ))

            if not apply_response.HasField('error'):
                print(f"   配置已应用到页面")

            # 清理
            client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
            client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_anti_detection():
    """反检测技术示例"""
    print("\n" + "=" * 60)
    print("反检测技术示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 1. 创建配置并启用所有反检测选项
        print("\n1. 创建配置并启用所有反检测选项...")

        custom_request = profile_pb2.CreateCustomProfileRequest(
            profile_name="anti_detection_profile",
            template=profile_pb2.PROFILE_TYPE_WINDOWS,
            options=profile_pb2.CustomProfileOptions(
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                platform="Win32",
                cpu_cores=8,
                device_memory=16
            ),
            profile_options=profile_pb2.ProfileOptions(
                inject_navigator=True,      # 注入 navigator 属性
                inject_screen=True,         # 注入 screen 属性
                inject_webgl=True,          # 注入 WebGL 属性
                inject_canvas=True,         # Canvas 指纹保护
                inject_audio=True,          # 音频指纹保护
                neutralize_utility_world=True,  # 中立化 utility world
                use_isolated_world=True,    # 使用隔离世界
                randomize_metrics=True,     # 随机化性能指标
                prevent_detection=True      # 防止检测脚本
            )
        )

        profile_response = client.profile.CreateCustomProfile(custom_request)
        profile = profile_response.profile

        print(f"   配置已创建: {profile.profile_id}")
        print(f"   启用的反检测选项:")
        print(f"   - Navigator 注入: {profile.options.inject_navigator}")
        print(f"   - Screen 注入: {profile.options.inject_screen}")
        print(f"   - WebGL 注入: {profile.options.inject_webgl}")
        print(f"   - Canvas 保护: {profile.options.inject_canvas}")
        print(f"   - 音频保护: {profile.options.inject_audio}")
        print(f"   - Utility World 中立化: {profile.options.neutralize_utility_world}")
        print(f"   - 隔离世界: {profile.options.use_isolated_world}")
        print(f"   - 指标随机化: {profile.options.randomize_metrics}")
        print(f"   - 防检测: {profile.options.prevent_detection}")

        # 2. 启动浏览器并应用配置
        print("\n2. 启动浏览器并应用配置...")

        launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
            options=common_pb2.BrowserOptions(headless=True)
        ))
        browser_id = launch_response.browser_info.browser_id

        page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
            browser_id=browser_id
        ))
        page_id = page_response.page_info.page_id

        apply_response = client.profile.ApplyProfile(profile_pb2.ApplyProfileRequest(
            page_id=page_id,
            profile_id=profile.profile_id,
            override_existing=True
        ))

        if apply_response.HasField('error'):
            print(f"   应用配置失败: {apply_response.error.message}")
        else:
            print(f"   配置已应用")

        # 3. 运行反检测测试
        print("\n3. 运行反检测测试...")

        anti_detection_script = """
        ({
            // 检测 1: navigator.webdriver
            webdriver: navigator.webdriver,

            // 检测 2: Chrome 对象
            hasChrome: typeof window.chrome !== 'undefined',

            // 检测 3: 权限 API
            hasPermissions: navigator.permissions ? true : false,

            // 检测 4: 插件枚举
            pluginsLength: navigator.plugins.length,

            // 检测 5: 自动化指示器
            automation: {
                selenium: window.document.$cdc_asdjflasutopfhvcZLmcfl_ || window.document.$chrome_asyncScriptInfo,
                webdriverAttribute: navigator.webdriver,
                hasAutomation: Object.keys(window).filter(key => key.includes('automation')).length > 0
            },

            // 检测 6: iframe 检测
            iframeDetection: (() => {
                try {
                    const iframe = document.createElement('iframe');
                    iframe.style.display = 'none';
                    document.body.appendChild(iframe);
                    const result = iframe.contentWindow.navigator.webdriver;
                    document.body.removeChild(iframe);
                    return result;
                } catch (e) {
                    return 'error';
                }
            })(),

            // 检测 7: toString 检测
            toStringDetection: {
                navigator: Object.prototype.toString.call(navigator),
                window: Object.prototype.toString.call(window)
            }
        })
        """

        client.page.Navigate(page_pb2.NavigateRequest(
            page_id=page_id,
            url="https://example.com"
        ))

        eval_response = client.page.Evaluate(page_pb2.EvaluateRequest(
            page_id=page_id,
            expression=anti_detection_script
        ))

        if not eval_response.HasField('error'):
            result = json.loads(eval_response.result.string_value)

            print(f"   测试结果:")
            print(f"   - navigator.webdriver: {result['webdriver']}")
            print(f"   - Chrome 对象: {result['hasChrome']}")
            print(f"   - 权限 API: {result['hasPermissions']}")
            print(f"   - 插件数量: {result['pluginsLength']}")
            print(f"   - Selenium 检测: {result['automation']['selenium']}")
            print(f"   - 自动化指示器: {result['automation']['hasAutomation']}")
            print(f"   - iframe 检测: {result['iframeDetection']}")

            # 评估反检测效果
            print(f"\n   反检测效果评估:")
            score = 0
            total = 5

            if result['webdriver'] == False:
                print(f"   ✓ navigator.webdriver 已隐藏")
                score += 1
            else:
                print(f"   ✗ navigator.webdriver 未隐藏")

            if result['automation']['selenium'] == None:
                print(f"   ✓ Selenium 检测绕过")
                score += 1
            else:
                print(f"   ✗ Selenium 检测未绕过")

            if result['automation']['hasAutomation'] == False:
                print(f"   ✓ 自动化指示器已隐藏")
                score += 1
            else:
                print(f"   ✗ 自动化指示器未隐藏")

            if result['hasChrome'] == True:
                print(f"   ✓ Chrome 对象存在")
                score += 1
            else:
                print(f"   ✗ Chrome 对象缺失")

            if result['iframeDetection'] == False or result['iframeDetection'] == 'error':
                print(f"   ✓ iframe 检测绕过")
                score += 1
            else:
                print(f"   ✗ iframe 检测未绕过")

            print(f"\n   总体评分: {score}/{total}")
            if score == total:
                print(f"   优秀! 所有检测均已绕过")
            elif score >= total * 0.8:
                print(f"   良好! 大部分检测已绕过")
            elif score >= total * 0.6:
                print(f"   一般, 部分检测未绕过")
            else:
                print(f"   需要改进反检测策略")

        # 清理
        client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
        client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


def example_profile_presets():
    """预定义配置示例"""
    print("\n" + "=" * 60)
    print("预定义配置示例")
    print("=" * 60)

    client = ChaserOxideClient()

    try:
        # 1. 获取所有预定义配置
        print("\n1. 获取所有预定义配置...")

        get_presets_request = profile_pb2.GetPresetsRequest()
        get_presets_response = client.profile.GetPresets(get_presets_request)

        if get_presets_response.HasField('error'):
            print(f"   获取失败: {get_presets_response.error.message}")
            return

        presets = get_presets_response.presets.presets
        print(f"   找到 {len(presets)} 个预定义配置")

        # 2. 显示所有预定义配置
        for preset in presets:
            print(f"\n   配置类型: {profile_pb2.ProfileType.Name(preset.type)}")
            print(f"   配置 ID: {preset.profile_id}")
            print(f"   User-Agent: {preset.fingerprint.headers.user_agent[:60]}...")
            print(f"   平台: {preset.fingerprint.navigator.platform}")
            print(f"   屏幕分辨率: {preset.fingerprint.screen.width}x{preset.fingerprint.screen.height}")
            print(f"   CPU 核心: {preset.fingerprint.hardware.cpu_cores}")
            print(f"   设备内存: {preset.fingerprint.hardware.device_memory}GB")
            print(f"   时区: {preset.timezone}")
            print(f"   语言: {', '.join(preset.languages[:2])}")

        # 3. 按类型获取配置
        print("\n2. 按类型获取配置...")

        for profile_type in [profile_pb2.PROFILE_TYPE_WINDOWS, profile_pb2.PROFILE_TYPE_ANDROID]:
            print(f"\n   获取 {profile_pb2.ProfileType.Name(profile_type)} 配置...")

            get_type_request = profile_pb2.GetPresetsRequest(type=profile_type)
            get_type_response = client.profile.GetPresets(get_type_request)

            if get_type_response.HasField('error'):
                print(f"   获取失败: {get_type_response.error.message}")
                continue

            type_presets = get_type_response.presets.presets
            print(f"   找到 {len(type_presets)} 个 {profile_pb2.ProfileType.Name(profile_type)} 配置")

            if type_presets:
                preset = type_presets[0]
                print(f"   示例配置: {preset.profile_id}")
                print(f"   User-Agent: {preset.fingerprint.headers.user_agent[:50]}...")

        # 4. 使用预定义配置
        print("\n3. 使用预定义配置...")

        if presets:
            # 使用第一个 Windows 配置
            windows_presets = [p for p in presets if p.type == profile_pb2.PROFILE_TYPE_WINDOWS]

            if windows_presets:
                profile = windows_presets[0]

                launch_response = client.browser.Launch(browser_pb2.LaunchRequest(
                    options=common_pb2.BrowserOptions(
                        headless=True,
                        user_agent=profile.fingerprint.headers.user_agent
                    )
                ))
                browser_id = launch_response.browser_info.browser_id

                page_response = client.page.CreatePage(page_pb2.CreatePageRequest(
                    browser_id=browser_id
                ))
                page_id = page_response.page_info.page_id

                apply_response = client.profile.ApplyProfile(profile_pb2.ApplyProfileRequest(
                    page_id=page_id,
                    profile_id=profile.profile_id
                ))

                if not apply_response.HasField('error'):
                    print(f"   已应用 {profile_pb2.ProfileType.Name(profile.type)} 配置")

                # 获取当前活动配置
                get_active_request = profile_pb2.GetActiveProfileRequest(page_id=page_id)
                get_active_response = client.profile.GetActiveProfile(get_active_request)

                if not get_active_response.HasField('error'):
                    active_profile = get_active_response.profile
                    print(f"   当前活动配置: {active_profile.profile_id}")
                    print(f"   配置类型: {profile_pb2.ProfileType.Name(active_profile.type)}")

                # 清理
                client.page.ClosePage(page_pb2.ClosePageRequest(page_id=page_id))
                client.browser.Close(browser_pb2.CloseRequest(browser_id=browser_id))

    except grpc.RpcError as e:
        print(f"\nRPC 错误: {e.code()} - {e.details()}")
    finally:
        client.close()


if __name__ == "__main__":
    # 运行所有隐身功能示例
    example_custom_profile()
    example_human_behavior()
    example_randomized_profiles()
    example_anti_detection()
    example_profile_presets()

    print("\n" + "=" * 60)
    print("所有隐身功能示例执行完成！")
    print("=" * 60)
