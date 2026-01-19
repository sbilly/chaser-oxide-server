//! Chaser-Oxide gRPC server entry point

// Core imports
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

// Chaser-Oxide imports
use chaser_oxide::{
    config::Config,
    cdp::browser::CdpBrowserImpl,
    cdp::mock::MockCdpClient,
    cdp::traits::CdpBrowser,
    process::{ProcessManager, ProcessManagerConfig, run_health_check},
    session::{SessionManager, SessionManagerImpl},
    services::{
        BrowserServiceGrpc, ElementGrpcService, EventDispatcher, EventGrpcService,
        PageServiceGrpc, ProfileServiceImpl,
        profile::{ProfileManagerImpl, ProfileServiceGrpc},
    },
    stealth::{
        BehaviorSimulatorImpl, FingerprintGeneratorImpl, ScriptInjectorImpl,
        StealthEngineImpl,
    },
    stealth::traits::{BehaviorSimulator, FingerprintGenerator, ProfileManager, ScriptInjector, StealthEngine},
};

// gRPC generated types
use chaser_oxide::chaser_oxide::v1::{
    browser_service_server::BrowserServiceServer as BrowserServer,
    element_service_server::ElementServiceServer as ElementServer,
    event_service_server::EventServiceServer,
    page_service_server::PageServiceServer as PageServer,
    profile_service_server::ProfileServiceServer as ProfileServer,
};

#[cfg(unix)]
use nix::sys::wait::{waitpid, WaitPidFlag};
#[cfg(unix)]
use nix::unistd::Pid;

/// Container for all service dependencies
struct ServiceDependencies {
    session_manager_impl: Arc<SessionManagerImpl>,
    session_manager: Arc<dyn SessionManager>,
    event_dispatcher: Arc<EventDispatcher>,
    profile_manager: Arc<dyn ProfileManager>,
    stealth_engine: Arc<dyn StealthEngine>,
    process_manager: Arc<ProcessManager>,
}

/// Initialize tracing subscriber with configurable log level
fn init_tracing() {
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|v| v.parse::<Level>().ok())
        .unwrap_or(Level::INFO);

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

/// Build ProcessManagerConfig from environment variables
fn build_process_manager_config() -> ProcessManagerConfig {
    ProcessManagerConfig {
        chrome_path: std::env::var("CHASER_BROWSER_PATH")
            .unwrap_or_else(|_| "chromium".to_string()),
        data_dir: std::env::var("CHASER_DATA_DIR")
            .unwrap_or_else(|_| "/app/data".to_string()),
        port_range: (
            std::env::var("CHASER_CDP_PORT_START")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9000),
            std::env::var("CHASER_CDP_PORT_END")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9900),
        ),
        health_check_interval: std::time::Duration::from_secs(
            std::env::var("CHASER_HEALTH_CHECK_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        ),
    }
}

/// Initialize all service dependencies
fn init_services(_config: &Config) -> ServiceDependencies {
    // Initialize process manager
    let pm_config = build_process_manager_config();
    let process_manager = Arc::new(ProcessManager::with_config(pm_config.clone()));
    info!("Process manager initialized");

    // Determine CDP mode
    let cdp_endpoint = std::env::var("CHASER_CDP_ENDPOINT").ok();
    let use_external_cdp = cdp_endpoint.is_some();

    if use_external_cdp {
        info!("Using external CDP endpoint: {}", cdp_endpoint.as_ref().unwrap());
    } else {
        info!("Using self-managed browser mode");
    }

    // Create session manager
    let session_manager_impl: Arc<SessionManagerImpl> = if let Some(endpoint) = cdp_endpoint {
        let factory = move || {
            Ok(Arc::new(CdpBrowserImpl::new(endpoint.clone())) as Arc<dyn CdpBrowser>)
        };
        Arc::new(SessionManagerImpl::new(factory))
    } else {
        let factory = || {
            Ok(Arc::new(CdpBrowserImpl::new("ws://localhost:9222".to_string())) as Arc<dyn CdpBrowser>)
        };
        Arc::new(SessionManagerImpl::with_process_manager(factory, process_manager.clone()))
    };
    let session_manager: Arc<dyn SessionManager> = session_manager_impl.clone();
    info!("Session manager initialized");

    // Create event dispatcher
    let event_dispatcher = Arc::new(EventDispatcher::new(1000));

    // Create stealth components
    let script_injector: Arc<dyn ScriptInjector> = Arc::new(ScriptInjectorImpl::new(session_manager.clone()));
    let fingerprint_generator: Arc<dyn FingerprintGenerator> = Arc::new(FingerprintGeneratorImpl::new());
    let profile_manager: Arc<dyn ProfileManager> = Arc::new(ProfileManagerImpl::new(fingerprint_generator));
    let behavior_simulator: Arc<dyn BehaviorSimulator> = Arc::new(BehaviorSimulatorImpl::new(Arc::new(MockCdpClient::new())));
    let stealth_engine: Arc<dyn StealthEngine> = Arc::new(StealthEngineImpl::new(script_injector, behavior_simulator));

    ServiceDependencies {
        session_manager_impl,
        session_manager,
        event_dispatcher,
        profile_manager,
        stealth_engine,
        process_manager,
    }
}

/// Type alias for the complete set of gRPC services
type GrpcServices = (
    BrowserServer<BrowserServiceGrpc<SessionManagerImpl>>,
    PageServer<PageServiceGrpc<SessionManagerImpl>>,
    ElementServer<ElementGrpcService>,
    EventServiceServer<EventGrpcService>,
    ProfileServer<ProfileServiceGrpc>,
);

/// Create all gRPC service instances
fn create_grpc_services(deps: &ServiceDependencies) -> GrpcServices {
    let browser_service = BrowserServiceGrpc::new(deps.session_manager_impl.clone());
    let page_service = PageServiceGrpc::new(deps.session_manager_impl.clone());
    let element_service = ElementGrpcService::new(deps.session_manager.clone());
    let event_service = EventGrpcService::new(deps.event_dispatcher.clone());

    let profile_service = ProfileServiceGrpc::new(Arc::new(ProfileServiceImpl::new(
        deps.profile_manager.clone(),
        deps.stealth_engine.clone(),
        deps.session_manager.clone(),
    )));

    // Wrap services in generated Server types
    let browser_service = BrowserServer::new(browser_service);
    let page_service = PageServer::new(page_service);
    let element_service = element_service.into_server();
    let event_service = event_service.into_server();
    let profile_service = ProfileServer::new(profile_service);

    (browser_service, page_service, element_service, event_service, profile_service)
}

/// Spawn periodic session cleanup task
fn spawn_cleanup_task(session_manager: Arc<SessionManagerImpl>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Err(e) = session_manager.cleanup().await {
                warn!("Session cleanup failed: {}", e);
            } else {
                info!("Session cleanup completed. Active sessions: {}",
                    session_manager.session_count());
            }
        }
    });
}

/// Spawn zombie reaper task to reap orphaned child processes
/// This is necessary when running as PID 1 in a container
#[cfg(unix)]
fn spawn_zombie_reaper_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        let mut total_reaped = 0u64;

        loop {
            interval.tick().await;

            // Reap all available zombies using WNOHANG (non-blocking)
            let mut reaped_this_round = 0u32;
            loop {
                match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                    Ok(nix::sys::wait::WaitStatus::Exited(pid, exit_code)) => {
                        debug!("Reaped zombie: PID {} exited with code {}", pid, exit_code);
                        reaped_this_round += 1;
                    }
                    Ok(nix::sys::wait::WaitStatus::Signaled(pid, signal, ..)) => {
                        debug!("Reaped zombie: PID {} killed by signal {:?}", pid, signal);
                        reaped_this_round += 1;
                    }
                    Ok(nix::sys::wait::WaitStatus::StillAlive) => {
                        // No more zombies to reap
                        break;
                    }
                    Ok(_) => {
                        // Other states don't need reaping
                        break;
                    }
                    Err(nix::errno::Errno::ECHILD) => {
                        // No child processes
                        break;
                    }
                    Err(e) => {
                        warn!("Error waiting for child processes: {}", e);
                        break;
                    }
                }
            }

            if reaped_this_round > 0 {
                total_reaped += reaped_this_round as u64;
                info!("Reaped {} zombie process(es) (total: {})", reaped_this_round, total_reaped);
            }
        }
    });
}

#[cfg(not(unix))]
fn spawn_zombie_reaper_task() {
    // No-op on non-Unix systems
}

/// Setup graceful shutdown signal handler
async fn shutdown_signal() {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing();
    info!("Chaser-Oxide Server v{}", chaser_oxide::VERSION);

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded: host={}, port={}", config.host, config.port);

    // Initialize all service dependencies
    let deps = init_services(&config);

    // Create gRPC services
    let (browser_service, page_service, element_service, event_service, profile_service) =
        create_grpc_services(&deps);

    info!("gRPC services initialized");

    // Create gRPC server address
    let addr = format!("{}:{}", config.host, config.port);
    let addr = addr.parse::<std::net::SocketAddr>()?;

    info!("Starting gRPC server on {}", addr);

    // Start cleanup task
    spawn_cleanup_task(deps.session_manager_impl.clone());

    // Start health check task for browser processes
    tokio::spawn(run_health_check(deps.process_manager.clone()));
    info!("Health check task started");

    // Start zombie reaper task to reap orphaned child processes
    spawn_zombie_reaper_task();
    info!("Zombie reaper task started");

    // Setup graceful shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        shutdown_signal().await;
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
    if let Err(e) = deps.session_manager.cleanup().await {
        error!("Failed to cleanup sessions: {}", e);
    }

    // Cleanup all browser processes
    info!("Cleaning up all browser processes...");
    if let Err(e) = deps.process_manager.cleanup_all().await {
        error!("Failed to cleanup browser processes: {}", e);
    }

    info!("Server shutdown complete");
    Ok(())
}
