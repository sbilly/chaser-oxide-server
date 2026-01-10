# Chaser-Oxide gRPC API 设计文档

## 概述

Chaser-Oxide gRPC API 为浏览器自动化功能提供网络服务接口，支持 Python 和 Go 客户端调用。API 设计遵循 gRPC 最佳实践，提供类型安全的接口定义和高效的数据传输。

## 核心特性

1. **会话隔离** - 每个页面有独立的会话 ID，支持并发操作
2. **实时事件** - 双向流式传输实现事件推送
3. **隐身能力** - 完整的指纹配置和人类行为模拟
4. **类型安全** - Protocol Buffers 提供强类型接口定义

## 服务架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Client (Python/Go)                      │
└──────────────────────────┬──────────────────────────────────┘
                           │ gRPC over HTTP/2
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                 Chaser-Oxide gRPC Server                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Browser     │  │ Page        │  │ Element     │         │
│  │ Service     │  │ Service     │  │ Service     │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐         │
│  │ Profile     │  │ Event       │  │             │         │
│  │ Service     │  │ Service     │  │             │         │
│  └─────────────┘  └──────┬──────┘  └─────────────┘         │
│                          │                                   │
│  ┌───────────────────────┴─────────────────────────────┐    │
│  │              Chaser-Oxide Core Library               │    │
│  └───────────────────────┬─────────────────────────────┘    │
└──────────────────────────┼──────────────────────────────────┘
                           │
                           ▼
                   ┌───────────────┐
                   │ Chrome/Chromium│
                   └───────────────┘
```

## 服务定义

### 1. BrowserService

浏览器生命周期管理服务，负责启动、管理和关闭浏览器实例。

#### 方法列表

| 方法 | 请求 | 响应 | 描述 |
|------|------|------|------|
| `Launch` | `LaunchRequest` | `LaunchResponse` | 启动新浏览器实例 |
| `GetPages` | `GetPagesRequest` | `GetPagesResponse` | 获取所有页面列表 |
| `Close` | `CloseRequest` | `CloseResponse` | 关闭浏览器实例 |
| `GetVersion` | `GetVersionRequest` | `GetVersionResponse` | 获取浏览器版本信息 |
| `GetStatus` | `GetStatusRequest` | `GetStatusResponse` | 获取浏览器状态 |
| `Connect` | `ConnectRequest` | `ConnectResponse` | 连接到现有浏览器 |

#### 使用示例

```python
# 启动浏览器
request = LaunchRequest(
    options=BrowserOptions(
        headless=True,
        window_width=1920,
        window_height=1080
    )
)
response = client.browser.Launch(request)
browser_id = response.browser_info.browser_id
```

### 2. PageService

页面操作服务，提供导航、内容访问、JavaScript 执行等功能。

#### 方法列表

| 方法 | 请求 | 响应 | 描述 |
|------|------|------|------|
| `CreatePage` | `CreatePageRequest` | `CreatePageResponse` | 创建新页面 |
| `Navigate` | `NavigateRequest` | `NavigateResponse` | 导航到 URL |
| `GetSnapshot` | `GetSnapshotRequest` | `GetSnapshotResponse` | 获取页面快照 |
| `Screenshot` | `ScreenshotRequest` | `ScreenshotResponse` | 截图 |
| `Evaluate` | `EvaluateRequest` | `EvaluateResponse` | 执行 JavaScript |
| `SetContent` | `SetContentRequest` | `SetContentResponse` | 设置页面内容 |
| `GetContent` | `GetContentRequest` | `GetContentResponse` | 获取页面内容 |
| `Reload` | `ReloadRequest` | `ReloadResponse` | 刷新页面 |
| `GoBack` | `GoBackRequest` | `GoBackResponse` | 后退 |
| `GoForward` | `GoForwardRequest` | `GoForwardResponse` | 前进 |
| `SetViewport` | `SetViewportRequest` | `SetViewportResponse` | 设置视口大小 |
| `EmulateDevice` | `EmulateDeviceRequest` | `EmulateDeviceResponse` | 模拟设备 |
| `ClosePage` | `ClosePageRequest` | `ClosePageResponse` | 关闭页面 |
| `WaitFor` | `WaitForRequest` | `WaitForResponse` | 等待条件 |
| `GetPDF` | `GetPDFRequest` | `GetPDFResponse` | 获取 PDF |
| `AddInitScript` | `AddInitScriptRequest` | `AddInitScriptResponse` | 添加初始化脚本 |
| `OverridePermissions` | `OverridePermissionsRequest` | `OverridePermissionsResponse` | 覆盖权限 |
| `SetGeolocation` | `SetGeolocationRequest` | `SetGeolocationResponse` | 设置地理位置 |
| `SetOfflineMode` | `SetOfflineModeRequest` | `SetOfflineModeResponse` | 设置离线模式 |
| `SetCacheEnabled` | `SetCacheEnabledRequest` | `SetCacheEnabledResponse` | 设置缓存 |
| `GetCookies` | `GetCookiesRequest` | `GetCookiesResponse` | 获取 Cookie |
| `SetCookies` | `SetCookiesRequest` | `SetCookiesResponse` | 设置 Cookie |
| `ClearCookies` | `ClearCookiesRequest` | `ClearCookiesResponse` | 清除 Cookie |

#### 使用示例

```python
# 导航到 URL
request = NavigateRequest(
    page_id=page_id,
    url="https://example.com",
    options=NavigationOptions(
        wait_until=LoadState.LOAD_STATE_NETWORK_IDLE,
        timeout=30000
    )
)
response = client.page.Navigate(request)
```

### 3. ElementService

元素交互服务，提供元素查找和交互功能。

#### 方法列表

| 方法 | 请求 | 响应 | 描述 |
|------|------|------|------|
| `FindElement` | `FindElementRequest` | `FindElementResponse` | 查找单个元素 |
| `FindElements` | `FindElementsRequest` | `FindElementsResponse` | 查找多个元素 |
| `Click` | `ClickRequest` | `ClickResponse` | 点击元素 |
| `Type` | `TypeRequest` | `TypeResponse` | 输入文本 |
| `Fill` | `FillRequest` | `FillResponse` | 填充表单 |
| `GetAttribute` | `GetAttributeRequest` | `GetAttributeResponse` | 获取属性 |
| `GetAttributes` | `GetAttributesRequest` | `GetAttributesResponse` | 获取多个属性 |
| `GetText` | `GetTextRequest` | `GetTextResponse` | 获取文本 |
| `GetHTML` | `GetHTMLRequest` | `GetHTMLResponse` | 获取 HTML |
| `Hover` | `HoverRequest` | `HoverResponse` | 鼠标悬停 |
| `Focus` | `FocusRequest` | `FocusResponse` | 聚焦元素 |
| `SelectOption` | `SelectOptionRequest` | `SelectOptionResponse` | 选择选项 |
| `UploadFile` | `UploadFileRequest` | `UploadFileResponse` | 上传文件 |
| `ScrollIntoView` | `ScrollIntoViewRequest` | `ScrollIntoViewResponse` | 滚动到元素 |
| `GetBoundingBox` | `GetBoundingBoxRequest` | `GetBoundingBoxResponse` | 获取位置信息 |
| `IsVisible` | `IsVisibleRequest` | `IsVisibleResponse` | 检查可见性 |
| `IsEnabled` | `IsEnabledRequest` | `IsEnabledResponse` | 检查是否可用 |
| `WaitForElement` | `WaitForElementRequest` | `WaitForElementResponse` | 等待元素 |
| `GetProperties` | `GetPropertiesRequest` | `GetPropertiesResponse` | 获取属性 |
| `PressKey` | `PressKeyRequest` | `PressKeyResponse` | 按键 |
| `DragAndDrop` | `DragAndDropRequest` | `DragAndDropResponse` | 拖拽 |

#### 使用示例

```python
# 查找并点击元素
find_request = FindElementRequest(
    page_id=page_id,
    selector_type=SelectorType.SELECTOR_TYPE_CSS,
    selector="#submit-button"
)
element = client.element.FindElement(find_request).element

click_request = ClickRequest(
    element=element,
    human_like=True,
    movement_duration=200
)
client.element.Click(click_request)
```

### 4. ProfileService

隐身配置服务，提供指纹管理和反检测功能。

#### 方法列表

| 方法 | 请求 | 响应 | 描述 |
|------|------|------|------|
| `CreateProfile` | `CreateProfileRequest` | `CreateProfileResponse` | 创建指纹配置 |
| `ApplyProfile` | `ApplyProfileRequest` | `ApplyProfileResponse` | 应用配置 |
| `GetPresets` | `GetPresetsRequest` | `GetPresetsResponse` | 获取预定义配置 |
| `GetActiveProfile` | `GetActiveProfileRequest` | `GetActiveProfileResponse` | 获取当前配置 |
| `CreateCustomProfile` | `CreateCustomProfileRequest` | `CreateCustomProfileResponse` | 创建自定义配置 |
| `RandomizeProfile` | `RandomizeProfileRequest` | `RandomizeProfileResponse` | 随机化配置 |

#### 预定义配置类型

- `WINDOWS` - Windows 指纹
- `LINUX` - Linux 指纹
- `MACOS` - macOS 指纹
- `ANDROID` - Android 移动设备指纹
- `IOS` - iOS 移动设备指纹
- `CUSTOM` - 自定义指纹

#### 使用示例

```python
# 创建并应用 Windows 指纹配置
create_request = CreateProfileRequest(
    type=ProfileType.PROFILE_TYPE_WINDOWS
)
profile = client.profile.CreateProfile(create_request).profile

apply_request = ApplyProfileRequest(
    page_id=page_id,
    profile_id=profile.profile_id
)
client.profile.ApplyProfile(apply_request)
```

### 5. EventService

事件流服务，提供实时事件推送功能。

#### 方法列表

| 方法 | 请求 | 响应 | 描述 |
|------|------|------|------|
| `Subscribe` | `stream SubscribeRequest` | `stream Event` | 订阅事件流（双向流） |

#### 事件类型

| 类型 | 描述 |
|------|------|
| `PAGE_CREATED` | 页面创建 |
| `PAGE_LOADED` | 页面加载完成 |
| `PAGE_NAVIGATED` | 页面导航 |
| `PAGE_CLOSED` | 页面关闭 |
| `CONSOLE_LOG` | 控制台日志 |
| `CONSOLE_ERROR` | 控制台错误 |
| `REQUEST_SENT` | 网络请求发送 |
| `RESPONSE_RECEIVED` | 网络响应接收 |
| `JS_EXCEPTION` | JavaScript 异常 |
| `DIALOG_OPENED` | 对话框打开 |

#### 使用示例

```python
# 订阅页面事件
def event_stream():
    yield SubscribeRequest(
        action=Action.ACTION_SUBSCRIBE,
        subscription=Subscription(
            page_id=page_id,
            event_types=[
                EventType.EVENT_TYPE_PAGE_LOADED,
                EventType.EVENT_TYPE_CONSOLE_LOG,
                EventType.EVENT_TYPE_RESPONSE_RECEIVED
            ]
        )
    )

for event in client.event.Subscribe(event_stream()):
    if event.metadata.type == EventType.EVENT_TYPE_PAGE_LOADED:
        print(f"Page loaded: {event.page_event.url}")
    elif event.metadata.type == EventType.EVENT_TYPE_CONSOLE_LOG:
        print(f"Console: {event.console_event.args}")
```

## 数据模型

### 会话标识

```protobuf
message SessionID {
    string browser_id = 1;  // 浏览器实例 UUID
    string page_id = 2;     // 页面 UUID
}
```

### 错误处理

```protobuf
message Error {
    ErrorCode code = 1;     // 错误码
    string message = 2;     // 错误消息
    map<string, string> details = 3;  // 额外信息
}
```

所有响应都使用 `oneof` 包装，确保类型安全：

```protobuf
message Response {
    oneof result {
        SuccessType success = 1;
        Error error = 2;
    }
}
```

## 最佳实践

### 1. 资源管理

始终在完成后关闭浏览器和页面：

```python
try:
    # 执行操作
    pass
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
        timeout=30000  # 30 秒超时
    )
)
```

### 3. 错误处理

始终检查响应中的错误：

```python
response = client.page.Navigate(request)
if response.HasField('error'):
    if response.error.code == ErrorCode.ERROR_CODE_NAVIGATION_FAILED:
        print("导航失败")
    # 处理其他错误
```

### 4. 事件订阅

使用适当的过滤器减少不必要的事件：

```python
subscription = Subscription(
    page_id=page_id,
    event_types=[
        EventType.EVENT_TYPE_PAGE_LOADED,
        EventType.EVENT_TYPE_REQUEST_SENT
    ],
    filter=EventFilter(
        url_pattern="*example.com*",
        resource_types=["document", "xhr"]
    )
)
```

## 扩展性

API 设计考虑了未来扩展：

1. **新服务** - 可以添加新的服务而不影响现有服务
2. **新方法** - 可以添加新方法而不破坏现有客户端
3. **新字段** - Proto3 允许安全添加新字段
4. **事件类型** - 可以添加新的事件类型

## 性能考虑

1. **复用连接** - 保持 gRPC 连接长时间存活
2. **批量操作** - 对于多个操作，使用批量 API
3. **流式传输** - 对于大量数据，使用流式 API
4. **事件过滤** - 在服务端过滤事件以减少网络传输

## 安全考虑

1. **认证** - 生产环境应启用 gRPC 认证（TLS/SSL）
2. **授权** - 实施适当的访问控制
3. **资源限制** - 限制并发浏览器和页面数量
4. **隔离** - 使用沙箱环境运行浏览器
