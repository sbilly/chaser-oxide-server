# Chaser-Oxide API 使用文档

本文档详细说明 Chaser-Oxide gRPC 服务的 API 接口、消息格式和使用示例。

## 目录

- [服务概述](#服务概述)
- [通用概念](#通用概念)
- [BrowserService](#browserservice)
- [PageService](#pageservice)
- [ElementService](#elementservice)
- [EventService](#eventservice)
- [ProfileService](#profileservice)
- [错误处理](#错误处理)
- [最佳实践](#最佳实践)

## 服务概述

Chaser-Oxide 提供 5 个核心 gRPC 服务：

| 服务 | 描述 |
|------|------|
| `BrowserService` | 浏览器生命周期管理 |
| `PageService` | 页面操作和内容访问 |
| `ElementService` | 元素查找和交互 |
| `EventService` | 实时事件订阅和推送 |
| `ProfileService` | 隐身配置和指纹管理 |

## 通用概念

### 会话标识

所有操作都需要会话标识来定位特定的浏览器、页面或元素：

```protobuf
message BrowserId {
    string value = 1;  // 浏览器实例 UUID
}

message PageId {
    string value = 1;  // 页面 UUID
}

message ElementId {
    string value = 1;  // 元素引用 ID
}
```

### 选项模式

大多数操作支持可选参数：

```protobuf
message NavigationOptions {
    LoadState wait_until = 1;    // 等待状态
    int32 timeout = 2;            // 超时时间（毫秒）
    string referrer = 3;          // 引用页
}
```

### 响应格式

所有响应使用 `oneof` 包装，确保类型安全：

```protobuf
message Response {
    oneof result {
        SuccessType success = 1;
        Error error = 2;
    }
}
```

## BrowserService

浏览器生命周期管理服务。

### 启动浏览器

启动一个新的浏览器实例。

**方法**: `Launch`

**请求**:
```protobuf
message LaunchRequest {
    BrowserOptions options = 1;  // 浏览器配置选项
}

message BrowserOptions {
    bool headless = 1;           // 无头模式
    int32 window_width = 2;      // 窗口宽度
    int32 window_height = 3;     // 窗口高度
    string executable_path = 4;  // Chrome 可执行文件路径
    repeated string args = 5;    // 额外命令行参数
    ProxyConfig proxy = 6;       // 代理配置
    int64 timeout = 7;           // 启动超时（毫秒）
}
```

**响应**:
```protobuf
message LaunchResponse {
    BrowserInfo browser_info = 1;  // 浏览器信息
}

message BrowserInfo {
    BrowserId browser_id = 1;      // 浏览器 ID
    string version = 2;            // 浏览器版本
    string user_agent = 3;         // User Agent
}
```

**使用示例**:
```python
# Python 客户端
request = LaunchRequest(
    options=BrowserOptions(
        headless=True,
        window_width=1920,
        window_height=1080,
        args=["--disable-gpu", "--no-sandbox"]
    )
)
response = client.browser.Launch(request)
browser_id = response.browser_info.browser_id
print(f"Browser launched: {browser_id}")
```

### 获取页面列表

获取指定浏览器的所有页面。

**方法**: `GetPages`

**请求**:
```protobuf
message GetPagesRequest {
    BrowserId browser_id = 1;  // 浏览器 ID
}
```

**响应**:
```protobuf
message GetPagesResponse {
    repeated Page pages = 1;  // 页面列表
}

message Page {
    PageId page_id = 1;       // 页面 ID
    string url = 2;           // 当前 URL
    string title = 3;         // 页面标题
    bool is_loaded = 4;       // 是否加载完成
}
```

### 关闭浏览器

关闭指定的浏览器实例及其所有页面。

**方法**: `Close`

**请求**:
```protobuf
message CloseRequest {
    BrowserId browser_id = 1;  // 浏览器 ID
}
```

**响应**:
```protobuf
message CloseResponse {
    bool success = 1;  // 是否成功
}
```

### 连接到现有浏览器

连接到已运行的 Chrome 实例。

**方法**: `Connect`

**请求**:
```protobuf
message ConnectRequest {
    string endpoint = 1;  // CDP WebSocket 端点
}
```

**响应**:
```protobuf
message ConnectResponse {
    BrowserInfo browser_info = 1;  // 浏览器信息
}
```

### 获取浏览器版本

获取浏览器版本信息。

**方法**: `GetVersion`

**请求**:
```protobuf
message GetVersionRequest {}
```

**响应**:
```protobuf
message GetVersionResponse {
    string protocol_version = 1;  // CDP 协议版本
    string product = 2;           // 产品名称
    string revision = 3;          // 修订版本
    string user_agent = 4;        // User Agent
    string js_version = 5;        // JavaScript 版本
}
```

### 获取浏览器状态

获取浏览器运行状态。

**方法**: `GetStatus`

**请求**:
```protobuf
message GetStatusRequest {
    BrowserId browser_id = 1;  // 浏览器 ID
}
```

**响应**:
```protobuf
message GetStatusResponse {
    bool is_running = 1;         // 是否运行中
    int32 page_count = 2;        // 页面数量
    int64 memory_usage = 3;      // 内存使用（字节）
    int64 cpu_usage = 4;         // CPU 使用（百分比）
}
```

## PageService

页面操作服务。

### 创建页面

在指定浏览器中创建新页面。

**方法**: `CreatePage`

**请求**:
```protobuf
message CreatePageRequest {
    BrowserId browser_id = 1;  // 浏览器 ID
}
```

**响应**:
```protobuf
message CreatePageResponse {
    Page page = 1;  // 新创建的页面
}
```

### 导航

导航到指定 URL。

**方法**: `Navigate`

**请求**:
```protobuf
message NavigateRequest {
    PageId page_id = 1;              // 页面 ID
    string url = 2;                  // 目标 URL
    NavigationOptions options = 3;   // 导航选项
}

message NavigationOptions {
    LoadState wait_until = 1;  // 等待状态
    int32 timeout = 2;          // 超时（毫秒）
    string referrer = 3;        // 引用页
}

enum LoadState {
    LOAD_STATE_UNSPECIFIED = 0;
    LOAD_STATE_LOAD = 1;              // onload 事件触发
    LOAD_STATE_DOM_CONTENT_LOADED = 2; // DOMContentLoaded 事件触发
    LOAD_STATE_NETWORK_IDLE = 3;      // 网络空闲（至少 500ms 无网络请求）
    LOAD_STATE_NETWORK_ALMOST_IDLE = 4; // 网络基本空闲
}
```

**响应**:
```protobuf
message NavigateResponse {
    string url = 1;        // 实际导航的 URL
    int64 timestamp = 2;   // 导航时间戳
}
```

**使用示例**:
```python
# 导航并等待网络空闲
request = NavigateRequest(
    page_id=page_id,
    url="https://example.com",
    options=NavigationOptions(
        wait_until=LoadState.LOAD_STATE_NETWORK_IDLE,
        timeout=30000
    )
)
response = client.page.Navigate(request)
print(f"Navigated to: {response.url}")
```

### 获取页面快照

获取页面的可访问性树快照。

**方法**: `GetSnapshot`

**请求**:
```protobuf
message GetSnapshotRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message GetSnapshotResponse {
    repeated SnapshotNode nodes = 1;  // 快照节点列表
}

message SnapshotNode {
    string uid = 1;              // 唯一标识符
    string role = 2;             // 角色（button、link 等）
    string name = 3;             // 名称
    string description = 4;      // 描述
    repeated string attributes = 5;  // 属性列表
    repeated SnapshotNode children = 6;  // 子节点
}
```

### 截图

对页面进行截图。

**方法**: `Screenshot`

**请求**:
```protobuf
message ScreenshotRequest {
    PageId page_id = 1;       // 页面 ID
    string format = 2;        // 图片格式（png、jpeg、webp）
    int32 quality = 3;        // 图片质量（0-100，仅 jpeg/webp）
    string clip = 4;          // 裁剪区域
    bool full_page = 5;       // 是否全页截图
}

message Clip {
    double x = 1;
    double y = 2;
    double width = 3;
    double height = 4;
    double scale = 5;
}
```

**响应**:
```protobuf
message ScreenshotResponse {
    bytes data = 1;      // 图片数据（Base64 编码）
    int32 width = 2;     // 图片宽度
    int32 height = 3;    // 图片高度
}
```

### 执行 JavaScript

在页面上下文中执行 JavaScript 代码。

**方法**: `Evaluate`

**请求**:
```protobuf
message EvaluateRequest {
    PageId page_id = 1;           // 页面 ID
    string expression = 2;         // JavaScript 表达式
    bool await_promise = 3;        // 是否等待 Promise
    int32 timeout = 4;             // 超时（毫秒）
}
```

**响应**:
```protobuf
message EvaluateResponse {
    EvalResult result = 1;  // 执行结果
}

message EvalResult {
    oneof value {
        string string_value = 1;
        int64 int_value = 2;
        double double_value = 3;
        bool bool_value = 4;
        google.protobuf.NullValue null_value = 5;
    }
    string error = 6;  // 错误信息（如果有）
}
```

**使用示例**:
```python
# 获取页面标题
request = EvaluateRequest(
    page_id=page_id,
    expression="document.title"
)
response = client.page.Evaluate(request)
title = response.result.string_value
print(f"Page title: {title}")

# 执行异步操作
request = EvaluateRequest(
    page_id=page_id,
    expression="fetch('/api/data').then(r => r.json())",
    await_promise=True,
    timeout=5000
)
response = client.page.Evaluate(request)
```

### 设置页面内容

设置页面的 HTML 内容。

**方法**: `SetContent`

**请求**:
```protobuf
message SetContentRequest {
    PageId page_id = 1;      // 页面 ID
    string html = 2;         // HTML 内容
    int32 timeout = 3;       // 超时（毫秒）
}
```

**响应**:
```protobuf
message SetContentResponse {
    bool success = 1;  // 是否成功
}
```

### 获取页面内容

获取页面的 HTML 内容。

**方法**: `GetContent`

**请求**:
```protobuf
message GetContentRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message GetContentResponse {
    string content = 1;  // HTML 内容
}
```

### 刷新页面

重新加载当前页面。

**方法**: `Reload`

**请求**:
```protobuf
message ReloadRequest {
    PageId page_id = 1;              // 页面 ID
    NavigationOptions options = 2;   // 导航选项
}
```

**响应**:
```protobuf
message ReloadResponse {
    string url = 1;  // 刷新后的 URL
}
```

### 后退和前进

在浏览器历史中导航。

**方法**: `GoBack` / `GoForward`

**请求**:
```protobuf
message GoBackRequest {
    PageId page_id = 1;  // 页面 ID
}

message GoForwardRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message GoBackResponse {
    string url = 1;  // 导航后的 URL
}

message GoForwardResponse {
    string url = 1;  // 导航后的 URL
}
```

### 设置视口

设置页面视口大小。

**方法**: `SetViewport`

**请求**:
```protobuf
message SetViewportRequest {
    PageId page_id = 1;       // 页面 ID
    int32 width = 2;          // 视口宽度
    int32 height = 3;         // 视口高度
    double device_scale_factor = 4;  // 设备缩放因子
    bool mobile = 5;          // 是否移动设备
    double orientation = 6;    // 屏幕方向（度）
}
```

**响应**:
```protobuf
message SetViewportResponse {
    bool success = 1;  // 是否成功
}
```

### 模拟设备

使用预设设备配置模拟移动设备。

**方法**: `EmulateDevice`

**请求**:
```protobuf
message EmulateDeviceRequest {
    PageId page_id = 1;        // 页面 ID
    DeviceType device = 2;      // 设备类型
}

enum DeviceType {
    DEVICE_TYPE_UNSPECIFIED = 0;
    DEVICE_TYPE_DESKTOP = 1;     // 桌面
    DEVICE_TYPE_IPHONE = 2;      // iPhone
    DEVICE_TYPE_IPAD = 3;        // iPad
    DEVICE_TYPE_ANDROID_PHONE = 4;  // Android 手机
    DEVICE_TYPE_ANDROID_TABLET = 5; // Android 平板
}
```

**响应**:
```protobuf
message EmulateDeviceResponse {
    bool success = 1;  // 是否成功
}
```

### 关闭页面

关闭指定页面。

**方法**: `ClosePage`

**请求**:
```protobuf
message ClosePageRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message ClosePageResponse {
    bool success = 1;  // 是否成功
}
```

### 等待条件

等待页面满足特定条件。

**方法**: `WaitFor`

**请求**:
```protobuf
message WaitForRequest {
    PageId page_id = 1;       // 页面 ID
    string selector = 2;      // CSS 选择器
    int32 timeout = 3;        // 超时（毫秒）
    WaitForState state = 4;   // 等待状态
}

enum WaitForState {
    WAIT_FOR_STATE_UNSPECIFIED = 0;
    WAIT_FOR_STATE_ATTACHED = 1;     // 元素附加到 DOM
    WAIT_FOR_STATE_DETACHED = 2;     // 元素从 DOM 移除
    WAIT_FOR_STATE_VISIBLE = 3;      // 元素可见
    WAIT_FOR_STATE_HIDDEN = 4;       // 元素隐藏
    WAIT_FOR_STATE_ENABLED = 5;      // 元素可用
    WAIT_FOR_STATE_DISABLED = 6;     // 元素禁用
}
```

**响应**:
```protobuf
message WaitForResponse {
    bool success = 1;  // 是否成功
}
```

### Cookie 管理

获取、设置和清除 Cookie。

**方法**: `GetCookies` / `SetCookies` / `ClearCookies`

**请求**:
```protobuf
message GetCookiesRequest {
    PageId page_id = 1;        // 页面 ID
    repeated string urls = 2;   // 过滤 URL
}

message SetCookiesRequest {
    PageId page_id = 1;                // 页面 ID
    repeated Cookie cookies = 2;        // 要设置的 Cookie
}

message Cookie {
    string name = 1;
    string value = 2;
    string domain = 3;
    string path = 4;
    int64 expires = 5;      // 过期时间（Unix 时间戳）
    bool http_only = 6;
    bool secure = 7;
    string same_site = 8;   // Strict、Lax、None
}

message ClearCookiesRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message GetCookiesResponse {
    repeated Cookie cookies = 1;  // Cookie 列表
}

message SetCookiesResponse {
    bool success = 1;  // 是否成功
}

message ClearCookiesResponse {
    bool success = 1;  // 是否成功
}
```

## ElementService

元素交互服务。

### 查找元素

在页面中查找元素。

**方法**: `FindElement` / `FindElements`

**请求**:
```protobuf
message FindElementRequest {
    PageId page_id = 1;       // 页面 ID
    SelectorType selector_type = 2;  // 选择器类型
    string selector = 3;      // 选择器字符串
}

enum SelectorType {
    SELECTOR_TYPE_UNSPECIFIED = 0;
    SELECTOR_TYPE_CSS = 1;         // CSS 选择器
    SELECTOR_TYPE_XPATH = 2;       // XPath 选择器
    SELECTOR_TYPE_TEXT = 3;        // 文本内容
    SELECTOR_TYPE_ARIA = 4;        // ARIA 标签
}
```

**响应**:
```protobuf
message FindElementResponse {
    Element element = 1;  // 找到的元素
}

message FindElementsResponse {
    repeated Element elements = 1;  // 元素列表
}

message Element {
    ElementId element_id = 1;  // 元素 ID
    string tag_name = 2;       // 标签名
    string text_content = 3;   // 文本内容
    map<string, string> attributes = 4;  // 属性
}
```

**使用示例**:
```python
# 查找提交按钮
request = FindElementRequest(
    page_id=page_id,
    selector_type=SelectorType.SELECTOR_TYPE_CSS,
    selector="#submit-button"
)
response = client.element.FindElement(request)
element = response.element
print(f"Found element: {element.tag_name}")
```

### 点击元素

点击指定元素。

**方法**: `Click`

**请求**:
```protobuf
message ClickRequest {
    ElementId element_id = 1;  // 元素 ID
    bool human_like = 2;       // 是否模拟人类点击（Bezier 曲线移动）
    int32 movement_duration = 3;  // 鼠标移动持续时间（毫秒）
    int32 delay = 4;           // 点击后延迟（毫秒）
}
```

**响应**:
```protobuf
message ClickResponse {
    bool success = 1;  // 是否成功
}
```

### 输入文本

向元素输入文本。

**方法**: `Type`

**请求**:
```protobuf
message TypeRequest {
    ElementId element_id = 1;  // 元素 ID
    string text = 2;            // 要输入的文本
    int32 delay = 3;            // 每个字符间的延迟（毫秒）
    bool clear_first = 4;       // 是否先清除现有内容
}
```

**响应**:
```protobuf
message TypeResponse {
    bool success = 1;  // 是否成功
}
```

### 填充表单

填充表单字段。

**方法**: `Fill`

**请求**:
```protobuf
message FillRequest {
    ElementId element_id = 1;  // 元素 ID
    string value = 2;          // 要填充的值
}
```

**响应**:
```protobuf
message FillResponse {
    bool success = 1;  // 是否成功
}
```

### 获取属性

获取元素的属性值。

**方法**: `GetAttribute` / `GetAttributes`

**请求**:
```protobuf
message GetAttributeRequest {
    ElementId element_id = 1;  // 元素 ID
    string name = 2;            // 属性名
}

message GetAttributesRequest {
    ElementId element_id = 1;  // 元素 ID
    repeated string names = 2;  // 属性名列表
}
```

**响应**:
```protobuf
message GetAttributeResponse {
    string value = 1;  // 属性值
}

message GetAttributesResponse {
    map<string, string> attributes = 1;  // 属性键值对
}
```

### 获取文本

获取元素的文本内容。

**方法**: `GetText`

**请求**:
```protobuf
message GetTextRequest {
    ElementId element_id = 1;  // 元素 ID
    bool inner_text = 2;       // 是否使用 innerText（否则用 textContent）
}
```

**响应**:
```protobuf
message GetTextResponse {
    string text = 1;  // 文本内容
}
```

### 鼠标悬停

将鼠标移动到元素上。

**方法**: `Hover`

**请求**:
```protobuf
message HoverRequest {
    ElementId element_id = 1;  // 元素 ID
    bool human_like = 2;       // 是否模拟人类移动
    int32 duration = 3;        // 移动持续时间（毫秒）
}
```

**响应**:
```protobuf
message HoverResponse {
    bool success = 1;  // 是否成功
}
```

### 选择选项

在下拉列表中选择选项。

**方法**: `SelectOption`

**请求**:
```protobuf
message SelectOptionRequest {
    ElementId element_id = 1;  // 元素 ID
    repeated string values = 2;  // 要选择的选项值
    bool multiple = 3;          // 是否支持多选
}
```

**响应**:
```protobuf
message SelectOptionResponse {
    bool success = 1;  // 是否成功
}
```

### 上传文件

上传文件到文件输入元素。

**方法**: `UploadFile`

**请求**:
```protobuf
message UploadFileRequest {
    ElementId element_id = 1;  // 元素 ID
    repeated string file_paths = 2;  // 文件路径列表
}
```

**响应**:
```protobuf
message UploadFileResponse {
    bool success = 1;  // 是否成功
}
```

### 滚动到元素

滚动页面直到元素可见。

**方法**: `ScrollIntoView`

**请求**:
```protobuf
message ScrollIntoViewRequest {
    ElementId element_id = 1;  // 元素 ID
    ScrollAlignment block = 2;  // 垂直对齐方式
    ScrollAlignment inline = 3; // 水平对齐方式
}

enum ScrollAlignment {
    SCROLL_ALIGNMENT_UNSPECIFIED = 0;
    SCROLL_ALIGNMENT_START = 1;    // 对齐到顶部/左侧
    SCROLL_ALIGNMENT_CENTER = 2;   // 居中对齐
    SCROLL_ALIGNMENT_END = 3;      // 对齐到底部/右侧
    SCROLL_ALIGNMENT_NEAREST = 4;  // 对齐到最近位置
}
```

**响应**:
```protobuf
message ScrollIntoViewResponse {
    bool success = 1;  // 是否成功
}
```

### 获取位置信息

获取元素的位置和尺寸。

**方法**: `GetBoundingBox`

**请求**:
```protobuf
message GetBoundingBoxRequest {
    ElementId element_id = 1;  // 元素 ID
}
```

**响应**:
```protobuf
message GetBoundingBoxResponse {
    BoundingBox box = 1;  // 位置信息
}

message BoundingBox {
    double x = 1;        // X 坐标
    double y = 2;        // Y 坐标
    double width = 3;    // 宽度
    double height = 4;   // 高度
}
```

### 检查可见性

检查元素是否可见。

**方法**: `IsVisible` / `IsEnabled`

**请求**:
```protobuf
message IsVisibleRequest {
    ElementId element_id = 1;  // 元素 ID
}

message IsEnabledRequest {
    ElementId element_id = 1;  // 元素 ID
}
```

**响应**:
```protobuf
message IsVisibleResponse {
    bool visible = 1;  // 是否可见
}

message IsEnabledResponse {
    bool enabled = 1;  // 是否可用
}
```

### 按键

在元素上按键。

**方法**: `PressKey`

**请求**:
```protobuf
message PressKeyRequest {
    ElementId element_id = 1;  // 元素 ID（可选，为空则在页面级别按键）
    string key = 2;            // 按键名称
    repeated string modifiers = 3;  // 修饰键（Control、Shift、Alt、Meta）
}
```

**响应**:
```protobuf
message PressKeyResponse {
    bool success = 1;  // 是否成功
}
```

### 拖拽

拖拽元素到目标位置。

**方法**: `DragAndDrop`

**请求**:
```protobuf
message DragAndDropRequest {
    ElementId from_element_id = 1;  // 源元素 ID
    ElementId to_element_id = 2;    // 目标元素 ID
    bool human_like = 3;             // 是否模拟人类拖拽
    int32 duration = 4;              // 拖拽持续时间（毫秒）
}
```

**响应**:
```protobuf
message DragAndDropResponse {
    bool success = 1;  // 是否成功
}
```

## EventService

事件流服务，提供实时事件推送。

### 订阅事件

订阅页面事件流。

**方法**: `Subscribe`（双向流）

**请求流**:
```protobuf
message SubscribeRequest {
    Action action = 1;  // 动作
    Subscription subscription = 2;  // 订阅信息
}

enum Action {
    ACTION_UNSPECIFIED = 0;
    ACTION_SUBSCRIBE = 1;    // 订阅
    ACTION_UNSUBSCRIBE = 2;  // 取消订阅
    ACTION_FILTER = 3;       // 过滤
}

message Subscription {
    PageId page_id = 1;                    // 页面 ID
    repeated EventType event_types = 2;    // 事件类型列表
    EventFilter filter = 3;                // 事件过滤器
}

enum EventType {
    EVENT_TYPE_UNSPECIFIED = 0;
    EVENT_TYPE_PAGE_CREATED = 1;
    EVENT_TYPE_PAGE_LOADED = 2;
    EVENT_TYPE_PAGE_NAVIGATED = 3;
    EVENT_TYPE_PAGE_CLOSED = 4;
    EVENT_TYPE_CONSOLE_LOG = 5;
    EVENT_TYPE_CONSOLE_ERROR = 6;
    EVENT_TYPE_REQUEST_SENT = 7;
    EVENT_TYPE_RESPONSE_RECEIVED = 8;
    EVENT_TYPE_JS_EXCEPTION = 9;
    EVENT_TYPE_DIALOG_OPENED = 10;
}

message EventFilter {
    string url_pattern = 1;              // URL 模式
    repeated string resource_types = 2;  // 资源类型
    int32 min_level = 3;                 // 最小日志级别
}
```

**响应流**:
```protobuf
message Event {
    EventMetadata metadata = 1;  // 事件元数据
    oneof payload {
        PageEvent page_event = 2;
        ConsoleEvent console_event = 3;
        NetworkEvent network_event = 4;
        DialogEvent dialog_event = 5;
    }
}

message EventMetadata {
    EventType type = 1;      // 事件类型
    int64 timestamp = 2;     // 时间戳
    PageId page_id = 3;      // 页面 ID
}

message PageEvent {
    string url = 1;          // URL
    string title = 2;        // 页面标题
}

message ConsoleEvent {
    string level = 1;        // 日志级别
    repeated string args = 2;  // 日志参数
    string location = 3;     // 位置信息
}

message NetworkEvent {
    string url = 1;          // URL
    string method = 2;       // HTTP 方法
    int32 status_code = 3;   // 状态码
    map<string, string> headers = 4;  // 响应头
    int64 size = 5;          // 响应大小
}

message DialogEvent {
    string dialog_type = 1;  // 对话框类型（alert、confirm、prompt）
    string message = 2;      // 消息内容
    string default_value = 3; // 默认值（prompt）
}
```

**使用示例**:
```python
# 订阅页面事件
def event_stream():
    # 发送订阅请求
    yield SubscribeRequest(
        action=Action.ACTION_SUBSCRIBE,
        subscription=Subscription(
            page_id=page_id,
            event_types=[
                EventType.EVENT_TYPE_PAGE_LOADED,
                EventType.EVENT_TYPE_CONSOLE_LOG,
                EventType.EVENT_TYPE_RESPONSE_RECEIVED
            ],
            filter=EventFilter(
                url_pattern="*example.com*",
                resource_types=["document", "xhr", "fetch"]
            )
        )
    )

    # 保持连接以接收事件
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        # 取消订阅
        yield SubscribeRequest(
            action=Action.ACTION_UNSUBSCRIBE
        )

# 接收事件流
for event in client.event.Subscribe(event_stream()):
    if event.metadata.type == EventType.EVENT_TYPE_PAGE_LOADED:
        print(f"Page loaded: {event.page_event.url}")
    elif event.metadata.type == EventType.EVENT_TYPE_CONSOLE_LOG:
        print(f"Console [{event.console_event.level}]: {' '.join(event.console_event.args)}")
    elif event.metadata.type == EventType.EVENT_TYPE_RESPONSE_RECEIVED:
        print(f"Response: {event.network_event.url} - {event.network_event.status_code}")
```

## ProfileService

隐身配置服务。

### 创建配置

创建新的指纹配置。

**方法**: `CreateProfile`

**请求**:
```protobuf
message CreateProfileRequest {
    ProfileType type = 1;  // 配置类型
}

enum ProfileType {
    PROFILE_TYPE_UNSPECIFIED = 0;
    PROFILE_TYPE_WINDOWS = 1;
    PROFILE_TYPE_LINUX = 2;
    PROFILE_TYPE_MACOS = 3;
    PROFILE_TYPE_ANDROID = 4;
    PROFILE_TYPE_IOS = 5;
    PROFILE_TYPE_CUSTOM = 6;
}
```

**响应**:
```protobuf
message CreateProfileResponse {
    Profile profile = 1;  // 创建的配置
}

message Profile {
    ProfileId profile_id = 1;  // 配置 ID
    ProfileType type = 2;      // 配置类型
    Fingerprint fingerprint = 3;  // 指纹信息
}

message ProfileId {
    string value = 1;
}

message Fingerprint {
    NavigatorInfo navigator = 1;      // Navigator 信息
    ScreenInfo screen = 2;            // 屏幕信息
    WebGLInfo webgl = 3;              // WebGL 信息
    CanvasInfo canvas = 4;            // Canvas 信息
    AudioInfo audio = 5;              // 音频信息
    TimeZoneInfo timezone = 6;        // 时区信息
    LocaleInfo locale = 7;            // 区域设置
    PermissionInfo permissions = 8;   // 权限信息
}
```

### 应用配置

将指纹配置应用到页面。

**方法**: `ApplyProfile`

**请求**:
```protobuf
message ApplyProfileRequest {
    PageId page_id = 1;     // 页面 ID
    ProfileId profile_id = 2;  // 配置 ID
}
```

**响应**:
```protobuf
message ApplyProfileResponse {
    bool success = 1;  // 是否成功
}
```

**使用示例**:
```python
# 创建 Windows 指纹配置
create_request = CreateProfileRequest(
    type=ProfileType.PROFILE_TYPE_WINDOWS
)
create_response = client.profile.CreateProfile(create_request)
profile_id = create_response.profile.profile_id

# 应用到页面
apply_request = ApplyProfileRequest(
    page_id=page_id,
    profile_id=profile_id
)
client.profile.ApplyProfile(apply_request)
print(f"Profile {profile_id} applied to page {page_id}")
```

### 获取预设配置

获取所有预定义的配置类型。

**方法**: `GetPresets`

**请求**:
```protobuf
message GetPresetsRequest {}
```

**响应**:
```protobuf
message GetPresetsResponse {
    repeated ProfilePreset presets = 1;  // 预设配置列表
}

message ProfilePreset {
    ProfileType type = 1;        // 配置类型
    string name = 2;             // 配置名称
    string description = 3;      // 配置描述
}
```

### 获取当前配置

获取页面当前应用的配置。

**方法**: `GetActiveProfile`

**请求**:
```protobuf
message GetActiveProfileRequest {
    PageId page_id = 1;  // 页面 ID
}
```

**响应**:
```protobuf
message GetActiveProfileResponse {
    Profile profile = 1;  // 当前配置（如果未应用则为 null）
}
```

### 创建自定义配置

创建自定义指纹配置。

**方法**: `CreateCustomProfile`

**请求**:
```protobuf
message CreateCustomProfileRequest {
    Fingerprint fingerprint = 1;  // 自定义指纹
}
```

**响应**:
```protobuf
message CreateCustomProfileResponse {
    Profile profile = 1;  // 创建的配置
}
```

**使用示例**:
```python
# 创建自定义配置
custom_fingerprint = Fingerprint(
    navigator=NavigatorInfo(
        user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        platform="Win32",
        hardware_concurrency=8,
        device_memory=8,
        vendor="Google Inc."
    ),
    screen=ScreenInfo(
        width=1920,
        height=1080,
        color_depth=24,
        pixel_ratio=1.0
    ),
    # ... 其他字段
)

request = CreateCustomProfileRequest(fingerprint=custom_fingerprint)
response = client.profile.CreateCustomProfile(request)
profile_id = response.profile.profile_id
```

### 随机化配置

随机化配置的某些参数。

**方法**: `RandomizeProfile`

**请求**:
```protobuf
message RandomizeProfileRequest {
    ProfileId profile_id = 1;  // 配置 ID
    repeated string fields = 2;  // 要随机化的字段（为空则随机化所有）
}
```

**响应**:
```protobuf
message RandomizeProfileResponse {
    Profile profile = 1;  // 随机化后的配置
}
```

## 错误处理

### 错误码

所有错误响应包含错误码和详细信息：

```protobuf
message Error {
    ErrorCode code = 1;              // 错误码
    string message = 2;              // 错误消息
    map<string, string> details = 3;  // 额外信息
}

enum ErrorCode {
    ERROR_CODE_UNSPECIFIED = 0;

    // 客户端错误 (4xx)
    ERROR_CODE_INVALID_ARGUMENT = 1;      // 无效参数
    ERROR_CODE_NOT_FOUND = 2;             // 资源未找到
    ERROR_CODE_ALREADY_EXISTS = 3;        // 资源已存在
    ERROR_CODE_PERMISSION_DENIED = 4;     // 权限不足
    ERROR_CODE_RESOURCE_EXHAUSTED = 5;    // 资源耗尽
    ERROR_CODE_ABORTED = 6;               // 操作被中止
    ERROR_CODE_OUT_OF_RANGE = 7;          // 超出范围
    ERROR_CODE_UNAUTHENTICATED = 8;       // 未认证
    ERROR_CODE_UNAVAILABLE = 9;           // 服务不可用

    // 特定错误
    ERROR_CODE_BROWSER_CLOSED = 10;       // 浏览器已关闭
    ERROR_CODE_PAGE_CLOSED = 11;          // 页面已关闭
    ERROR_CODE_ELEMENT_NOT_FOUND = 12;    // 元素未找到
    ERROR_CODE_NAVIGATION_FAILED = 13;    // 导航失败
    ERROR_CODE_EVALUATION_FAILED = 14;    // JavaScript 执行失败
    ERROR_CODE_TIMEOUT = 15;              // 超时
    ERROR_CODE_NETWORK_ERROR = 16;        // 网络错误
    ERROR_CODE_DIALOG_OPENED = 17;        // 对话框已打开
    ERROR_CODE_DOWNLOAD_STARTED = 18;     // 下载已开始
}
```

### 错误处理示例

```python
# Python 错误处理
try:
    response = client.page.Navigate(NavigateRequest(
        page_id=page_id,
        url="https://example.com"
    ))
except grpc.RpcError as e:
    if e.code() == grpc.StatusCode.NOT_FOUND:
        print(f"Page not found: {e.details()}")
    elif e.code() == grpc.StatusCode.DEADLINE_EXCEEDED:
        print("Navigation timeout")
    else:
        print(f"Navigation failed: {e.details()}")
```

## 最佳实践

### 1. 资源管理

始终在完成后关闭浏览器和页面：

```python
try:
    # 执行操作
    browser_id = launch_browser()
    page_id = create_page(browser_id)
    # ...
finally:
    # 清理资源
    client.page.ClosePage(ClosePageRequest(page_id=page_id))
    client.browser.Close(CloseRequest(browser_id=browser_id))
```

### 2. 超时设置

为所有操作设置合理的超时：

```python
request = NavigateRequest(
    page_id=page_id,
    url="https://example.com",
    options=NavigationOptions(
        wait_until=LoadState.LOAD_STATE_NETWORK_IDLE,
        timeout=30000  # 30 秒
    )
)
```

### 3. 等待策略

使用适当的等待状态：

```python
# 导航时等待网络空闲
NavigateRequest(
    options=NavigationOptions(
        wait_until=LoadState.LOAD_STATE_NETWORK_IDLE
    )
)

# 等待特定元素
WaitForRequest(
    selector=".loaded-content",
    state=WaitForState.WAIT_FOR_STATE_VISIBLE
)
```

### 4. 事件过滤

使用过滤器减少不必要的事件：

```python
subscription = Subscription(
    page_id=page_id,
    event_types=[
        EventType.EVENT_TYPE_PAGE_LOADED,
        EventType.EVENT_TYPE_RESPONSE_RECEIVED
    ],
    filter=EventFilter(
        url_pattern="*example.com*",
        resource_types=["document", "xhr", "fetch"]
    )
)
```

### 5. 错误重试

对暂时性错误实施重试：

```python
import time

def retry_operation(operation, max_retries=3):
    for attempt in range(max_retries):
        try:
            return operation()
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.UNAVAILABLE and attempt < max_retries - 1:
                time.sleep(2 ** attempt)  # 指数退避
                continue
            raise
```

### 6. 并发操作

利用异步特性执行并发操作：

```python
import asyncio

async def concurrent_operations():
    # 并发创建多个页面
    tasks = [
        client.page.CreatePage(CreatePageRequest(browser_id=browser_id))
        for _ in range(5)
    ]
    pages = await asyncio.gather(*tasks)
    return pages
```

### 7. 隐身配置

在生产环境中使用隐身配置：

```python
# 创建随机配置
profile = client.profile.CreateProfile(CreateProfileRequest(
    type=ProfileType.PROFILE_TYPE_WINDOWS
)).profile

# 应用到所有新页面
client.profile.ApplyProfile(ApplyProfileRequest(
    page_id=page_id,
    profile_id=profile.profile_id
))
```

## 相关文档

- [README.md](README.md) - 项目介绍
- [DEPLOYMENT.md](DEPLOYMENT.md) - 部署指南
- [DEVELOPMENT.md](DEVELOPMENT.md) - 开发指南
- [docs/api-design.md](docs/api-design.md) - API 设计文档
