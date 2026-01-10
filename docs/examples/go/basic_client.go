// Chaser-Oxide Go 客户端 - 基础示例
//
// 演示如何使用 Go 客户端进行基本的浏览器自动化操作。
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
//   go run basic_client.go

package main

import (
	"context"
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
	eventgrpc "your-module-path/protos"
)

// ChaserOxideClient Chaser-Oxide gRPC 客户端封装
type ChaserOxideClient struct {
	conn    *grpc.ClientConn
	Browser browsergrpc.BrowserServiceClient
	Page    pagegrpc.PageServiceClient
	Element elementgrpc.ElementServiceClient
	Profile profilegrpc.ProfileServiceClient
	Events  eventgrpc.EventServiceClient
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
		Events:  eventgrpc.NewEventServiceClient(conn),
	}, nil
}

// Close 关闭客户端连接
func (c *ChaserOxideClient) Close() error {
	return c.conn.Close()
}

// exampleBasicNavigation 基础导航示例
func exampleBasicNavigation() {
	fmt.Println("============================================================")
	fmt.Println("基础导航示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 启动浏览器
	fmt.Println("\n1. 启动浏览器...")
	launchReq := &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{
			Headless:   true,
			WindowWidth:  1920,
			WindowHeight: 1080,
			UserAgent:   "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
		},
	}

	launchResp, err := client.Browser.Launch(ctx, launchReq)
	if err != nil {
		log.Fatalf("启动失败: %v", err)
	}

	if launchResp.Error != nil {
		log.Fatalf("启动失败: %s", launchResp.Error.Message)
	}

	browserID := launchResp.BrowserInfo.BrowserId
	fmt.Printf("   浏览器已启动: %s\n", browserID)

	// 2. 创建新页面
	fmt.Println("\n2. 创建新页面...")
	createPageReq := &pagepb.CreatePageRequest{
		BrowserId: browserID,
		Url:       "about:blank",
	}

	createPageResp, err := client.Page.CreatePage(ctx, createPageReq)
	if err != nil {
		log.Fatalf("创建页面失败: %v", err)
	}

	if createPageResp.Error != nil {
		log.Fatalf("创建页面失败: %s", createPageResp.Error.Message)
	}

	pageID := createPageResp.PageInfo.PageId
	fmt.Printf("   页面已创建: %s\n", pageID)

	// 3. 导航到 URL
	fmt.Println("\n3. 导航到 example.com...")
	navigateReq := &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
		Options: &commonpb.NavigationOptions{
			Timeout:  30000,
			WaitUntil: &commonpb.NavigationOptions_LoadState{
				LoadState: commonpb.NavigationOptions_LOAD_STATE_NETWORK_IDLE,
			},
		},
	}

	navigateResp, err := client.Page.Navigate(ctx, navigateReq)
	if err != nil {
		log.Fatalf("导航失败: %v", err)
	}

	if navigateResp.Error != nil {
		log.Fatalf("导航失败: %s", navigateResp.Error.Message)
	}

	fmt.Printf("   导航成功: %s\n", navigateResp.Result.Url)
	fmt.Printf("   状态码: %d\n", navigateResp.Result.StatusCode)

	// 4. 获取页面标题
	fmt.Println("\n4. 获取页面内容...")
	snapshotReq := &pagepb.GetSnapshotRequest{
		PageId: pageID,
	}

	snapshotResp, err := client.Page.GetSnapshot(ctx, snapshotReq)
	if err != nil {
		log.Printf("   获取快照失败: %v", err)
	} else if snapshotResp.Error == nil {
		fmt.Printf("   页面标题: %s\n", snapshotResp.Snapshot.Title)
	}

	// 5. 截图
	fmt.Println("\n5. 截取页面截图...")
	screenshotReq := &pagepb.ScreenshotRequest{
		PageId: pageID,
		Options: &commonpb.ScreenshotOptions{
			Format: &commonpb.ScreenshotOptions_Format{
				Format: commonpb.ScreenshotOptions_FORMAT_PNG,
			},
			FullPage: true,
		},
	}

	screenshotResp, err := client.Page.Screenshot(ctx, screenshotReq)
	if err != nil {
		log.Printf("   截图失败: %v", err)
	} else if screenshotResp.Error == nil {
		fmt.Printf("   截图成功: %d bytes\n", len(screenshotResp.Result.Data))
		// 在实际应用中保存截图
		// err = os.WriteFile("screenshot.png", screenshotResp.Result.Data, 0644)
		// if err != nil {
		//     log.Printf("   保存截图失败: %v", err)
		// }
		// fmt.Printf("   已保存到: screenshot.png\n")
	}

	// 6. 执行 JavaScript
	fmt.Println("\n6. 执行 JavaScript...")
	evaluateReq := &pagepb.EvaluateRequest{
		PageId:       pageID,
		Expression:   "document.title",
		AwaitPromise: true,
	}

	evaluateResp, err := client.Page.Evaluate(ctx, evaluateReq)
	if err != nil {
		log.Printf("   执行失败: %v", err)
	} else if evaluateResp.Error == nil {
		fmt.Printf("   执行结果: %s\n", evaluateResp.Result.StringValue)
	}

	// 7. 清理资源
	fmt.Println("\n7. 清理资源...")
	closePageReq := &pagepb.ClosePageRequest{
		PageId: pageID,
	}
	_, _ = client.Page.ClosePage(ctx, closePageReq)

	closeBrowserReq := &browserpb.CloseRequest{
		BrowserId: browserID,
	}
	_, _ = client.Browser.Close(ctx, closeBrowserReq)

	fmt.Println("   资源已清理")
}

// exampleElementInteraction 元素交互示例
func exampleElementInteraction() {
	fmt.Println("\n============================================================")
	fmt.Println("元素交互示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 启动浏览器和页面
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 导航到测试页面
	_, _ = client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	// 查找元素
	fmt.Println("\n1. 查找 h1 元素...")
	findReq := &elementpb.FindElementRequest{
		PageId:       pageID,
		SelectorType: commonpb.SelectorType_SELECTOR_TYPE_CSS,
		Selector:     "h1",
	}

	findResp, err := client.Element.FindElement(ctx, findReq)
	if err != nil {
		log.Printf("   查找失败: %v", err)
		return
	}

	if findResp.Error != nil {
		log.Printf("   查找失败: %s", findResp.Error.Message)
		return
	}

	element := findResp.Element
	fmt.Printf("   找到元素: %s\n", element.ElementId)

	// 获取元素文本
	fmt.Println("\n2. 获取元素文本...")
	textReq := &elementpb.GetTextRequest{
		Element: element,
	}

	textResp, err := client.Element.GetText(ctx, textReq)
	if err == nil && textResp.Error == nil {
		fmt.Printf("   文本内容: %s\n", textResp.Text.Text)
	}

	// 获取元素属性
	fmt.Println("\n3. 获取元素属性...")
	attrReq := &elementpb.GetAttributeRequest{
		Element: element,
		Name:    "class",
	}

	attrResp, err := client.Element.GetAttribute(ctx, attrReq)
	if err == nil && attrResp.Error == nil {
		fmt.Printf("   class 属性: %s\n", attrResp.Value.Value)
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleEventSubscription 事件订阅示例
func exampleEventSubscription() {
	fmt.Println("\n============================================================")
	fmt.Println("事件订阅示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 启动浏览器和页面
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 创建事件流
	fmt.Println("\n1. 订阅页面事件...")

	stream, err := client.Events.Subscribe(ctx)
	if err != nil {
		log.Fatalf("创建事件流失败: %v", err)
	}

	// 发送订阅请求
	subscribeReq := &eventpb.SubscribeRequest{
		Action: eventpb.SubscribeRequest_ACTION_SUBSCRIBE,
		Subscription: &eventpb.Subscription{
			Target: &eventpb.Subscription_PageId{
				PageId: pageID,
			},
			EventTypes: []eventpb.EventType{
				eventpb.EventType_EVENT_TYPE_PAGE_LOADED,
				eventpb.EventType_EVENT_TYPE_CONSOLE_LOG,
				eventpb.EventType_EVENT_TYPE_RESPONSE_RECEIVED,
			},
		},
	}

	if err := stream.Send(subscribeReq); err != nil {
		log.Fatalf("发送订阅请求失败: %v", err)
	}

	// 接收事件
	fmt.Println("\n2. 接收事件...")
	eventCount := 0
	maxEvents := 5

	for eventCount < maxEvents {
		event, err := stream.Recv()
		if err != nil {
			break
		}

		eventCount++
		fmt.Printf("\n   收到事件 #%d:\n", eventCount)
		fmt.Printf("   类型: %s\n", eventpb.EventType_name[int32(event.Metadata.Type)])
		fmt.Printf("   时间戳: %d\n", event.Metadata.Timestamp)

		// 解析不同类型的事件
		switch event.Metadata.Type {
		case eventpb.EventType_EVENT_TYPE_PAGE_LOADED:
			if event.PageEvent != nil {
				fmt.Printf("   URL: %s\n", event.PageEvent.Url)
				fmt.Printf("   标题: %s\n", event.PageEvent.Title)
			}

		case eventpb.EventType_EVENT_TYPE_CONSOLE_LOG:
			if event.ConsoleEvent != nil {
				fmt.Printf("   日志级别: %s\n", eventpb.ConsoleEvent.LogLevel)
				fmt.Printf("   内容: %v\n", event.ConsoleEvent.Args)
			}

		case eventpb.EventType_EVENT_TYPE_RESPONSE_RECEIVED:
			if event.NetworkEvent != nil {
				fmt.Printf("   URL: %s\n", event.NetworkEvent.Url)
				fmt.Printf("   状态码: %d\n", event.NetworkEvent.StatusCode)
			}
		}
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleStealthBrowsing 隐身浏览示例
func exampleStealthBrowsing() {
	fmt.Println("\n============================================================")
	fmt.Println("隐身浏览示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 创建 Windows 指纹配置
	fmt.Println("\n1. 创建 Windows 指纹配置...")
	profileReq := &profilepb.CreateProfileRequest{
		Type: profilepb.ProfileType_PROFILE_TYPE_WINDOWS,
	}

	profileResp, err := client.Profile.CreateProfile(ctx, profileReq)
	if err != nil {
		log.Fatalf("创建配置失败: %v", err)
	}

	if profileResp.Error != nil {
		log.Fatalf("创建配置失败: %s", profileResp.Error.Message)
	}

	profileID := profileResp.Profile.ProfileId
	fmt.Printf("   配置已创建: %s\n", profileID)
	fmt.Printf("   User-Agent: %s\n", profileResp.Profile.Fingerprint.Headers.UserAgent)

	// 2. 启动浏览器
	fmt.Println("\n2. 启动浏览器...")
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{
			Headless:   true,
			UserAgent: profileResp.Profile.Fingerprint.Headers.UserAgent,
		},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	// 3. 创建页面
	fmt.Println("\n3. 创建页面...")
	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 4. 应用指纹配置
	fmt.Println("\n4. 应用指纹配置...")
	applyReq := &profilepb.ApplyProfileRequest{
		PageId:           pageID,
		ProfileId:        profileID,
		OverrideExisting: true,
	}

	applyResp, err := client.Profile.ApplyProfile(ctx, applyReq)
	if err != nil {
		log.Printf("   应用配置失败: %v", err)
	} else if applyResp.Error == nil {
		fmt.Printf("   配置已应用\n")
		fmt.Printf("   应用特性: %v\n", applyResp.Result.AppliedFeatures)
	}

	// 5. 访问测试页面并检查指纹
	fmt.Println("\n5. 检查浏览器指纹...")
	_, _ = client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	// 执行 JavaScript 检查指纹
	checkScript := `
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
	`

	evalResp, err := client.Page.Evaluate(ctx, &pagepb.EvaluateRequest{
		PageId:     pageID,
		Expression: checkScript,
	})

	if err == nil && evalResp.Error == nil {
		fmt.Printf("   检测结果:\n")
		fmt.Printf("   User-Agent: %s\n", evalResp.Result.StringValue)
		// 在实际应用中解析 JSON 并显示所有字段
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleBrowserInfo 浏览器信息示例
func exampleBrowserInfo() {
	fmt.Println("\n============================================================")
	fmt.Println("浏览器信息示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. 获取浏览器版本
	fmt.Println("\n1. 获取浏览器版本...")
	versionReq := &browserpb.GetVersionRequest{}
	versionResp, err := client.Browser.GetVersion(ctx, versionReq)
	if err == nil && versionResp.Error == nil {
		fmt.Printf("   用户代理: %s\n", versionResp.Version.UserAgent)
		fmt.Printf("   浏览器版本: %s\n", versionResp.Version.BrowserVersion)
		fmt.Printf("   协议版本: %s\n", versionResp.Version.ProtocolVersion)
	}

	// 2. 启动浏览器并获取状态
	fmt.Println("\n2. 启动浏览器并获取状态...")
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	statusReq := &browserpb.GetStatusRequest{
		BrowserId: browserID,
	}
	statusResp, err := client.Browser.GetStatus(ctx, statusReq)
	if err == nil && statusResp.Error == nil {
		fmt.Printf("   浏览器状态: %s\n", statusResp.Status.Status)
		fmt.Printf("   页面数量: %d\n", statusResp.Status.PageCount)
		fmt.Printf("   运行时间: %dms\n", statusResp.Status.Uptime)
	}

	// 3. 获取所有页面列表
	fmt.Println("\n3. 获取所有页面列表...")
	pagesReq := &browserpb.GetPagesRequest{
		BrowserId: browserID,
	}
	pagesResp, err := client.Browser.GetPages(ctx, pagesReq)
	if err == nil && pagesResp.Error == nil {
		fmt.Printf("   找到 %d 个页面\n", len(pagesResp.Pages))
		for _, page := range pagesResp.Pages {
			fmt.Printf("   - %s: %s\n", page.PageId, page.Url)
		}
	}

	// 清理
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleCookieManagement Cookie 管理示例
func exampleCookieManagement() {
	fmt.Println("\n============================================================")
	fmt.Println("Cookie 管理示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 启动浏览器和页面
	launchResp, _ := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{Headless: true},
	})
	browserID := launchResp.BrowserInfo.BrowserId

	pageResp, _ := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})
	pageID := pageResp.PageInfo.PageId

	// 1. 设置 Cookie
	fmt.Println("\n1. 设置 Cookie...")
	setCookiesReq := &pagepb.SetCookiesRequest{
		PageId: pageID,
		Cookies: []*commonpb.Cookie{
			{
				Name:   "session_id",
				Value:  "abc123",
				Domain: "example.com",
				Path:   "/",
			},
			{
				Name:   "user_pref",
				Value:  "dark_mode",
				Domain: "example.com",
				Path:   "/",
			},
		},
	}

	setCookiesResp, err := client.Page.SetCookies(ctx, setCookiesReq)
	if err == nil && setCookiesResp.Error == nil {
		fmt.Printf("   成功设置 %d 个 Cookie\n", len(setCookiesReq.Cookies))
	}

	// 2. 获取 Cookie
	fmt.Println("\n2. 获取 Cookie...")
	getCookiesReq := &pagepb.GetCookiesRequest{
		PageId: pageID,
	}

	getCookiesResp, err := client.Page.GetCookies(ctx, getCookiesReq)
	if err == nil && getCookiesResp.Error == nil {
		fmt.Printf("   找到 %d 个 Cookie\n", len(getCookiesResp.Cookies))
		for _, cookie := range getCookiesResp.Cookies {
			fmt.Printf("   - %s: %s\n", cookie.Name, cookie.Value)
		}
	}

	// 3. 清除 Cookie
	fmt.Println("\n3. 清除 Cookie...")
	clearCookiesReq := &pagepb.ClearCookiesRequest{
		PageId: pageID,
	}

	clearCookiesResp, err := client.Page.ClearCookies(ctx, clearCookiesReq)
	if err == nil && clearCookiesResp.Error == nil {
		fmt.Printf("   Cookie 已清除\n")
	}

	// 清理
	_, _ = client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{PageId: pageID})
	_, _ = client.Browser.Close(ctx, &browserpb.CloseRequest{BrowserId: browserID})
}

// exampleWithErrorHandling 带错误处理的完整示例
func exampleWithErrorHandling() {
	fmt.Println("\n============================================================")
	fmt.Println("带错误处理的完整示例")
	fmt.Println("============================================================")

	client, err := NewChaserOxideClient("localhost:50051")
	if err != nil {
		log.Fatalf("创建客户端失败: %v", err)
	}
	defer func() {
		if err := client.Close(); err != nil {
			log.Printf("关闭客户端失败: %v", err)
		}
	}()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// 启动浏览器
	launchResp, err := client.Browser.Launch(ctx, &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{
			Headless:    true,
			WindowWidth:  1920,
			WindowHeight: 1080,
		},
	})

	if err != nil {
		log.Printf("   RPC 错误: %v", err)
		return
	}

	if launchResp.Error != nil {
		log.Printf("   启动失败: %s (code: %d)",
			launchResp.Error.Message,
			launchResp.Error.Code)
		return
	}

	browserID := launchResp.BrowserInfo.BrowserId
	fmt.Printf("✓ 浏览器已启动: %s\n", browserID)

	// 创建页面
	pageResp, err := client.Page.CreatePage(ctx, &pagepb.CreatePageRequest{
		BrowserId: browserID,
	})

	if err != nil {
		log.Printf("   RPC 错误: %v", err)
		return
	}

	if pageResp.Error != nil {
		log.Printf("   创建页面失败: %s", pageResp.Error.Message)
		return
	}

	pageID := pageResp.PageInfo.PageId
	fmt.Printf("✓ 页面已创建: %s\n", pageID)

	// 导航到 URL
	navigateResp, err := client.Page.Navigate(ctx, &pagepb.NavigateRequest{
		PageId: pageID,
		Url:    "https://example.com",
	})

	if err != nil {
		log.Printf("   RPC 错误: %v", err)
	} else if navigateResp.Error != nil {
		log.Printf("   导航失败: %s", navigateResp.Error.Message)
	} else {
		fmt.Printf("✓ 导航成功: %s (状态码: %d)\n",
			navigateResp.Result.Url,
			navigateResp.Result.StatusCode)
	}

	// 清理
	closePageResp, err := client.Page.ClosePage(ctx, &pagepb.ClosePageRequest{
		PageId: pageID,
	})

	if err != nil {
		log.Printf("   关闭页面失败: %v", err)
	} else if closePageResp.Error != nil {
		log.Printf("   关闭页面失败: %s", closePageResp.Error.Message)
	} else {
		fmt.Printf("✓ 页面已关闭\n")
	}

	closeBrowserResp, err := client.Browser.Close(ctx, &browserpb.CloseRequest{
		BrowserId: browserID,
	})

	if err != nil {
		log.Printf("   关闭浏览器失败: %v", err)
	} else if closeBrowserResp.Error != nil {
		log.Printf("   关闭浏览器失败: %s", closeBrowserResp.Error.Message)
	} else {
		fmt.Printf("✓ 浏览器已关闭\n")
	}

	fmt.Println("\n操作完成!")
}

func main() {
	fmt.Println("Chaser-Oxide Go 客户端示例")
	fmt.Println("====================================\n")

	// 运行所有示例
	exampleBasicNavigation()
	exampleElementInteraction()
	exampleEventSubscription()
	exampleStealthBrowsing()
	exampleBrowserInfo()
	exampleCookieManagement()
	exampleWithErrorHandling()

	fmt.Println("\n============================================================")
	fmt.Println("所有示例执行完成!")
	fmt.Println("============================================================")
}
