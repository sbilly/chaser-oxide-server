# Chaser-Oxide 服务器端实施计划

## 📋 项目概述

**项目名称**: Chaser-Oxide gRPC 服务器

**项目描述**: 基于 Rust 的浏览器自动化微服务，通过 gRPC 协议提供对 Chrome DevTools Protocol (CDP) 的抽象访问，支持高隐身和反检测能力。

**目标**: 实现一个高性能、类型安全、支持隐身浏览的浏览器自动化服务器。

---

## 📊 当前状态

### ✅ 已完成

| 组件 | 状态 | 说明 |
|------|------|------|
| gRPC 服务定义 | ✅ 完成 | 6 个 proto 文件，定义完整 API |
| 架构设计文档 | ✅ 完成 | 详细的系统架构和组件设计 |
| API 设计文档 | ✅ 完成 | API 接口规范和使用指南 |
| Python 客户端示例 | ✅ 完成 | 基础和隐身功能示例代码 |

### ❌ 待实现

| 组件 | 优先级 | 工作量估算 |
|------|--------|-----------|
| Rust 项目配置 | 🔴 高 | 1 天 |
| gRPC 服务器框架 | 🔴 高 | 3 天 |
| CDP 核心库 | 🔴 高 | 5 天 |
| BrowserService | 🔴 高 | 3 天 |
| PageService | 🔴 高 | 5 天 |
| ElementService | 🟡 中 | 4 天 |
| EventService | 🟡 中 | 3 天 |
| ProfileService | 🟡 中 | 4 天 |
| 隐身引擎 | 🟢 低 | 5 天 |
| 测试套件 | 🟡 中 | 5 天 |
| 部署配置 | 🟢 低 | 2 天 |

**总工作量估算**: 约 40 个工作日

---

## 🛣️ 实施路线图

### 阶段 1: 基础设施搭建 (1-2 周)

**目标**: 建立项目基础架构，实现基本的 gRPC 服务能力。

**交付物**:
- [x] Cargo 项目配置
- [x] Proto 代码生成脚本
- [x] 基础 gRPC 服务器框架
- [x] 日志和错误处理系统
- [x] 配置管理系统

**关键文件**:
```
chaser-oxide-server/
├── Cargo.toml                    # Rust 项目配置
├── build.rs                      # 构建脚本
├── config/
│   └── default.toml              # 默认配置
├── src/
│   ├── main.rs                   # 服务入口
│   ├── lib.rs                    # 库入口
│   ├── error.rs                  # 错误类型定义
│   ├── config.rs                 # 配置管理
│   └── proto/
│       ├── mod.rs                # Proto 模块
│       ├── common.rs             # 生成的 common.proto
│       ├── browser.rs            # 生成的 browser.proto
│       ├── page.rs               # 生成的 page.proto
│       ├── element.rs            # 生成的 element.proto
│       ├── profile.rs            # 生成的 profile.proto
│       └── event.rs              # 生成的 event.proto
└── build/
    └── gen_proto.sh              # Proto 生成脚本
```

**技术栈**:
- `tonic` v0.12 - gRPC 框架
- `prost` v0.13 - Protocol Buffers
- `tokio` v1 - 异步运行时
- `tower` v0.5 - 中间件
- `tracing` v0.1 - 日志和追踪
- `serde` v1 - 序列化
- `config` v0.14 - 配置管理

### 阶段 2: CDP 核心库 (2-3 周)

**目标**: 实现与 Chrome DevTools Protocol 的通信层。

**交付物**:
- [x] CDP WebSocket 连接管理
- [x] CDP 命令发送和响应处理
- [x] CDP 事件监听和分发
- [x] 浏览器进程管理
- [x] 页面生命周期管理

**关键文件**:
```
src/cdp/
├── mod.rs                        # CDP 模块入口
├── connection.rs                 # WebSocket 连接
│   └── struct CdpConnection
│       ├── connect()             # 建立 WebSocket 连接
│       ├── send_command()        # 发送 CDP 命令
│       ├── listen_events()       # 监听 CDP 事件
│       └── close()               # 关闭连接
├── client.rs                     # CDP 客户端
│   └── struct CdpClient
│       ├── new()                 # 创建客户端
│       ├── call_method()         # 调用 CDP 方法
│       ├── on_event()            # 注册事件处理器
│       └── wait_for_event()      # 等待特定事件
├── browser.rs                    # 浏览器控制
│   └── struct Browser
│       ├── launch()              # 启动浏览器进程
│       ├── connect()             # 连接到浏览器
│       ├── close()               # 关闭浏览器
│       ├── get_version()         # 获取版本信息
│       └── get_pages()           # 获取所有页面
├── page.rs                       # 页面控制
│   └── struct Page
│       ├── create()              # 创建新页面
│       ├── navigate()            # 导航到 URL
│       ├── evaluate()            # 执行 JavaScript
│       ├── screenshot()          # 截图
│       ├── close()               # 关闭页面
│       └── on_event()            # 注册页面事件
└── types.rs                      # CDP 类型定义
    ├── mod Target                # Target 域
    ├── mod Page                  # Page 域
    ├── mod Runtime               # Runtime 域
    ├── mod DOM                   # DOM 域
    └── mod Network               # Network 域
```

**实现要点**:
1. 使用 `tokio-tungstenite` 实现 WebSocket 连接
2. 使用 `serde_json` 处理 CDP 消息的 JSON 格式
3. 实现命令 ID 追踪和响应匹配
4. 实现事件订阅和分发机制
5. 支持并发命令执行

### 阶段 3: 服务层实现 (4-5 周)

**目标**: 实现 5 个 gRPC 服务及其业务逻辑。

#### 3.1 BrowserService (3 天)

```
src/services/browser/
├── mod.rs
├── service.rs                    # BrowserService 实现
│   └── impl BrowserService for BrowserServiceImpl
│       ├── launch()              # 启动浏览器
│       ├── close()               # 关闭浏览器
│       ├── connect()             # 连接到现有浏览器
│       ├── get_version()         # 获取版本
│       ├── get_status()          # 获取状态
│       └── get_pages()           # 获取页面列表
└── options.rs                    # 浏览器选项处理
    └── struct BrowserLauncher
        ├── build_command()       # 构建启动命令
        ├── parse_args()          # 解析启动参数
        └── validate()            # 验证选项
```

#### 3.2 PageService (5 天)

```
src/services/page/
├── mod.rs
├── service.rs                    # PageService 实现
│   └── impl PageService for PageServiceImpl
│       ├── create_page()         # 创建页面
│       ├── navigate()            # 导航
│       ├── get_snapshot()        # 获取快照
│       ├── screenshot()          # 截图
│       ├── evaluate()            # 执行 JS
│       ├── set_content()         # 设置内容
│       ├── get_content()         # 获取内容
│       ├── reload()              # 刷新
│       ├── go_back()             # 后退
│       ├── go_forward()          # 前进
│       ├── set_viewport()        # 设置视口
│       ├── emulate_device()      # 模拟设备
│       ├── close_page()          # 关闭页面
│       ├── wait_for()            # 等待条件
│       ├── get_pdf()             # 生成 PDF
│       ├── add_init_script()     # 添加初始化脚本
│       ├── override_permissions() # 覆盖权限
│       ├── set_geolocation()     # 设置地理位置
│       ├── set_offline_mode()    # 设置离线模式
│       ├── set_cache_enabled()   # 设置缓存
│       ├── get_cookies()         # 获取 Cookie
│       ├── set_cookies()         # 设置 Cookie
│       └── clear_cookies()       # 清除 Cookie
├── navigator.rs                  # 导航控制器
│   └── struct Navigator
│       ├── navigate()            # 执行导航
│       ├── wait_for_load()       # 等待加载完成
│       └── handle_navigation_events() # 处理导航事件
├── screenshot.rs                 # 截图控制器
│   └── struct ScreenshotTaker
│       ├── capture()             # 截取页面
│       ├── optimize()            # 优化图片
│       └── encode()              # 编码格式
└── script.rs                     # 脚本执行器
    └── struct ScriptEvaluator
        ├── evaluate()            # 执行脚本
        ├── await_promise()       # 等待 Promise
        └── handle_exception()    # 处理异常
```

#### 3.3 ElementService (4 天)

```
src/services/element/
├── mod.rs
├── service.rs                    # ElementService 实现
│   └── impl ElementService for ElementServiceImpl
│       ├── find_element()        # 查找单个元素
│       ├── find_elements()       # 查找多个元素
│       ├── click()               # 点击
│       ├── type()                # 输入文本
│       ├── fill()                # 填充表单
│       ├── get_attribute()       # 获取属性
│       ├── get_attributes()      # 获取多个属性
│       ├── get_text()            # 获取文本
│       ├── get_html()            # 获取 HTML
│       ├── hover()               # 悬停
│       ├── focus()               # 聚焦
│       ├── select_option()       # 选择选项
│       ├── upload_file()         # 上传文件
│       ├── scroll_into_view()    # 滚动到元素
│       ├── get_bounding_box()    # 获取位置
│       ├── is_visible()          # 检查可见性
│       ├── is_enabled()          # 检查是否可用
│       ├── wait_for_element()    # 等待元素
│       ├── get_properties()      # 获取属性
│       ├── press_key()           # 按键
│       └── drag_and_drop()       # 拖拽
├── finder.rs                     # 元素查找器
│   └── struct ElementFinder
│       ├── find_by_css()         # CSS 选择器
│       ├── find_by_xpath()       # XPath 选择器
│       ├── find_by_text()        # 文本查找
│       ├── wait_for()            # 等待元素出现
│       └── handle_stale()        # 处理过期元素
├── interactor.rs                 # 元素交互器
│   └── struct ElementInteractor
│       ├── click()               # 点击元素
│       ├── type_text()           # 输入文本
│       ├── hover()               # 悬停
│       ├── drag()                # 拖拽
│       └── scroll()              # 滚动
└── reference.rs                  # 元素引用
    └── struct ElementRef
        ├── backend_id            # 后端节点 ID
        ├── protocol_id           # Protocol ID
        └── is_stale()            # 检查是否过期
```

#### 3.4 ProfileService (4 天)

```
src/services/profile/
├── mod.rs
├── service.rs                    # ProfileService 实现
│   └── impl ProfileService for ProfileServiceImpl
│       ├── create_profile()      # 创建配置
│       ├── apply_profile()       # 应用配置
│       ├── get_presets()         # 获取预定义配置
│       ├── get_active_profile()  # 获取当前配置
│       ├── create_custom_profile() # 创建自定义配置
│       └── randomize_profile()   # 随机化配置
├── profile.rs                    # 配置管理
│   └── struct ProfileManager
│       ├── create()              # 创建新配置
│       ├── get()                 # 获取配置
│       ├── apply()               # 应用到页面
│       ├── randomize()           # 随机化
│       └── get_presets()         # 获取预定义
├── fingerprint.rs                # 指纹生成
│   └── struct FingerprintGenerator
│       ├── generate_windows()    # 生成 Windows 指纹
│       ├── generate_macos()      # 生成 macOS 指纹
│       ├── generate_linux()      # 生成 Linux 指纹
│       ├── generate_android()    # 生成 Android 指纹
│       └── generate_ios()        # 生成 iOS 指纹
├── presets.rs                    # 预定义配置
│   └── lazy_static! {
│           static ref WINDOWS_PRESETS: Vec<Profile>
│           static ref MACOS_PRESETS: Vec<Profile>
│           static ref LINUX_PRESETS: Vec<Profile>
│           static ref ANDROID_PRESETS: Vec<Profile>
│           static ref IOS_PRESETS: Vec<Profile>
│       }
└── randomizer.rs                 # 随机化工具
    └── struct ProfileRandomizer
        ├── randomize_screen()    # 随机化屏幕
        ├── randomize_timezone()  # 随机化时区
        ├── randomize_language()  # 随机化语言
        └── randomize_webgl()     # 随机化 WebGL
```

#### 3.5 EventService (3 天)

```
src/services/event/
├── mod.rs
├── service.rs                    # EventService 实现
│   └── impl EventService for EventServiceImpl
│       └── subscribe()           # 事件订阅（双向流）
├── dispatcher.rs                 # 事件分发器
│   └── struct EventDispatcher
│       ├── subscribe()           # 订阅事件
│       ├── unsubscribe()         # 取消订阅
│       ├── publish()             # 发布事件
│       ├── add_filter()          # 添加过滤器
│       └── cleanup()             # 清理订阅
├── subscription.rs               # 订阅管理
│   └── struct Subscription
│       ├── id                    # 订阅 ID
│       ├── page_id               # 页面 ID
│       ├── event_types           # 事件类型列表
│       ├── filter                # 过滤条件
│       └── tx                    # 事件发送器
└── converter.rs                  # 事件转换器
    └── struct EventConverter
        ├── cdp_to_grpc()         # CDP 事件转 gRPC
        ├── page_loaded()         # 页面加载事件
        ├── console_log()         # 控制台日志事件
        ├── network_event()       # 网络事件
        └── dialog_event()        # 对话框事件
```

### 阶段 4: 隐身引擎 (3-4 周)

**目标**: 实现高级反检测和人类行为模拟功能。

**交付物**:
- [x] 指纹注入系统
- [x] Navigator 属性覆盖
- [x] WebGL/Canvas 指纹保护
- [x] 人类行为模拟器
- [x] 贝塞尔曲线鼠标移动
- [x] 人类打字模式

**关键文件**:
```
src/stealth/
├── mod.rs
├── engine.rs                     # 隐身引擎
│   └── struct StealthEngine
│       ├── apply_profile()       # 应用隐身配置
│       ├── inject_navigator()    # 注入 Navigator 属性
│       ├── inject_webgl()        # 注入 WebGL 指纹
│       ├── inject_canvas()       # 注入 Canvas 保护
│       └── configure_transport() # 配置传输层
├── injector.rs                   # 脚本注入器
│   └── struct ScriptInjector
│       ├── inject()              # 注入脚本
│       ├── create_isolated_world() # 创建隔离世界
│       └── evaluate_on_new_document() # 文档加载前执行
├── navigator.rs                  # Navigator 注入
│   └── struct NavigatorInjector
│       ├── inject_platform()     # 注入 platform
│       ├── inject_hardware()     # 注入硬件信息
│       ├── inject_vendor()       # 注入 vendor
│       └── inject_languages()    # 注入语言
├── webgl.rs                      # WebGL 保护
│   └── struct WebGLProtector
│       ├── spoof_vendor()        # 伪装 vendor
│       ├── spoof_renderer()      # 伪装 renderer
│       └── add_noise()           # 添加噪声
├── canvas.rs                     # Canvas 保护
│   └── struct CanvasProtector
│       ├── add_noise()           # 添加噪声
│       ├── randomize_curve()     # 随机化曲线
│       └── protect_fingerprint() # 保护指纹
├── behavior.rs                   # 行为模拟
│   └── struct BehaviorSimulator
│       ├── simulate_mouse_move() # 模拟鼠标移动
│       ├── simulate_typing()     # 模拟打字
│       ├── simulate_scroll()     # 模拟滚动
│       └── randomize_timing()    # 随机化时序
└── bezier.rs                     # 贝塞尔曲线
    └── struct BezierGenerator
        ├── generate_curve()      # 生成曲线
        ├── calculate_point()     # 计算点位置
        └── randomize_control()   # 随机化控制点
```

**实现要点**:
1. 使用 `Page.addScriptToEvaluateOnNewDocument` 在文档加载前注入
2. 使用 `Page.createIsolatedWorld` 创建隔离上下文
3. 实现 `Object.defineProperty` 覆盖只读属性
4. 使用贝塞尔曲线生成自然的鼠标轨迹
5. 添加随机延迟模拟人类反应时间

### 阶段 5: 会话管理器 (1 周)

**目标**: 实现浏览器、页面和元素的生命周期管理。

**关键文件**:
```
src/session/
├── mod.rs
├── manager.rs                    # 会话管理器
│   └── struct SessionManager
│       ├── browsers              # 浏览器实例映射
│       ├── pages                 # 页面实例映射
│       ├── elements              # 元素引用映射
│       ├── create_browser()      # 创建浏览器
│       ├── get_browser()         # 获取浏览器
│       ├── close_browser()       # 关闭浏览器
│       ├── create_page()         # 创建页面
│       ├── get_page()            # 获取页面
│       ├── close_page()          # 关闭页面
│       └── cleanup()             # 清理资源
├── browser.rs                    # 浏览器上下文
│   └── struct BrowserContext
│       ├── id                    # 浏览器 ID
│       ├── process               # 子进程句柄
│       ├── cdp_client            # CDP 客户端
│       ├── pages                 # 页面列表
│       └── options               # 启动选项
├── page.rs                       # 页面上下文
│   └── struct PageContext
│       ├── id                    # 页面 ID
│       ├── browser_id            # 所属浏览器
│       ├── target_id             # CDP Target ID
│       ├── profile_id            # 当前配置
│       └── subscriptions         # 事件订阅
└── element.rs                    # 元素引用
    └-> struct ElementRef
        ├── id                    # 元素 ID
        ├── page_id               # 所属页面
        ├── backend_node_id       # 后端节点 ID
        └── is_stale()            # 检查是否过期
```

### 阶段 6: 测试和验证 (2 周)

**目标**: 确保代码质量和功能正确性。

**测试文件结构**:
```
tests/
├── integration/
│   ├── browser_test.rs           # 浏览器服务测试
│   ├── page_test.rs              # 页面服务测试
│   ├── element_test.rs           # 元素服务测试
│   ├── profile_test.rs           # 配置服务测试
│   └── event_test.rs             # 事件服务测试
├── unit/
│   ├── cdp_test.rs               # CDP 客户端测试
│   ├── session_test.rs           # 会话管理测试
│   └── stealth_test.rs           # 隐身引擎测试
└── e2e/
    └── full_workflow_test.rs     # 端到端测试
```

**测试覆盖率目标**:
- 单元测试: ≥ 80%
- 集成测试: ≥ 70%
- 端到端测试: 核心流程 100%

### 阶段 7: 部署和运维 (1 周)

**目标**: 实现部署配置和监控。

**交付物**:
- [x] Docker 镜像
- [x] Docker Compose 配置
- [x] Prometheus 监控
- [x] 日志配置
- [x] 启动脚本

**关键文件**:
```
docker/
├── Dockerfile                    # Docker 镜像
└── docker-compose.yml            # Compose 配置

scripts/
├── start.sh                      # 启动脚本
├── stop.sh                       # 停止脚本
└── build.sh                      # 构建脚本

monitoring/
├── prometheus.yml                # Prometheus 配置
└── grafana/dashboards/           # Grafana 仪表板
```

---

## 🔧 技术栈

### 核心依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tonic` | 0.12 | gRPC 框架 |
| `prost` | 0.13 | Protocol Buffers |
| `tokio` | 1.0 | 异步运行时 |
| `tower` | 0.5 | 中间件 |
| `tower-http` | 0.5 | HTTP 中间件 |
| `tracing` | 0.1 | 日志和追踪 |
| `tracing-subscriber` | 0.3 | 日志订阅器 |
| `serde` | 1.0 | 序列化 |
| `serde_json` | 1.0 | JSON 支持 |
| `config` | 0.14 | 配置管理 |
| `uuid` | 1.0 | UUID 生成 |
| `bytes` | 1.0 | 字节缓冲 |
| `async-trait` | 0.1 | 异步 trait |
| ` anyhow` | 1.0 | 错误处理 |
| `thiserror` | 1.0 | 错误定义 |

### CDP 相关

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tokio-tungstenite` | 0.21 | WebSocket 客户端 |
| `base64` | 0.21 | Base64 编码 |
| `url` | 2.5 | URL 解析 |

### 隐身功能

| 依赖 | 版本 | 用途 |
|------|------|------|
| `rand` | 0.8 | 随机数生成 |
| `rand_chacha` | 0.3 | ChaCha 随机数 |
| `fake` | 2.9 | 假数据生成 |

### 监控和测试

| 依赖 | 版本 | 用途 |
|------|------|------|
| `prometheus` | 0.13 | 指标收集 |
| `tracing-opentelemetry` | 0.22 | OpenTelemetry 集成 |
| `tokio-test` | 0.4 | 测试工具 |

### 开发依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tonic-build` | 0.12 | Proto 代码生成 |
| `prost-build` | 0.13 | Proto 构建 |
| `cargo-watch` | 8.4 | 文件监控 |
| `criterion` | 0.5 | 性能测试 |

---

## 📝 开发规范

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 命名规范：
  - 结构体: `PascalCase`
  - 函数: `snake_case`
  - 常量: `SCREAMING_SNAKE_CASE`
  - 宏: `snake_case!`

### 错误处理

```rust
// 定义错误类型
#[derive(thiserror::Error, Debug)]
pub enum ChaserError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    #[error("CDP error: {0}")]
    CdpError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// 转换为 gRPC Status
impl From<ChaserError> for tonic::Status {
    fn from(err: ChaserError) -> Self {
        match err {
            ChaserError::InvalidArgument(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            ChaserError::BrowserNotFound(msg) => {
                tonic::Status::not_found(msg)
            }
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}
```

### 日志规范

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn launch_browser(&self, opts: BrowserOptions) -> Result<BrowserId> {
    info!(options = ?opts, "Launching browser");

    match self.launch_impl(opts).await {
        Ok(id) => {
            info!(browser_id = %id, "Browser launched successfully");
            Ok(id)
        }
        Err(e) => {
            error!(error = %e, "Failed to launch browser");
            Err(e)
        }
    }
}
```

### 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launch_browser() {
        let service = BrowserService::new();
        let request = LaunchRequest {
            options: Some(BrowserOptions::default()),
        };

        let response = service.launch(request).await;

        assert!(response.browser_info.is_some());
    }

    #[tokio::test]
    async fn test_close_browser() {
        // 测试关闭浏览器
    }
}
```

---

## 🎯 验收标准

### 功能验收

- [ ] 所有 proto 定义的服务方法均已实现
- [ ] Python 客户端可以成功调用所有 API
- [ ] 隐身配置可以正确应用到页面
- [ ] 事件订阅可以正常工作
- [ ] 资源清理正确执行（无内存泄漏）

### 性能验收

- [ ] 单个浏览器启动时间 < 3 秒
- [ ] 页面导航响应时间 < 500ms
- [ ] 并发支持 ≥ 10 个浏览器实例
- [ ] 并发支持 ≥ 50 个页面实例
- [ ] 内存占用 < 500MB (空闲状态)

### 稳定性验收

- [ ] 连续运行 24 小时无崩溃
- [ ] 压力测试: 100 并发请求无错误
- [ ] 异常恢复: 浏览器崩溃后服务正常

### 安全验收

- [ ] 所有输入参数均已验证
- [ ] 资源限制正确实施
- [ ] 错误信息不泄露敏感数据

---

## 📦 交付清单

### 代码交付

- [ ] 完整的 Rust 源代码
- [ ] 单元测试（覆盖率 ≥ 80%）
- [ ] 集成测试
- [ ] 端到端测试
- [ ] API 文档注释

### 文档交付

- [ ] README.md (项目介绍和快速开始)
- [ ] API.md (API 使用文档)
- [ ] DEPLOYMENT.md (部署指南)
- [ ] DEVELOPMENT.md (开发指南)
- [ ] CHANGELOG.md (变更日志)

### 配置交付

- [ ] Cargo.toml
- [ ] config/default.toml
- [ ] Dockerfile
- [ ] docker-compose.yml
- [ .github/workflows/ci.yml

### 示例交付

- [ ] Python 客户端示例（已完成）
- [ ] Go 客户端示例（可选）

---

## 🚀 快速开始（完成后）

### 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/ccheshirecat/chaser-oxide.git
cd chaser-oxide
```

### 构建项目

```bash
# 生成 Proto 代码
./build/gen_proto.sh

# 构建 Release 版本
cargo build --release
```

### 启动服务

```bash
# 使用默认配置启动
./target/release/chaser-oxide-server

# 或使用自定义配置
./target/release/chaser-oxide-server --config config/custom.toml
```

### 运行客户端示例

```bash
cd docs/examples/python

# 安装依赖
pip install -r requirements.txt

# 运行基础示例
python basic_client.py

# 运行隐身功能示例
python stealth_client.py
```

---

## 📞 支持和维护

### 问题反馈

- GitHub Issues: https://github.com/ccheshirecat/chaser-oxide/issues
- 讨论: GitHub Discussions

### 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交 Pull Request
4. 等待代码审查

### 许可证

MIT License

---

## 📅 里程碑时间表

| 里程碑 | 目标日期 | 交付物 |
|--------|----------|--------|
| M1: 基础设施 | Week 2 | 项目框架、gRPC 服务器 |
| M2: CDP 核心 | Week 5 | CDP 连接、浏览器控制 |
| M3: 核心服务 | Week 10 | Browser、Page、Element 服务 |
| M4: 高级服务 | Week 14 | Profile、Event 服务 |
| M5: 隐身引擎 | Week 18 | 隐身功能、行为模拟 |
| M6: 测试验证 | Week 20 | 测试套件、质量保证 |
| M7: 部署发布 | Week 22 | 部署配置、文档 |

---

## 🎓 参考资源

### 技术文档

- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [tonic 文档](https://docs.rs/tonic/)
- [Tokio 文档](https://tokio.rs/)
- [Protocol Buffers](https://protobuf.dev/)

### 相关项目

- [Puppeteer](https://github.com/puppeteer/puppeteer) - Node.js 浏览器自动化
- [Playwright](https://github.com/microsoft/playwright) - 跨浏览器自动化
- [headless-chrome](https://github.com/alixaxel/chrome-remote-interface) - Rust Chrome 控制

### 反检测技术

- [botSight](https://github.com/fipis/botSight) - 浏览器指纹检测
- [CreepJS](https://abrahamjuliot.github.io/creepjs/) - 浏览器指纹测试
- [selenium-stealth](https://github.com/olkal/selenium-stealth) - Selenium 隐身

---

*文档版本: 1.0*
*创建日期: 2026-01-09*
*最后更新: 2026-01-09*
