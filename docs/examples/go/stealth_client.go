// Chaser-Oxide Go 客户端 - 隐身功能示例
//
// 演示高级隐身功能，包括指纹配置、人类行为模拟和反检测技术。
//
// 依赖安装:
//   go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
//   go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
//
// 生成代码:
//   protoc -I=./protos --go_out=. --go-grpc_out=. \
//     protos/common.proto \
//     protos/browser.proto \
//     protos/page.proto \
//     protos/element.proto \
//     protos/profile.proto \
//     protos/event.proto
//
// 运行示例:
//   go run stealth_client.go

package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	// 从生成的 proto 文件导入
	// 注意：需要先运行 protoc 生成 Go 代码
	// 请将 your-module-path 替换为您的实际模块路径
	commonpb "your-module-path/protos"
	browserpb "your-module-path/protos"
	pagepb "your-module-path/protos"
	elementpb "your-module-path/protos"
	profilepb "your-module-path/protos"
	eventpb "your-module-path/protos"

	browsergrpc "your-module-path/protos"
	pagegrpc "your-module-path/protos"
	elementgrpc "your-module-path/protos"
	profilegrpc "your-module-path/protos"
)

// ChaserOxideClient Chaser-Oxide gRPC 客户端封装
type ChaserOxideClient struct {
	conn    *grpc.ClientConn
	Browser browsergrpc.BrowserServiceClient
	Page    pagegrpc.PageServiceClient
	Element elementgrpc.ElementServiceClient
	Profile profilegrpc.ProfileServiceClient
}

// NewChaserOxideClient 创建新的客户端连接
func NewChaserOxideClient(host string) (*ChaserOxideClient, error) {
	conn, err := grpc.NewClient(host, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, fmt.Errorf("连接失败: %w", err)
	}

	return &ChaserOxideClient{
		conn:    conn,
		Browser: browsergrpc.NewBrowserServiceClient(conn),
		Page:    pagegrpc.NewPageServiceClient(conn),
		Element: elementgrpc.NewElementServiceClient(conn),
		Profile: profilegrpc.NewProfileServiceClient(conn),
	}, nil
}

// Close 关闭客户端连接
func (c *ChaserOxideClient) Close() error {
	return c.conn.Close()
}

// exampleCustomProfile 自定义指纹配置示例
func exampleCustomProfile() {
	fmt.Println("============================================================")
	fmt.Println("自定义指纹配置示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 创建自定义配置
	fmt.Println("\n1. 创建自定义指纹配置...")
	customReq := &profilepb.CreateCustomProfileRequest{
		ProfileName: "my_stealth_profile",
		Template:    profilepb.ProfileType_PROFILE_TYPE_WINDOWS,
		Options: &profilepb.CustomProfileOptions{
			UserAgent:        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
			Platform:         "Win32",
			ScreenWidth:      1920,
			ScreenHeight:     1080,
			DevicePixelRatio: 1.0,
			CpuCores:         8,
			DeviceMemory:     16,
			Locale:           "zh-CN",
			Languages:        []string{"zh-CN", "zh", "en-US", "en"},
			Timezone:         "Asia/Shanghai",
			WebglVendor:      "Intel Inc.",
			WebglRenderer:    "Intel(R) UHD Graphics 630",
			NavigatorVendor:  "Google Inc.",
			NavigatorProduct: "Gecko",
		},
		ProfileOptions: &profilepb.ProfileOptions{
			InjectNavigator:       true,
			InjectScreen:          true,
			InjectWebgl:           true,
			InjectCanvas:          true,
			InjectAudio:           true,
			NeutralizeUtilityWorld: true,
			UseIsolatedWorld:      true,
			RandomizeMetrics:      true,
			PreventDetection:      true,
		},
	}

	customResp, err := client.Profile.CreateCustomProfile(ctx, customReq)
	if err != nil {
		log.Fatalf("创建配置失败: %v", err)
	}

	if customResp.Error != nil {
		log.Fatalf("创建配置失败: %s", customResp.Error.Message)
	}

	profile := customResp.Profile
	fmt.Printf("   配置已创建: %s\n", profile.ProfileId)
	fmt.Printf("   User-Agent: %s\n", profile.Fingerprint.Headers.UserAgent)
	fmt.Printf("   平台: %s\n", profile.Fingerprint.Navigator.Platform)
	fmt.Printf("   时区: %s\n", profile.Timezone)

	// 2. 启动浏览器并应用配置
	fmt.Println("\n2. 启动浏览器...")
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{
			Headless:   true,
			UserAgent: profile.Fingerprint.Headers.UserAgent,
		},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	fmt.Println("\n3. 创建页面...")
	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 3. 应用指纹配置
	fmt.Println("\n4. 应用指纹配置...")
	applyReq := &profilepb.ApplyProfileRequest{
		PageId:           pageID,
		ProfileId:        profile.ProfileId,
		OverrideExisting: true,
	}

	applyResp, err := client.Profile.ApplyProfile(ctx, applyReq)
	if err != nil {
		log.Printf("   应用配置失败: %v", err)
	} else if applyResp.Error == nil {
		fmt.Printf("   配置已应用\n")
		fmt.Printf("   应用的特性: %v\n", applyResp.Result.AppliedFeatures)
	}

	// 4. 验证指纹
	fmt.Println("\n5. 验证浏览器指纹...")
	_, _ = client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	// 执行指纹检测脚本
	detectionScript := `
	({
		userAgent: navigator.userAgent,
		platform: navigator.platform,
		vendor: navigator.vendor,
		language: navigator.language,
		languages: navigator.languages,
		hardwareConcurrency: navigator.hardwareConcurrency,
		deviceMemory: navigator.deviceMemory,
		screen: {
			width: screen.width,
			height: screen.height,
			availWidth: screen.availWidth,
			availHeight: screen.availHeight,
			colorDepth: screen.colorDepth,
			pixelDepth: screen.pixelDepth
		},
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
		canvas: (() => {
			const canvas = document.createElement('canvas');
			const ctx = canvas.getContext('2d');
			ctx.textBaseline = 'top';
			ctx.font = '14px Arial';
			ctx.fillText('Hello, World!', 2, 2);
			return canvas.toDataURL().substring(0, 50) + '...';
		})(),
		automation: {
			webdriver: navigator.webdriver,
			chrome: window.chrome ? {
				runtime: window.chrome.runtime ? true : false
			} : null,
			permissions: navigator.permissions ? true : false
		}
	})
	`

	evalResp, err := client.Page.Evaluate(ctx, &pagepb.EvaluateRequest{
		PageId:     pageID,
		Expression: detectionScript,
	})

	if err == nil && evalResp.Error == nil {
		fmt.Printf("   指纹检测结果:\n")

		var result map[string]interface{}
		if err := json.Unmarshal([]byte(evalResp.Result.StringValue), &result); err == nil {
			fmt.Printf("   User-Agent: %v\n", result["userAgent"])
			fmt.Printf("   平台: %v\n", result["platform"])
			fmt.Printf("   语言: %v\n", result["language"])
			fmt.Printf("   CPU 核心数: %v\n", result["hardwareConcurrency"])
			fmt.Printf("   设备内存: %vGB\n", result["deviceMemory"])

			if screen, ok := result["screen"].(map[string]interface{}); ok {
				fmt.Printf("   屏幕分辨率: %vx%v\n", screen["width"], screen["height"])
			}

			if webgl, ok := result["webgl"].(map[string]interface{}); ok {
				fmt.Printf("   WebGL Vendor: %v\n", webgl["vendor"])
				fmt.Printf("   WebGL Renderer: %v\n", webgl["renderer"])
			}

			if automation, ok := result["automation"].(map[string]interface{}); ok {
				fmt.Printf("   Webdriver: %v\n", automation["webdriver"])

				// 评估反检测效果
				fmt.Printf("\n   反检测评估:\n")
				score := 0
				if automation["webdriver"] == false {
					fmt.Printf("   ✓ Webdriver 属性已隐藏\n")
					score++
				}
				if result["hardwareConcurrency"].(float64) >= 4 {
					fmt.Printf("   ✓ CPU 核心数看起来真实\n")
					score++
				}
				if result["deviceMemory"].(float64) >= 8 {
					fmt.Printf("   ✓ 设备内存看起来真实\n")
					score++
				}
				fmt.Printf("   总体评分: %d/3\n", score)
			}
		}
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleHumanBehavior 人类行为模拟示例
func exampleHumanBehavior() {
	fmt.Println("\n============================================================")
	fmt.Println("人类行为模拟示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 启动浏览器
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 导航到测试页面
	fmt.Println("\n1. 导航到测试页面...")
	_, _ = client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	// 查找元素
	fmt.Println("\n2. 查找元素...")
	findResp, err := client.Element.FindElement(ctx, &elementpb.FindElementRequest{
		PageId:       pageID,
		SelectorType: commonpb.SelectorType_SELECTOR_TYPE_CSS,
		Selector:     "h1",
	})

	if err != nil || findResp.Error != nil {
		log.Printf("   查找失败: %v", err)
		return
	}

	element := findResp.Element
	fmt.Printf("   找到元素: %s\n", element.ElementId)

	// 人类式移动和点击
	fmt.Println("\n3. 模拟人类鼠标移动和点击...")
	fmt.Println("   使用贝塞尔曲线轨迹，速度变化，随机偏移")

	hoverResp, err := client.Element.Hover(ctx, &elementpb.HoverRequest{
		Element:          element,
		HumanLike:        true,
		MovementDuration: 500,
		BezierCurve:      true,
	})

	if err == nil && hoverResp.Error == nil {
		fmt.Printf("   鼠标悬停成功\n")
	}

	time.Sleep(300 * time.Millisecond)

	clickResp, err := client.Element.Click(ctx, &elementpb.ClickRequest{
		Element:          element,
		HumanLike:        true,
		MovementDuration: 300,
		RandomOffset:     true,
		BezierCurve:      true,
	})

	if err == nil && clickResp.Error == nil {
		fmt.Printf("   点击成功\n")
	}

	// 人类式输入
	fmt.Println("\n4. 模拟人类输入...")
	fmt.Println("   包含打字速度变化，随机停顿，偶尔回删")

	typeReq := &elementpb.TypeRequest{
		Element:             element,
		Text:                "Hello, World!",
		HumanLike:           true,
		TypingSpeed:         150,
		RandomDelays:        true,
		OccasionalBackspace: true,
	}

	fmt.Printf("   文本: %s\n", typeReq.Text)
	fmt.Printf("   打字速度: %dms/字符\n", typeReq.TypingSpeed)
	fmt.Printf("   随机停顿: %v\n", typeReq.RandomDelays)
	fmt.Printf("   偶尔回删: %v\n", typeReq.OccasionalBackspace)

	// 拖拽示例
	fmt.Println("\n5. 模拟人类拖拽...")
	fmt.Println("   使用贝塞尔曲线轨迹，速度变化")

	dragReq := &elementpb.DragAndDropRequest{
		SourceElement:     element,
		TargetElement:     element,
		HumanLike:         true,
		MovementDuration:  800,
		BezierCurve:       true,
		RandomPath:        true,
	}

	fmt.Printf("   拖拽持续时间: %dms\n", dragReq.MovementDuration)
	fmt.Printf("   贝塞尔曲线: %v\n", dragReq.BezierCurve)
	fmt.Printf("   随机路径: %v\n", dragReq.RandomPath)

	fmt.Println("\n   人类行为模拟完成!")
	fmt.Println("   提示: 所有交互都包含:")
	fmt.Println("   - 贝塞尔曲线鼠标轨迹")
	fmt.Println("   - 速度变化和随机停顿")
	fmt.Println("   - 模拟人类反应时间")
	fmt.Println("   - 随机偏移和抖动")

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleRandomizedProfiles 随机化指纹配置示例
func exampleRandomizedProfiles() {
	fmt.Println("\n============================================================")
	fmt.Println("随机化指纹配置示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 创建多个随机化配置
	fmt.Println("\n1. 创建多个随机化 Windows 配置...")

	var profiles []*profilepb.Profile
	for i := 0; i < 3; i++ {
		fmt.Printf("\n   生成配置 #%d...\n", i+1)

		randomizeReq := &profilepb.RandomizeProfileRequest{
			Type: profilepb.ProfileType_PROFILE_TYPE_WINDOWS,
			Options: &profilepb.RandomizationOptions{
				RandomizeScreen:   true,
				RandomizeTimezone: true,
				RandomizeLanguage: true,
				RandomizeWebgl:    true,
				Entropy:           0.8,
			},
		}

		randomizeResp, err := client.Profile.RandomizeProfile(ctx, randomizeReq)
		if err != nil || randomizeResp.Error != nil {
			log.Printf("   生成失败: %v", err)
			continue
		}

		profile := randomizeResp.Profile
		profiles = append(profiles, profile)

		fmt.Printf("   配置 ID: %s\n", profile.ProfileId)
		fmt.Printf("   User-Agent: %s...\n", profile.Fingerprint.Headers.UserAgent[:50])
		fmt.Printf("   屏幕分辨率: %dx%d\n", profile.Fingerprint.Screen.Width, profile.Fingerprint.Screen.Height)
		fmt.Printf("   时区: %s\n", profile.Timezone)
		fmt.Printf("   语言: %s\n", fmt.Sprintf("%v", profile.Languages[:2]))
	}

	fmt.Printf("\n   成功生成 %d 个随机配置\n", len(profiles))

	// 比较配置差异
	fmt.Println("\n2. 比较配置差异...")
	if len(profiles) >= 2 {
		p1, p2 := profiles[0], profiles[1]

		fmt.Printf("   配置 1 vs 配置 2:\n")
		fmt.Printf("   屏幕分辨率: %dx%d vs %dx%d\n",
			p1.Fingerprint.Screen.Width, p1.Fingerprint.Screen.Height,
			p2.Fingerprint.Screen.Width, p2.Fingerprint.Screen.Height)
		fmt.Printf("   时区: %s vs %s\n", p1.Timezone, p2.Timezone)
		fmt.Printf("   CPU 核心: %d vs %d\n",
			p1.Fingerprint.Hardware.CpuCores,
			p2.Fingerprint.Hardware.CpuCores)
		fmt.Printf("   设备内存: %dGB vs %dGB\n",
			p1.Fingerprint.Hardware.DeviceMemory,
			p2.Fingerprint.Hardware.DeviceMemory)

		if p1.Fingerprint.Screen.Width != p2.Fingerprint.Screen.Width {
			fmt.Printf("   ✓ 屏幕分辨率不同\n")
		}
		if p1.Timezone != p2.Timezone {
			fmt.Printf("   ✓ 时区不同\n")
		}
		if p1.Fingerprint.Hardware.CpuCores != p2.Fingerprint.Hardware.CpuCores {
			fmt.Printf("   ✓ CPU 核心数不同\n")
		}
	}

	// 使用随机化配置
	fmt.Println("\n3. 使用随机化配置...")
	if len(profiles) > 0 {
		profile := profiles[0]

		launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
			Options: &commonpb.BrowserOptions{
				Headless:   true,
				UserAgent: profile.Fingerprint.Headers.UserAgent,
			},
		})
		browserID := launchResp.BrowserInfo.BrowserId

		pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
			BrowserId: browserID,
		})
		pageID := pageResp.PageInfo.PageId

		applyResp, err := client.Profile.ApplyProfile(ctx, &profilepb.ApplyProfileRequest{
			PageId:    pageID,
			ProfileId: profile.ProfileId,
		})

		if err == nil && applyResp.Error == nil {
			fmt.Printf("   配置已应用到页面\n")
		}

		// 清理
		_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
		_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
	}
}

// exampleAntiDetection 反检测技术示例
func exampleAntiDetection() {
	fmt.Println("\n============================================================")
	fmt.Println("反检测技术示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 创建配置并启用所有反检测选项
	fmt.Println("\n1. 创建配置并启用所有反检测选项...")

	customReq := &profilepb.CreateCustomProfileRequest{
		ProfileName: "anti_detection_profile",
		Template:    profilepb.ProfileType_PROFILE_TYPE_WINDOWS,
		Options: &profilepb.CustomProfileOptions{
			UserAgent:    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
			Platform:     "Win32",
			CpuCores:     8,
			DeviceMemory: 16,
		},
		ProfileOptions: &profilepb.ProfileOptions{
			InjectNavigator:       true,
			InjectScreen:          true,
			InjectWebgl:           true,
			InjectCanvas:          true,
			InjectAudio:           true,
			NeutralizeUtilityWorld: true,
			UseIsolatedWorld:      true,
			RandomizeMetrics:      true,
			PreventDetection:      true,
		},
	}

	profileResp, err := client.Profile.CreateCustomProfile(ctx, customReq)
	if err != nil || profileResp.Error != nil {
		log.Fatalf("创建配置失败: %v", err)
	}

	profile := profileResp.Profile

	fmt.Printf("   配置已创建: %s\n", profile.ProfileId)
	fmt.Printf("   启用的反检测选项:\n")
	fmt.Printf("   - Navigator 注入: %v\n", profile.Options.InjectNavigator)
	fmt.Printf("   - Screen 注入: %v\n", profile.Options.InjectScreen)
	fmt.Printf("   - WebGL 注入: %v\n", profile.Options.InjectWebgl)
	fmt.Printf("   - Canvas 保护: %v\n", profile.Options.InjectCanvas)
	fmt.Printf("   - 音频保护: %v\n", profile.Options.InjectAudio)
	fmt.Printf("   - Utility World 中立化: %v\n", profile.Options.NeutralizeUtilityWorld)
	fmt.Printf("   - 隔离世界: %v\n", profile.Options.UseIsolatedWorld)
	fmt.Printf("   - 指标随机化: %v\n", profile.Options.RandomizeMetrics)
	fmt.Printf("   - 防检测: %v\n", profile.Options.PreventDetection)

	// 2. 启动浏览器并应用配置
	fmt.Println("\n2. 启动浏览器并应用配置...")

	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	applyResp, err := client.Profile.ApplyProfile(ctx, &profilepb.ApplyProfileRequest{
		PageId:           pageID,
		ProfileId:        profile.ProfileId,
		OverrideExisting: true,
	})

	if err != nil || applyResp.Error != nil {
		log.Printf("   应用配置失败: %v", err)
	} else {
		fmt.Printf("   配置已应用\n")
	}

	// 3. 运行反检测测试
	fmt.Println("\n3. 运行反检测测试...")

	antiDetectionScript := `
	({
		webdriver: navigator.webdriver,
		hasChrome: typeof window.chrome !== 'undefined',
		hasPermissions: navigator.permissions ? true : false,
		pluginsLength: navigator.plugins.length,
		automation: {
			selenium: window.document.$cdc_asdjflasutopfhvcZLmcfl_ || window.document.$chrome_asyncScriptInfo,
			webdriverAttribute: navigator.webdriver,
			hasAutomation: Object.keys(window).filter(key => key.includes('automation')).length > 0
		},
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
		})()
	})
	`

	_, _ = client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	evalResp, err := client.Page.Evaluate(ctx, &pagepb.EvaluateRequest{
		PageId:     pageID,
		Expression: antiDetectionScript,
	})

	if err == nil && evalResp.Error == nil {
		var result map[string]interface{}
		if err := json.Unmarshal([]byte(evalResp.Result.StringValue), &result); err == nil {
			fmt.Printf("   测试结果:\n")
			fmt.Printf("   - navigator.webdriver: %v\n", result["webdriver"])
			fmt.Printf("   - Chrome 对象: %v\n", result["hasChrome"])
			fmt.Printf("   - 权限 API: %v\n", result["hasPermissions"])
			fmt.Printf("   - 插件数量: %v\n", result["pluginsLength"])

			if automation, ok := result["automation"].(map[string]interface{}); ok {
				fmt.Printf("   - Selenium 检测: %v\n", automation["selenium"])
				fmt.Printf("   - 自动化指示器: %v\n", automation["hasAutomation"])
			}

			fmt.Printf("   - iframe 检测: %v\n", result["iframeDetection"])

			// 评估反检测效果
			fmt.Printf("\n   反检测效果评估:\n")
			score := 0
			total := 5

			if result["webdriver"] == false {
				fmt.Printf("   ✓ navigator.webdriver 已隐藏\n")
				score++
			} else {
				fmt.Printf("   ✗ navigator.webdriver 未隐藏\n")
			}

			if automation, ok := result["automation"].(map[string]interface{}); ok {
				if automation["selenium"] == nil {
					fmt.Printf("   ✓ Selenium 检测绕过\n")
					score++
				} else {
					fmt.Printf("   ✗ Selenium 检测未绕过\n")
				}

				if automation["hasAutomation"] == false {
					fmt.Printf("   ✓ 自动化指示器已隐藏\n")
					score++
				} else {
					fmt.Printf("   ✗ 自动化指示器未隐藏\n")
				}
			}

			if result["hasChrome"] == true {
				fmt.Printf("   ✓ Chrome 对象存在\n")
				score++
			} else {
				fmt.Printf("   ✗ Chrome 对象缺失\n")
			}

			if result["iframeDetection"] == false || result["iframeDetection"] == "error" {
				fmt.Printf("   ✓ iframe 检测绕过\n")
				score++
			} else {
				fmt.Printf("   ✗ iframe 检测未绕过\n")
			}

			fmt.Printf("\n   总体评分: %d/%d\n", score, total)
			if score == total {
				fmt.Printf("   优秀! 所有检测均已绕过\n")
			} else if float64(score) >= float64(total)*0.8 {
				fmt.Printf("   良好! 大部分检测已绕过\n")
			} else if float64(score) >= float64(total)*0.6 {
				fmt.Printf("   一般, 部分检测未绕过\n")
			} else {
				fmt.Printf("   需要改进反检测策略\n")
			}
		}
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleProfilePresets 预定义配置示例
func exampleProfilePresets() {
	fmt.Println("\n============================================================")
	fmt.Println("预定义配置示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 获取所有预定义配置
	fmt.Println("\n1. 获取所有预定义配置...")

	getPresetsReq := &profilepb.GetPresetsRequest{}
	getPresetsResp, err := client.Profile.GetPresets(ctx, getPresetsReq)

	if err != nil || getPresetsResp.Error != nil {
		log.Fatalf("获取失败: %v", err)
	}

	presets := getPresetsResp.Presets.Presets
	fmt.Printf("   找到 %d 个预定义配置\n", len(presets))

	// 2. 显示所有预定义配置
	for _, preset := range presets {
		fmt.Printf("\n   配置类型: %s\n", profilepb.ProfileType_name[int32(preset.Type)])
		fmt.Printf("   配置 ID: %s\n", preset.ProfileId)
		fmt.Printf("   User-Agent: %s...\n", preset.Fingerprint.Headers.UserAgent[:60])
		fmt.Printf("   平台: %s\n", preset.Fingerprint.Navigator.Platform)
		fmt.Printf("   屏幕分辨率: %dx%d\n",
			preset.Fingerprint.Screen.Width,
			preset.Fingerprint.Screen.Height)
		fmt.Printf("   CPU 核心: %d\n", preset.Fingerprint.Hardware.CpuCores)
		fmt.Printf("   设备内存: %dGB\n", preset.Fingerprint.Hardware.DeviceMemory)
		fmt.Printf("   时区: %s\n", preset.Timezone)
		fmt.Printf("   语言: %v\n", preset.Languages[:2])
	}

	// 3. 按类型获取配置
	fmt.Println("\n2. 按类型获取配置...")

	for _, profileType := range []profilepb.ProfileType{
		profilepb.ProfileType_PROFILE_TYPE_WINDOWS,
		profilepb.ProfileType_PROFILE_TYPE_ANDROID,
	} {
		fmt.Printf("\n   获取 %s 配置...\n", profilepb.ProfileType_name[int32(profileType)])

		getTypeReq := &profilepb.GetPresetsRequest{Type: profileType}
		getTypeResp, err := client.Profile.GetPresets(ctx, getTypeReq)

		if err != nil || getTypeResp.Error != nil {
			log.Printf("   获取失败: %v", err)
			continue
		}

		typePresets := getTypeResp.Presets.Presets
		fmt.Printf("   找到 %d 个 %s 配置\n",
			len(typePresets),
			profilepb.ProfileType_name[int32(profileType)])

		if len(typePresets) > 0 {
			preset := typePresets[0]
			fmt.Printf("   示例配置: %s\n", preset.ProfileId)
			fmt.Printf("   User-Agent: %s...\n", preset.Fingerprint.Headers.UserAgent[:50])
		}
	}

	// 4. 使用预定义配置
	fmt.Println("\n3. 使用预定义配置...")

	if len(presets) > 0 {
		// 使用第一个 Windows 配置
		var windowsPreset *profilepb.Profile
		for _, preset := range presets {
			if preset.Type == profilepb.ProfileType_PROFILE_TYPE_WINDOWS {
				windowsPreset = preset
				break
			}
		}

		if windowsPreset != nil {
			launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
				Options: &commonpb.BrowserOptions{
					Headless:   true,
					UserAgent: windowsPreset.Fingerprint.Headers.UserAgent,
				},
			})
			browserID := launchResp.BrowserInfo.BrowserId

			pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
				BrowserId: browserID,
			})
			pageID := pageResp.PageInfo.PageId

			applyResp, err := client.Profile.ApplyProfile(ctx, &profilepb.ApplyProfileRequest{
				PageId:    pageID,
				ProfileId: windowsPreset.ProfileId,
			})

			if err == nil && applyResp.Error == nil {
				fmt.Printf("   已应用 %s 配置\n",
					profilepb.ProfileType_name[int32(windowsPreset.Type)])
			}

			// 获取当前活动配置
			getActiveReq := &profilepb.GetActiveProfileRequest{PageId: pageID}
			getActiveResp, err := client.Profile.GetActiveProfile(ctx, getActiveReq)

			if err == nil && getActiveResp.Error == nil {
				activeProfile := getActiveResp.Profile
				fmt.Printf("   当前活动配置: %s\n", activeProfile.ProfileId)
				fmt.Printf("   配置类型: %s\n",
					profilepb.ProfileType_name[int32(activeProfile.Type)])
			}

			// 清理
			_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
			_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
		}
	}
}

func main() {
	fmt.Println("Chaser-Oxide Go 隐身功能示例")
	fmt.Println("====================================\n")

	// 运行所有隐身功能示例
	exampleCustomProfile()
	exampleHumanBehavior()
	exampleRandomizedProfiles()
	exampleAntiDetection()
	exampleProfilePresets()

	fmt.Println("\n============================================================")
	fmt.Println("所有隐身功能示例执行完成!")
	fmt.Println("============================================================")
}
