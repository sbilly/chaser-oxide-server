# Google 搜索自动化样例

通过 chaser-oxide-server 实现 Google 搜索自动化，支持代理配置。

## 功能特性

- **代理支持**: 支持 HTTP/HTTPS/SOCKS 代理，通过环境变量配置
- **自动分页**: 自动处理 Google 搜索分页，提取指定数量的结果
- **URL 解析**: 自动处理 Google 重定向链接，提取最终目标 URL
- **CSV 导出**: 将搜索结果导出为 CSV 格式，支持 UTF-8 BOM 编码
- **双语言实现**: 提供 Python 和 Go 两种实现方式

## 目录结构

```
examples/google_search_with_proxy/
├── README.md           # 本文档
├── requirements.txt    # Python 依赖
├── go.mod             # Go 模块配置
├── config.py          # Python 代理配置模块
├── config.go          # Go 代理配置模块
├── google_search.py   # Python 实现
└── google_search.go   # Go 实现
```

## 安装依赖

### Python

```bash
# 从项目根目录运行
cd examples/google_search_with_proxy
pip install -r requirements.txt
```

### Go

```bash
# 从项目根目录运行
cd examples/google_search_with_proxy
go mod download
```

## 代理配置

支持以下环境变量：

| 环境变量 | 说明 | 示例 |
|---------|------|------|
| `HTTP_PROXY` / `http_proxy` | HTTP 代理 | `http://127.0.0.1:8118` |
| `HTTPS_PROXY` / `https_proxy` | HTTPS 代理 | `http://127.0.0.1:8118` |
| `SOCKS_PROXY` / `socks_proxy` | SOCKS 代理 | `socks5://127.0.0.1:1080` |
| `ALL_PROXY` / `all_proxy` | 全局代理（优先级最高） | `http://127.0.0.1:8118` |
| `NO_PROXY` / `no_proxy` | 代理绕过列表 | `localhost,127.0.0.1,.local` |

### 代理优先级

```
ALL_PROXY > SOCKS_PROXY > HTTPS_PROXY > HTTP_PROXY
```

### 配置示例

```bash
# 使用 HTTP 代理
export HTTP_PROXY=http://127.0.0.1:8118

# 使用 SOCKS5 代理
export SOCKS_PROXY=socks5://127.0.0.1:1080

# 设置代理绕过
export NO_PROXY=localhost,127.0.0.1,.local

# 使用全局代理（覆盖其他配置）
export ALL_PROXY=http://127.0.0.1:8118
```

## 使用方法

### 启动 chaser-oxide-server

首先需要启动 chaser-oxide 服务器：

```bash
# 从项目根目录
cargo run --release
```

服务器默认监听 `localhost:50051`。

### Python 实现

```bash
# 从项目根目录运行
cd examples/google_search_with_proxy

# 基础使用（默认搜索词）
python google_search.py

# 自定义搜索词
python google_search.py --query "Python 教程"

# 指定结果数量
python google_search.py --max-results 50

# 指定输出文件
python google_search.py --output results.csv

# 手动指定代理（覆盖环境变量）
python google_search.py --proxy http://127.0.0.1:8118

# 完整参数
python google_search.py --query "机器学习" --max-results 100 --output ml_results.csv
```

#### 命令行参数

| 参数 | 简写 | 默认值 | 说明 |
|-----|------|--------|------|
| `--query` | `-q` | `通过图片定位地理位置` | 搜索关键词 |
| `--max-results` | `-n` | `100` | 最大结果数 |
| `--output` | `-o` | `search_results.csv` | 输出 CSV 文件 |
| `--host` | - | `localhost:50051` | chaser-oxide 服务器地址 |
| `--proxy` | - | (环境变量) | 代理服务器（覆盖环境变量） |

### Go 实现

```bash
# 从项目根目录运行
cd examples/google_search_with_proxy

# 基础使用
go run google_search.go config.go
```

Go 版本目前使用硬编码的默认参数。如需自定义参数，请修改 `main()` 函数中的相应变量。

## 输出格式

CSV 文件包含以下列：

| 列名 | 说明 |
|-----|------|
| `rank` | 排名（从 1 开始） |
| `title` | 搜索结果标题 |
| `url` | 最终 URL（已处理 Google 重定向） |
| `display_url` | 显示的 URL（原始链接，包含重定向） |

### 输出示例

```csv
rank,title,url,display_url
1,Geolocation of images - Google Photos,https://photos.google.com/search/_/_,https://www.google.com/url?q=https://photos.google.com/search/_/_
2,How to Find Location from Photo,https://www.example.com/find-location,https://www.google.com/url?q=https://www.example.com/find-location
```

## 工作原理

1. **启动浏览器**: 使用 chaser-oxide-server 启动无头 Chrome 浏览器，配置代理（如果指定）
2. **导航到 Google**: 访问 https://www.google.com
3. **输入搜索词**: 在搜索框中输入关键词
4. **提交搜索**: 按 Enter 键提交搜索请求
5. **提取结果**: 使用 JavaScript 从页面 DOM 中提取搜索结果
6. **处理分页**: 自动点击"下一页"按钮，直到获得足够的结果
7. **处理 URL**: 解析 Google 的重定向链接，提取最终目标 URL
8. **保存结果**: 将结果保存为 CSV 文件

### JavaScript 提取逻辑

```javascript
// 查找所有搜索结果容器
const containers = document.querySelectorAll('div.g');

// 遍历每个容器
containers.forEach(container => {
    const titleElement = container.querySelector('h3');
    const linkElement = container.querySelector('a');

    if (titleElement && linkElement) {
        let finalUrl = linkElement.href;

        // 处理 Google 重定向 (/url?q=...)
        if (finalUrl.includes('/url?q=')) {
            const urlMatch = finalUrl.match(/[?&]q=([^&]+)/);
            if (urlMatch) {
                finalUrl = decodeURIComponent(urlMatch[1]);
            }
        }

        results.push({
            title: titleElement.textContent.trim(),
            url: finalUrl,
            display_url: linkElement.href
        });
    }
});
```

## 常见问题

### Q: 如何查看代理是否生效？

**A**: 启动时会打印代理配置信息，例如：
```
浏览器已启动: browser-123
  使用代理: http://127.0.0.1:8118
  绕过列表: localhost,127.0.0.1
```

### Q: Google 返回 CAPTCHA 验证怎么办？

**A**: 这表明 Google 检测到自动化访问。可以尝试：
- 减少搜索频率
- 使用不同的代理 IP
- 使用住宅代理而非数据中心代理
- 暂时使用非 headless 模式进行人工验证

### Q: 如何使用非 headless 模式？

**A**: 修改代码中的 `headless` 参数：
- Python: `google_search.py` 第 79 行，将 `headless=True` 改为 `headless=False`
- Go: `google_search.go` 第 125 行，将 `Headless: true` 改为 `Headless: false`

### Q: 为什么提取的结果少于 100 个？

**A**: 可能的原因：
- Google 搜索结果本身少于 100 个
- 检测到自动化访问，限制了结果数量
- 网络问题导致页面加载失败
- DOM 结构变化导致选择器失效

### Q: 如何修改超时时间？

**A**: 修改代码中的 `timeout` 参数：
- Python: `google_search.py` 第 135 行，修改 `timeout=30000`（毫秒）
- Go: `google_search.go` 第 177 行，修改 `Timeout: 30000`（毫秒）

### Q: 支持其他搜索引擎吗？

**A**: 当前实现仅支持 Google。如需支持其他搜索引擎，需要：
1. 修改导航 URL
2. 调整搜索框选择器
3. 调整结果容器和标题选择器
4. 根据需要修改分页逻辑

### Q: 如何调试选择器问题？

**A**: 使用浏览器开发者工具：
1. 在浏览器中打开 Google 搜索
2. 按 F12 打开开发者工具
3. 使用 Ctrl+Shift+C (Windows/Linux) 或 Cmd+Shift+C (Mac) 检查元素
4. 验证选择器是否正确匹配

## 技术细节

### DOM 选择器

| 元素 | CSS 选择器 |
|-----|-----------|
| 搜索框 | `textarea[name='q']` |
| 结果容器 | `div.g` |
| 标题 | `h3` |
| 链接 | `a` |
| 下一页 | `#pnnext`, `a[aria-label="Next"]`, `span.YyVfkd` |

### URL 重定向格式

Google 搜索结果使用以下格式进行重定向：

```
https://www.google.com/url?q=<encoded_url>&usg=...
```

代码使用正则表达式 `/[?&]q=([^&]+)/` 提取 `q` 参数，然后使用 `decodeURIComponent()` 解码。

### 分页策略

- Google 每页约显示 10 个结果
- 代码循环检查是否存在下一页按钮
- 使用多个选择器以适应不同的页面布局
- 最大提取 100 个结果（约 10 页）

## 许可证

本样例代码遵循项目主许可证。

## 相关资源

- [chaser-oxide-server 文档](../../README.md)
- [Python gRPC 文档](https://grpc.io/docs/languages/python/)
- [Go gRPC 文档](https://grpc.io/docs/languages/go/)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
