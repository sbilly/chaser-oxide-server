# Chaser-Oxide gRPC 服务架构设计

## 系统概述

Chaser-Oxide gRPC 服务是一个基于 Rust 的浏览器自动化微服务，通过 gRPC 协议提供对 Chrome DevTools Protocol (CDP) 的抽象访问。服务设计强调高性能、类型安全和反检测能力。

## 架构层次

```
┌──────────────────────────────────────────────────────────────────┐
│                        Client Layer                             │
│              (Python, Go, Rust, etc.)                            │
└─────────────────────────────────┬────────────────────────────────┘
                                  │ gRPC/HTTP2
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                      API Gateway Layer                           │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              gRPC Server (tonic/Tower)                 │     │
│  │  - Request validation                                   │     │
│  │  - Authentication/Authorization                         │     │
│  │  - Rate limiting                                        │     │
│  │  - Metrics collection                                   │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                       Service Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │  Browser    │ │    Page     │ │  Element    │               │
│  │  Service    │ │   Service   │ │   Service   │               │
│  │             │ │             │ │             │               │
│  │ - Launch    │ │ - Navigate  │ │ - Find      │               │
│  │ - Close     │ │ - Evaluate  │ │ - Click     │               │
│  │ - Connect   │ │ - Screenshot│ │ - Type      │               │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘               │
│         │                │                │                      │
│  ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐               │
│  │  Profile    │ │    Event    │ │             │               │
│  │  Service    │ │   Service   │ │             │               │
│  │             │ │             │ │             │               │
│  │ - Create    │ │ - Subscribe │ │             │               │
│  │ - Apply     │ │ - Stream    │ │             │               │
│  │ - Randomize │ │ - Filter    │ │             │               │
│  └──────┬──────┘ └──────┬──────┘ └─────────────┘               │
└─────────┼─────────────────┼─────────────────┼──────────────────┘
          │                 │                 │
          └─────────────────┴─────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Business Logic Layer                         │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              Session Manager                            │     │
│  │  - Browser instance lifecycle                          │     │
│  │  - Page instance management                            │     │
│  │  - Element reference tracking                          │     │
│  │  - Resource cleanup                                    │     │
│  └────────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              Event Dispatcher                           │     │
│  │  - Event aggregation                                    │     │
│  │  - Subscription management                              │     │
│  │  - Event filtering                                      │     │
│  │  - Stream multiplexing                                  │     │
│  └────────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐     │
│  │              Stealth Engine                             │     │
│  │  - Profile injection                                    │     │
│  │  - Fingerprint spoofing                                 │     │
│  │  - Human behavior simulation                            │     │
│  │  - Timing randomization                                 │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                   Data Access Layer                              │
│  ┌────────────────────────────────────────────────────────┐     │
│  │         Chaser-Oxide Core Library                       │     │
│  │  - Browser wrapper                                      │     │
│  │  - Page management                                      │     │
│  │  - Element interaction                                  │     │
│  │  - CDP communication                                    │     │
│  └────────────────────────────────────────────────────────┘     │
└─────────────────────────────────┬────────────────────────────────┘
                                  │ WebSocket (CDP)
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Chrome/Chromium Browser                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   Main      │ │  Renderer   │ │   Network   │               │
│  │   Process   │ │   Process   │ │   Process   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└──────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. gRPC Server (tonic + Tower)

使用 `tonic` 框架构建 gRPC 服务器，配合 `Tower` 中间件栈提供：

- **请求验证** - Proto 定义自动实现类型验证
- **认证授权** - TLS + Token 认证
- **限流控制** - 基于令牌桶的速率限制
- **指标收集** - Prometheus metrics
- **日志追踪** - 结构化日志和分布式追踪

```rust
// 服务构建示例
let layer = ServiceBuilder::new()
    .layer(ValidateRequestLayer::new())
    .layer(AuthLayer::new())
    .layer(RateLimitLayer::new(100, Duration::from_secs(1)))
    .layer(MetricsLayer::new())
    .into_inner();

Server::builder()
    .layer(layer)
    .add_service(BrowserServiceServer::new(browser_service))
    .add_service(PageServiceServer::new(page_service))
    .serve(addr)
    .await?;
```

### 2. Session Manager

管理浏览器、页面和元素的生命周期：

```rust
pub struct SessionManager {
    browsers: Arc<RwLock<HashMap<BrowserId, BrowserContext>>>,
    pages: Arc<RwLock<HashMap<PageId, PageContext>>>,
    elements: Arc<RwLock<HashMap<ElementId, ElementRef>>>,
}

impl SessionManager {
    pub async fn create_browser(&self, opts: BrowserOptions) -> Result<BrowserId>;
    pub async fn create_page(&self, browser_id: &BrowserId) -> Result<PageId>;
    pub async fn get_page(&self, page_id: &PageId) -> Result<Arc<Page>>;
    pub async fn cleanup(&self, browser_id: &BrowserId);
}
```

**生命周期管理**：
- Browser 实例：独立进程，由 Session Manager 管理
- Page 实例：隶属于 Browser，共享浏览器进程
- Element 引用：弱引用，自动清理

### 3. Event Dispatcher

处理事件订阅和分发：

```rust
pub struct EventDispatcher {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Subscription>>>,
    broadcasters: Arc<RwLock<HashMap<TargetId, BroadcastChannel<Event>>>>,
}

impl EventDispatcher {
    pub async fn subscribe(&self, req: SubscribeRequest) -> Result<SubscriptionId>;
    pub async fn unsubscribe(&self, id: &SubscriptionId);
    pub async fn publish(&self, event: Event);

    // CDP 事件处理
    pub async fn handle_cdp_event(&self, cdp_event: CdpEvent) {
        let event = self.convert_to_grpc_event(cdp_event).await;
        self.publish(event).await;
    }
}
```

**事件流架构**：
- 使用 Tokio 的 `broadcast` 通道实现多路复用
- 支持基于浏览器、页面或事件类型的过滤
- 背压处理防止内存溢出

### 4. Stealth Engine

实现隐身和反检测功能：

```rust
pub struct StealthEngine {
    profiles: Arc<RwLock<HashMap<ProfileId, Profile>>>,
    injectors: HashMap<InjectionType, Box<dyn Injector>>,
}

impl StealthEngine {
    pub async fn apply_profile(&self, page: &Page, profile: &Profile) -> Result<()> {
        // 注入 Navigator 属性
        self.inject_navigator(page, &profile.fingerprint.navigator).await?;

        // 注入 WebGL 指纹
        self.inject_webgl(page, &profile.fingerprint.webgl).await?;

        // 注入 Canvas 保护
        self.inject_canvas_protection(page).await?;

        // 配置传输层隐身
        self.configure_transport_stealth(page).await?;

        Ok(())
    }

    async fn inject_navigator(&self, page: &Page, nav: &NavigatorInfo) -> Result<()> {
        let script = format!(
            r#"
            Object.defineProperty(navigator, 'platform', {{get: () => '{}'}});
            Object.defineProperty(navigator, 'hardwareConcurrency', {{get: () => {}}});
            Object.defineProperty(navigator, 'deviceMemory', {{get: () => {}}});
            Object.defineProperty(navigator, 'vendor', {{get: () => '{}'}});
            "#,
            nav.platform, nav.cpu_cores, nav.device_memory, nav.vendor
        );

        page.evaluate(script, false).await?;
        Ok(())
    }
}
```

**隐身技术**：
- **Navigator 属性覆盖** - 使用 Object.defineProperty 覆盖只读属性
- **WebGL 指纹欺骗** - 修改 vendor/renderer 字符串
- **Canvas 保护** - 添加噪声防止指纹识别
- **传输层隐身** - 使用 Page.createIsolatedWorld 替代 Runtime.enable
- **时序随机化** - 添加随机延迟模拟人类行为

### 5. Human Behavior Simulator

模拟人类交互模式：

```rust
pub struct HumanBehaviorSimulator {
    rng: StdRng,
    bezier_solver: BezierSolver,
}

impl HumanBehaviorSimulator {
    // Bezier 曲线鼠标移动
    pub async fn simulate_mouse_move(&self, page: &Page, start: Point, end: Point) -> Result<()> {
        let curve = self.bezier_solver.generate_random_curve(start, end);
        let duration = self.random_duration(100, 300); // 100-300ms

        for point in curve.points(Duration::from_millis(duration)) {
            page.dispatch_mouse_event("mousemove", point).await?;
            tokio::time::sleep(Duration::from_millis(16)).await; // 60fps
        }

        Ok(())
    }

    // 人类打字模式
    pub async fn simulate_typing(&self, page: &Page, element: &ElementRef, text: &str) -> Result<()> {
        for ch in text.chars() {
            // 随机打字速度 (50-150ms)
            let delay = self.rng.gen_range(50..150);

            // 模拟打字错误
            if self.rng.gen::<f64>() < 0.02 { // 2% 错误率
                page.type_char(element, random_char()).await?;
                tokio::time::sleep(Duration::from_millis(200)).await?;
                page.press_key("Backspace").await?;
            }

            page.type_char(element, ch).await?;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        Ok(())
    }
}
```

## 并发模型

### 异步运行时 (Tokio)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 配置 Tokio 运行时
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chaser-oxide")
        .enable_all()
        .build()?;

    runtime.block_on(async {
        start_server().await?;
        Ok(())
    })
}
```

### 任务调度

- **I/O 密集型** - 使用 `tokio::spawn` 处理网络 I/O
- **CPU 密集型** - 使用 `tokio::task::spawn_blocking` 处理阻塞操作
- **定时任务** - 使用 `tokio::time::interval` 实现心跳和清理

### 资源限制

```rust
pub struct ResourceLimiter {
    max_browsers: Semaphore,
    max_pages: Semaphore,
    max_memory: AtomicU64,
}

impl ResourceLimiter {
    pub async fn acquire_browser(&self) -> Result<Permit<'_>> {
        self.max_browsers.acquire().await.map_err(|_| {
            Error::ResourceExhausted("Max browser limit reached")
        })
    }
}
```

## 数据流

### 请求处理流程

```
Client Request
    │
    ▼
[tonic] gRPC Server
    │
    ├─► Request Validation
    ├─► Authentication
    ├─► Rate Limiting
    │
    ▼
Service Handler
    │
    ├─► Parse Request
    ├─► Session Lookup
    │
    ▼
Business Logic
    │
    ├─► Parameter Validation
    ├─► State Check
    │
    ▼
[Chaser-Oxide] Core Library
    │
    ├─► CDP Command
    │
    ▼
Chrome Browser
    │
    ▼
Response
    │
    ◄─── CDP Event (async)
    │
    ▼
[Chaser-Oxide] Core Library
    │
    ▼
Event Dispatcher
    │
    ├─► Filter
    ├─► Transform
    │
    ▼
Subscribers (gRPC Stream)
```

### 事件流处理

```
CDP Event
    │
    ▼
CDP Event Listener
    │
    ├─► Parse Event
    ├─► Convert to gRPC Event
    │
    ▼
Event Dispatcher
    │
    ├─► Match Subscriptions
    ├─► Apply Filters
    │
    ▼
Broadcast Channels
    │
    ├─► Browser Channel
    ├─► Page Channel
    ├─► Event Type Channel
    │
    ▼
gRPC Stream Writers
    │
    ▼
Client Receivers
```

## 错误处理

### 错误分类

```rust
pub enum ChaserError {
    // 客户端错误 (4xx)
    InvalidArgument(String),
    NotFound(String),
    AlreadyExists(String),
    PermissionDenied(String),
    ResourceExhausted(String),

    // 服务端错误 (5xx)
    Internal(String),
    Unavailable(String),
    DeadlineExceeded(String),

    // 特定错误
    BrowserClosed,
    PageClosed,
    ElementNotFound,
    NavigationFailed(String),
    EvaluationFailed(String),
    Timeout(Duration),
}

impl From<ChaserError> for Status {
    fn from(err: ChaserError) -> Status {
        match err {
            ChaserError::InvalidArgument(msg) => Status::invalid_argument(msg),
            ChaserError::NotFound(msg) => Status::not_found(msg),
            // ... 其他映射
        }
    }
}
```

### 错误恢复策略

- **重试** - 对于暂时性错误（网络超时）
- **降级** - 禁用非核心功能
- **快速失败** - 对于不可恢复的错误
- **断路器** - 防止级联故障

## 配置管理

### 配置结构

```rust
#[derive(Config, Deserialize)]
pub struct ServerConfig {
    pub server: ServerNetworkConfig,
    pub browser: BrowserConfig,
    pub limits: ResourceLimits,
    pub stealth: StealthConfig,
    pub logging: LoggingConfig,
}

#[derive(Config, Deserialize)]
pub struct ServerNetworkConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
    pub max_connections: usize,
}

#[derive(Config, Deserialize)]
pub struct BrowserConfig {
    pub executable_path: Option<String>,
    pub headless: bool,
    pub default_args: Vec<String>,
    pub proxy: Option<ProxyConfig>,
}
```

### 配置源优先级

1. 环境变量 (`CHASER_SERVER_PORT=50051`)
2. 配置文件 (`config.toml`)
3. 默认值

## 监控和可观测性

### Metrics (Prometheus)

```rust
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_browsers: Gauge,
    pub active_pages: Gauge,
}
```

### Logging (tracing)

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn launch_browser(&self, opts: BrowserOptions) -> Result<BrowserId> {
    info!(options = ?opts, "Launching browser");
    // ...
    Ok(id)
}
```

### Distributed Tracing (OpenTelemetry)

```rust
use opentelemetry::trace::TraceContextExt;

#[instrument]
pub async fn navigate_page(&self, page_id: &PageId, url: &str) -> Result<()> {
    let span = tracing::span!(Level::INFO, "navigate", page_id, url);
    let _enter = span.enter();

    // 操作...

    Ok(())
}
```

## 部署架构

### 单机部署

```
┌─────────────────────────────────────┐
│         chaser-oxide-server         │
│  ┌─────────────────────────────┐   │
│  │  gRPC Server (tokio)        │   │
│  │  - 4 workers                │   │
│  │  - 2GB RAM limit            │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Browser Pool               │   │
│  │  - Max 10 browsers          │   │
│  │  - Max 50 pages             │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

### 集群部署

```
                      Load Balancer
                            │
            ┌───────────────┼───────────────┐
            │               │               │
    ┌───────▼───────┐ ┌─────▼──────┐ ┌────▼─────┐
    │  Instance 1   │ │ Instance 2 │ │Instance 3 │
    │  gRPC Server  │ │ gRPC Server│ │gRPC Server│
    └───────┬───────┘ └─────┬──────┘ └────┬─────┘
            │               │               │
            └───────────────┼───────────────┘
                            │
                    ┌───────▼────────┐
                    │ Shared Storage │
                    │ - Redis (state)│
                    │ - PostgreSQL   │
                    └────────────────┘
```

## 性能优化

### 1. 连接复用

- HTTP/2 多路复用
- Keep-alive 连接
- 连接池管理

### 2. 批量操作

```rust
pub async fn batch_click(&self, requests: Vec<ClickRequest>) -> Vec<ClickResponse> {
    let futures: Vec<_> = requests.into_iter()
        .map(|req| self.click(req))
        .collect();

    try_join_all(futures).await.unwrap_or_default()
}
```

### 3. 内存优化

- 使用 `bytes::Bytes` 替代 `Vec<u8>` (零拷贝)
- 弱引用减少内存占用
- 及时清理空闲资源

### 4. 并发限制

```rust
let semaphore = Arc::new(Semaphore::new(100));

async fn process_request(&self) -> Result<()> {
    let _permit = semaphore.acquire().await?;
    // 处理请求
    Ok(())
}
```

## 安全考虑

### 1. 网络安全

- TLS 加密通信
- 证书验证
- IP 白名单

### 2. 认证授权

- JWT Token 认证
- RBAC 权限控制
- API Key 管理

### 3. 沙箱隔离

- Chrome 进程沙箱
- 文件系统隔离
- 网络隔离

### 4. 资源限制

- CPU 限制
- 内存限制
- 超时限制

## 测试策略

### 单元测试

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
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_full_workflow() {
    // 1. 启动浏览器
    let browser_id = launch_browser().await?;

    // 2. 创建页面
    let page_id = create_page(browser_id).await?;

    // 3. 导航
    navigate(page_id, "https://example.com").await?;

    // 4. 查找元素
    let element = find_element(page_id, "#button").await?;

    // 5. 点击
    click(element).await?;

    // 清理
    close_browser(browser_id).await?;
}
```

### 压力测试

使用 `grpc-rs` 客户端模拟并发请求：

```bash
# 100 并发连接，1000 请求
ghz --insecure \
    --call chaser.oxide.v1.BrowserService.Launch \
    -- concurrency 100 \
    -- n 1000 \
    -d testdata/launch.json \
    localhost:50051
```

## 未来扩展

1. **插件系统** - 支持自定义插件扩展功能
2. **分布式调度** - 跨服务器任务调度
3. **智能路由** - 基于负载的请求路由
4. **缓存层** - 减少重复操作
5. **GraphQL 支持** - 提供 GraphQL API
