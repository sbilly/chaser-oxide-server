//! # Chaser-Oxide 服务入口
//!
//! Chaser-Oxide gRPC 服务器的入口点，提供基于 Chrome DevTools Protocol 的浏览器自动化服务。
//!
//! ## 主要功能
//! - 初始化并配置 gRPC 服务器
//! - 管理 CDP（Chrome DevTools Protocol）连接
//! - 提供浏览器、页面、元素、事件和配置文件的 gRPC 服务
//! - 实现优雅关闭和会话清理
//!
//! ## 架构
//! 服务由以下核心组件构成：
//! - **CDP 层**: 与 Chrome/Chromium 浏览器的 WebSocket 通信
//! - **会话管理**: 管理浏览器、页面和元素的生命周期
//! - **隐身引擎**: 提供浏览器指纹规避和人类行为模拟
//! - **服务层**: 实现 gRPC 服务接口
//!
//! ## 环境变量
//! - `CHASER_HOST`: 服务器监听地址（默认: 0.0.0.0）
//! - `CHASER_PORT`: 服务器监听端口（默认: 50051）
//! - `CHASER_CDP_ENDPOINT`: CDP WebSocket 端点（默认: ws://localhost:9222）

use chaser_oxide::{
    config::Config,
    cdp::browser::CdpBrowserImpl,
    cdp::mock::MockCdpClient,
    session::{SessionManagerImpl, SessionManager},
    services::{
        BrowserServiceGrpc, PageServiceGrpc, ElementGrpcService,
        EventGrpcService, EventDispatcher, ProfileServiceImpl,
        profile::{ProfileManagerImpl, ProfileServiceGrpc},
    },
    stealth::{
        StealthEngineImpl, ScriptInjectorImpl, BehaviorSimulatorImpl,
        FingerprintGeneratorImpl,
    },
};

// Import generated Server types for wrapping services
use chaser_oxide::chaser_oxide::v1::{
    browser_service_server::BrowserServiceServer as BrowserServer,
    page_service_server::PageServiceServer as PageServer,
    profile_service_server::ProfileServiceServer as ProfileServer,
};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing - respect RUST_LOG environment variable
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|v| v.parse::<Level>().ok())
        .unwrap_or(Level::INFO);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Chaser-Oxide Server v{}", chaser_oxide::VERSION);

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded: host={}, port={}", config.host, config.port);

    // Create CDP browser factory
    let cdp_endpoint = std::env::var("CHASER_CDP_ENDPOINT")
        .unwrap_or_else(|_| "ws://localhost:9222".to_string());

    let cdp_factory = move || {
        let endpoint = cdp_endpoint.clone();
        Ok(Arc::new(CdpBrowserImpl::new(endpoint)) as Arc<dyn chaser_oxide::cdp::traits::CdpBrowser>)
    };

    // Create session manager
    let session_manager_impl = Arc::new(SessionManagerImpl::new(cdp_factory));
    let session_manager: Arc<dyn SessionManager> = session_manager_impl.clone();
    info!("Session manager initialized");

    // Create event dispatcher
    let event_dispatcher = Arc::new(EventDispatcher::new(1000));

    // Create gRPC services - use concrete type for generic services, trait object for others
    let browser_service = BrowserServiceGrpc::new(session_manager_impl.clone());
    let page_service = PageServiceGrpc::new(session_manager_impl.clone());
    let element_service = ElementGrpcService::new(session_manager.clone());
    let event_service = EventGrpcService::new(event_dispatcher);

    // Create ProfileService dependencies
    // Use session_manager for ScriptInjector to get per-page CDP clients
    let script_injector = Arc::new(ScriptInjectorImpl::new(session_manager.clone()))
        as Arc<dyn chaser_oxide::stealth::traits::ScriptInjector>;

    // Create fingerprint generator
    let fingerprint_generator = Arc::new(FingerprintGeneratorImpl::new())
        as Arc<dyn chaser_oxide::stealth::traits::FingerprintGenerator>;

    // Create profile manager
    let profile_manager = Arc::new(ProfileManagerImpl::new(fingerprint_generator))
        as Arc<dyn chaser_oxide::stealth::traits::ProfileManager>;

    // Create behavior simulator with a mock client for now
    // (BehaviorSimulator is not actively used for script injection)
    let behavior_simulator = Arc::new(BehaviorSimulatorImpl::new(Arc::new(MockCdpClient::new())))
        as Arc<dyn chaser_oxide::stealth::traits::BehaviorSimulator>;

    // Create stealth engine
    let stealth_engine = Arc::new(StealthEngineImpl::new(script_injector, behavior_simulator))
        as Arc<dyn chaser_oxide::stealth::traits::StealthEngine>;

    // Create profile service
    let profile_service = ProfileServiceGrpc::new(Arc::new(ProfileServiceImpl::new(profile_manager, stealth_engine, session_manager.clone())));

    // Wrap services in generated Server types to implement NamedService
    let browser_service = BrowserServer::new(browser_service);
    let page_service = PageServer::new(page_service);
    let element_service = element_service.into_server();
    let event_service = event_service.into_server();
    let profile_service = ProfileServer::new(profile_service);

    info!("gRPC services initialized");

    // Create gRPC server address
    let addr = format!("{}:{}", config.host, config.port);
    let addr = addr.parse::<std::net::SocketAddr>()?;

    info!("Starting gRPC server on {}", addr);

    // Start cleanup task
    let session_manager_cleanup = session_manager_impl.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Err(e) = session_manager_cleanup.cleanup().await {
                warn!("Session cleanup failed: {}", e);
            } else {
                info!("Session cleanup completed. Active sessions: {}",
                    session_manager_cleanup.session_count());
            }
        }
    });

    // Setup graceful shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn signal handler
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM signal");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT signal");
                }
            }
        }

        #[cfg(windows)]
        {
            let _ = tokio::signal::ctrl_c().await;
            info!("Received Ctrl+C signal");
        }

        let _ = shutdown_tx.send(());
    });

    // Start gRPC server
    let server = Server::builder()
        .add_service(browser_service)
        .add_service(page_service)
        .add_service(element_service)
        .add_service(event_service)
        .add_service(profile_service)
        .serve_with_shutdown(addr, async {
            shutdown_rx.await.ok();
            info!("Shutdown signal received, stopping server...");
        });

    // Wait for server to complete
    server.await?;

    // Cleanup all sessions
    info!("Cleaning up all sessions...");
    if let Err(e) = session_manager.cleanup().await {
        error!("Failed to cleanup sessions: {}", e);
    }

    info!("Server shutdown complete");
    Ok(())
}
