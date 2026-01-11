// Chaser-Oxide Google 搜索爬虫
//
// 通过 chaser-oxide-server 实现 Google 搜索自动化，支持代理配置。
//
// 依赖安装:
//     go mod download
//
// 使用方法:
//     cd examples/google_search_with_proxy
//     go run google_search.go config.go
package main

import (
	"context"
	"encoding/csv"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	commonpb "chaser-oxide-server/protos/v1"
	browserpb "chaser-oxide-server/protos/v1"
	pagepb "chaser-oxide-server/protos/v1"
	elementpb "chaser-oxide-server/protos/v1"

	browsergrpc "chaser-oxide-server/protos/v1"
	pagegrpc "chaser-oxide-server/protos/v1"
	elementgrpc "chaser-oxide-server/protos/v1"
)

// SearchResult 搜索结果
type SearchResult struct {
	Rank       int    `json:"rank"`
	Title      string `json:"title"`
	URL        string `json:"url"`
	DisplayURL string `json:"display_url"`
}

// GoogleSearchScraper Google 搜索爬虫
type GoogleSearchScraper struct {
	conn        *grpc.ClientConn
	browser     browsergrpc.BrowserServiceClient
	page        pagegrpc.PageServiceClient
	element     elementgrpc.ElementServiceClient
	proxyConfig *ProxyConfig
	browserID   string
	pageID      string
}

// NewGoogleSearchScraper 创建爬虫实例
func NewGoogleSearchScraper(host string, proxyConfig *ProxyConfig) (*GoogleSearchScraper, error) {
	conn, err := grpc.NewClient(host, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, fmt.Errorf("连接失败: %w", err)
	}

	return &GoogleSearchScraper{
		conn:        conn,
		browser:     browsergrpc.NewBrowserServiceClient(conn),
		page:        pagegrpc.NewPageServiceClient(conn),
		element:     elementgrpc.NewElementServiceClient(conn),
		proxyConfig: proxyConfig,
	}, nil
}

// Close 关闭连接
func (s *GoogleSearchScraper) Close() error {
	if s.pageID != "" {
		_, _ = s.page.ClosePage(context.Background(), &pagepb.ClosePageRequest{
			PageId: s.pageID,
		})
		fmt.Println("页面已关闭")
	}

	if s.browserID != "" {
		_, _ = s.browser.Close(context.Background(), &browserpb.CloseRequest{
			BrowserId: s.browserID,
		})
		fmt.Println("浏览器已关闭")
	}

	return s.conn.Close()
}

// LaunchBrowser 启动浏览器
func (s *GoogleSearchScraper) LaunchBrowser() error {
	proxyServer := s.proxyConfig.ToChaserProxy()
	proxyBypass := s.proxyConfig.ToChaserBypassList()

	launchReq := &browserpb.LaunchRequest{
		Options: &commonpb.BrowserOptions{
			Headless:       true,
			WindowWidth:    1920,
			WindowHeight:   1080,
			ProxyServer:    proxyServer,
			ProxyBypassList: proxyBypass,
			UserAgent:      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
		},
	}

	launchResp, err := s.browser.Launch(context.Background(), launchReq)
	if err != nil {
		return fmt.Errorf("启动浏览器失败: %w", err)
	}

	if launchResp.Error != nil {
		return fmt.Errorf("启动浏览器失败: %s", launchResp.Error.Message)
	}

	s.browserID = launchResp.BrowserInfo.BrowserId
	fmt.Printf("浏览器已启动: %s\n", s.browserID)

	if proxyServer != "" {
		fmt.Printf("  使用代理: %s\n", proxyServer)
	}
	if proxyBypass != "" {
		fmt.Printf("  绕过列表: %s\n", proxyBypass)
	}

	return nil
}

// CreatePage 创建页面
func (s *GoogleSearchScraper) CreatePage() error {
	createReq := &pagepb.CreatePageRequest{
		BrowserId: s.browserID,
		Url:       "about:blank",
	}

	createResp, err := s.page.CreatePage(context.Background(), createReq)
	if err != nil {
		return fmt.Errorf("创建页面失败: %w", err)
	}

	if createResp.Error != nil {
		return fmt.Errorf("创建页面失败: %s", createResp.Error.Message)
	}

	s.pageID = createResp.PageInfo.PageId
	fmt.Printf("页面已创建: %s\n", s.pageID)

	return nil
}

// Navigate 导航到 URL
func (s *GoogleSearchScraper) Navigate(url string) error {
	navigateReq := &pagepb.NavigateRequest{
		PageId: s.pageID,
		Url:    url,
		Options: &commonpb.NavigationOptions{
			Timeout: 30000,
			WaitUntil: &commonpb.NavigationOptions_LoadState{
				LoadState: commonpb.NavigationOptions_LOAD_STATE_NETWORK_IDLE,
			},
		},
	}

	navigateResp, err := s.page.Navigate(context.Background(), navigateReq)
	if err != nil {
		return fmt.Errorf("导航失败: %w", err)
	}

	if navigateResp.Error != nil {
		return fmt.Errorf("导航失败: %s", navigateResp.Error.Message)
	}

	fmt.Printf("已导航到: %s\n", url)
	return nil
}

// Search 执行搜索
func (s *GoogleSearchScraper) Search(query string, maxResults int) ([]SearchResult, error) {
	// 导航到 Google
	if err := s.Navigate("https://www.google.com"); err != nil {
		return nil, err
	}

	time.Sleep(2 * time.Second)

	// 查找搜索框
	findReq := &elementpb.FindElementRequest{
		PageId:         s.pageID,
		SelectorType:   commonpb.SelectorType_SELECTOR_TYPE_CSS,
		Selector:       "textarea[name='q']",
		WaitForVisible: true,
		Timeout:        10000,
	}

	findResp, err := s.element.FindElement(context.Background(), findReq)
	if err != nil {
		return nil, fmt.Errorf("查找搜索框失败: %w", err)
	}

	if findResp.Error != nil {
		return nil, fmt.Errorf("查找搜索框失败: %s", findResp.Error.Message)
	}

	searchBox := findResp.Element

	// 输入搜索词
	typeReq := &elementpb.TypeRequest{
		Element:    searchBox,
		Text:       query,
		ClearFirst: true,
	}

	typeResp, err := s.element.Type(context.Background(), typeReq)
	if err != nil {
		return nil, fmt.Errorf("输入搜索词失败: %w", err)
	}

	if typeResp.Error != nil {
		return nil, fmt.Errorf("输入搜索词失败: %s", typeResp.Error.Message)
	}

	fmt.Printf("已输入搜索词: %s\n", query)

	// 使用 JavaScript 提交搜索表单（比 PressKey 更可靠）
	submitScript := `
	(() => {
		// 方法1: 提交表单
		const form = document.querySelector('form');
		if (form) {
			form.submit();
			return JSON.stringify({method: 'form_submit', success: true});
		}

		// 方法2: 触发搜索按钮点击
		const submitButton = document.querySelector('input[type="submit"]');
		if (submitButton) {
			submitButton.click();
			return JSON.stringify({method: 'button_click', success: true});
		}

		// 方法3: 直接构造 URL 并导航
		const searchInput = document.querySelector('textarea[name="q"], input[name="q"]');
		if (searchInput && searchInput.value) {
			const searchUrl = 'https://www.google.com/search?q=' + encodeURIComponent(searchInput.value);
			window.location.href = searchUrl;
			return JSON.stringify({method: 'direct_navigation', success: true});
		}

		return JSON.stringify({method: 'none', success: false, error: 'No submission method found'});
	})()
	`

	fmt.Printf("  [DEBUG] 准备提交搜索表单...\n")

	evalReq := &pagepb.EvaluateRequest{
		PageId:       s.pageID,
		Expression:   submitScript,
		AwaitPromise: true,
	}

	evalResp, err := s.page.Evaluate(context.Background(), evalReq)
	if err != nil {
		fmt.Printf("  [DEBUG] JavaScript 提交失败: %v\n", err)
		// 回退到 PressKey 方法
		fmt.Printf("  [DEBUG] 回退使用 PressKey 方法...\n")
		pressKeyReq := &elementpb.PressKeyRequest{
			Element: searchBox,
			Key:     "Enter",
		}
		pressKeyResp, err := s.element.PressKey(context.Background(), pressKeyReq)
		if err != nil {
			return nil, fmt.Errorf("提交搜索失败: %w", err)
		}
		if pressKeyResp.Error != nil {
			return nil, fmt.Errorf("提交搜索失败: %s", pressKeyResp.Error.Message)
		}
	} else if evalResp.Error != nil {
		fmt.Printf("  [DEBUG] JavaScript 提交失败: %s\n", evalResp.Error.Message)
		// 回退到 PressKey 方法
		fmt.Printf("  [DEBUG] 回退使用 PressKey 方法...\n")
		pressKeyReq := &elementpb.PressKeyRequest{
			Element: searchBox,
			Key:     "Enter",
		}
		pressKeyResp, err := s.element.PressKey(context.Background(), pressKeyReq)
		if err != nil {
			return nil, fmt.Errorf("提交搜索失败: %w", err)
		}
		if pressKeyResp.Error != nil {
			return nil, fmt.Errorf("提交搜索失败: %s", pressKeyResp.Error.Message)
		}
	} else {
		// 解析 JavaScript 返回结果
		var result struct {
			Method  string `json:"method"`
			Success bool   `json:"success"`
			Error   string `json:"error,omitempty"`
		}
		if err := json.Unmarshal([]byte(evalResp.Result.StringValue), &result); err == nil {
			fmt.Printf("  [DEBUG] 提交方法: %s, 成功: %v\n", result.Method, result.Success)
		}
	}

	fmt.Printf("  [DEBUG] 等待搜索结果加载...\n")
	time.Sleep(3 * time.Second)
	fmt.Printf("  [DEBUG] 开始提取搜索结果...\n")

	// 提取搜索结果
	var allResults []SearchResult
	pageNum := 0

	for len(allResults) < maxResults {
		results := s.extractResultsFromPage()

		// 添加排名
		for i := range results {
			results[i].Rank = len(allResults) + i + 1
		}

		allResults = append(allResults, results...)

		fmt.Printf("第 %d 页: 提取到 %d 个结果\n", pageNum+1, len(results))

		if len(allResults) >= maxResults {
			break
		}

		if !s.hasNextPage() {
			fmt.Println("没有更多结果页")
			break
		}

		if err := s.clickNextPage(); err != nil {
			fmt.Println("无法点击下一页")
			break
		}

		pageNum++
		time.Sleep(2 * time.Second)
	}

	if len(allResults) > maxResults {
		allResults = allResults[:maxResults]
	}

	return allResults, nil
}

// extractResultsFromPage 从当前页提取结果
func (s *GoogleSearchScraper) extractResultsFromPage() []SearchResult {
	script := `
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
	`

	evalReq := &pagepb.EvaluateRequest{
		PageId:       s.pageID,
		Expression:   script,
		AwaitPromise: true,
	}

	evalResp, err := s.page.Evaluate(context.Background(), evalReq)
	if err != nil {
		fmt.Printf("  警告: 提取结果失败: %v\n", err)
		return nil
	}

	if evalResp.Error != nil {
		fmt.Printf("  警告: 提取结果失败: %s\n", evalResp.Error.Message)
		return nil
	}

	// 解析包含调试信息的响应
	var response struct {
		Results []SearchResult `json:"results"`
		Debug   struct {
			SelectorsTested []map[string]interface{} `json:"selectors_tested"`
			FoundElements   int                      `json:"found_elements"`
		} `json:"debug"`
	}

	if err := json.Unmarshal([]byte(evalResp.Result.StringValue), &response); err != nil {
		// 尝试解析旧格式（直接数组）
		var results []SearchResult
		if err2 := json.Unmarshal([]byte(evalResp.Result.StringValue), &results); err2 != nil {
			fmt.Printf("  警告: 解析结果失败: %v\n", err)
			return nil
		}
		return results
	}

	// 只在找不到结果时打印调试信息
	if response.Debug.FoundElements == 0 {
		fmt.Printf("  [DEBUG] 选择器测试结果:\n")
		for _, s := range response.Debug.SelectorsTested {
			selector := s["selector"].(string)
			count := s["count"].(float64)
			fmt.Printf("           '%s': %d 个元素\n", selector, int(count))
		}
	}

	return response.Results
}

// hasNextPage 检查是否有下一页
func (s *GoogleSearchScraper) hasNextPage() bool {
	script := `
	!!(
		document.querySelector('#pnnext') ||
		document.querySelector('a[aria-label="Next"]') ||
		document.querySelector('span.YyVfkd')
	)
	`

	evalReq := &pagepb.EvaluateRequest{
		PageId:     s.pageID,
		Expression: script,
	}

	evalResp, err := s.page.Evaluate(context.Background(), evalReq)
	if err != nil || evalResp.Error != nil {
		return false
	}

	return evalResp.Result.BoolValue
}

// clickNextPage 点击下一页
func (s *GoogleSearchScraper) clickNextPage() error {
	selectors := []string{
		"a#pnnext",
		`a[aria-label="Next"]`,
		"span.YyVfkd",
	}

	for _, selector := range selectors {
		findReq := &elementpb.FindElementRequest{
			PageId:       s.pageID,
			SelectorType: commonpb.SelectorType_SELECTOR_TYPE_CSS,
			Selector:     selector,
			Timeout:      1000,
		}

		findResp, err := s.element.FindElement(context.Background(), findReq)
		if err != nil || findResp.Error != nil {
			continue
		}

		clickReq := &elementpb.ClickRequest{
			Element: findResp.Element,
		}

		clickResp, err := s.element.Click(context.Background(), clickReq)
		if err != nil || clickResp.Error != nil {
			continue
		}

		return nil
	}

	return fmt.Errorf("未找到下一页按钮")
}

// SaveToCSV 保存结果到 CSV
func (s *GoogleSearchScraper) SaveToCSV(results []SearchResult, filename string) error {
	if len(results) == 0 {
		fmt.Println("没有结果可保存")
		return nil
	}

	file, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("创建文件失败: %w", err)
	}
	defer file.Close()

	// 使用 UTF-8 BOM 以确保 Excel 正确显示中文
	writer := csv.Writer{}
	writer.UseCRLF = false

	file.WriteString("\xEF\xBB\xBF")

	csvWriter := csv.NewWriter(file)
	defer csvWriter.Flush()

	headers := []string{"rank", "title", "url", "display_url"}
	if err := csvWriter.Write(headers); err != nil {
		return fmt.Errorf("写入表头失败: %w", err)
	}

	for _, result := range results {
		record := []string{
			strconv.Itoa(result.Rank),
			result.Title,
			result.URL,
			result.DisplayURL,
		}
		if err := csvWriter.Write(record); err != nil {
			return fmt.Errorf("写入记录失败: %w", err)
		}
	}

	fmt.Printf("结果已保存到: %s\n", filename)
	return nil
}

func main() {
	// 默认参数
	query := "通过图片定位地理位置"
	maxResults := 100
	output := "search_results.csv"
	host := "localhost:50051"

	// 加载代理配置
	proxyConfig := NewProxyConfigFromEnv()

	fmt.Println("============================================================")
	fmt.Println("Google 搜索爬虫")
	fmt.Println("============================================================")
	fmt.Printf("搜索关键词: %s\n", query)
	fmt.Printf("最大结果数: %d\n", maxResults)
	fmt.Printf("输出文件: %s\n", output)

	if proxy := proxyConfig.ToChaserProxy(); proxy != "" {
		fmt.Printf("代理配置: %s\n", proxyConfig.String())
	}

	fmt.Println("============================================================")

	scraper, err := NewGoogleSearchScraper(host, proxyConfig)
	if err != nil {
		log.Fatalf("创建爬虫失败: %v", err)
	}
	defer scraper.Close()

	// 启动浏览器
	if err := scraper.LaunchBrowser(); err != nil {
		log.Fatalf("启动浏览器失败: %v", err)
	}

	// 创建页面
	if err := scraper.CreatePage(); err != nil {
		log.Fatalf("创建页面失败: %v", err)
	}

	// 执行搜索
	fmt.Printf("\n开始搜索: %s\n", query)
	results, err := scraper.Search(query, maxResults)
	if err != nil {
		log.Fatalf("搜索失败: %v", err)
	}

	// 保存结果
	scraper.SaveToCSV(results, output)

	fmt.Println("============================================================")
	fmt.Printf("搜索完成! 共获取 %d 个结果\n", len(results))
	fmt.Printf("结果已保存到: %s\n", output)
	fmt.Println("============================================================")
}
