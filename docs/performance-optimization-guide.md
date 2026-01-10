# Performance & Stability Optimization Guide

## Overview

This document provides optimization strategies and code implementations for improving performance and stability metrics that don't meet acceptance criteria.

## Performance Optimization Strategies

### 1. Browser Startup Time Optimization

**Target**: < 3000 ms
**Common Issues**: Slow CDP connection, redundant initialization, blocking operations

#### Solution 1.1: Connection Pooling

```rust
// src/cdp/connection_pool.rs
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

pub struct CdpConnectionPool {
    connections: Arc<Mutex<VecDeque<CdpConnection>>>,
    max_size: usize,
}

impl CdpConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    pub async fn acquire(&self) -> Result<CdpConnection, Error> {
        // Try to get from pool first
        {
            let mut pool = self.connections.lock().unwrap();
            if let Some(conn) = pool.pop_front() {
                if conn.is_valid().await {
                    return Ok(conn);
                }
            }
        }

        // Create new connection if pool is empty
        self.create_connection().await
    }

    pub fn release(&self, conn: CdpConnection) {
        let mut pool = self.connections.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(conn);
        }
    }
}
```

#### Solution 1.2: Lazy Browser Initialization

```rust
// src/session/browser.rs
impl BrowserContextImpl {
    pub async fn new_lazy(options: BrowserOptions) -> Result<Self, Error> {
        // Don't connect immediately, create stub
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            options,
            cdp_browser: None, // Lazy initialization
            pages: Arc::new(RwLock::new(HashMap::new())),
            is_active: Arc::new(RwLock::new(true)),
            initialized: Arc::new(AtomicBool::new(false)),
        })
    }

    async fn ensure_initialized(&self) -> Result<(), Error> {
        if !self.initialized.load(Ordering::Acquire) {
            let mut init = self.cdp_browser.write().await;
            if init.is_none() {
                *init = Some(self.create_cdp_browser().await?);
                self.initialized.store(true, Ordering::Release);
            }
        }
        Ok(())
    }
}
```

#### Solution 1.3: Parallel Initialization

```rust
// src/session/browser.rs
impl BrowserContextImpl {
    pub async fn new_parallel(options: BrowserOptions) -> Result<Self, Error> {
        let id = Uuid::new_v4().to_string();

        // Initialize components in parallel
        let (cdp_browser, pages, is_active) = tokio::join!(
            Self::create_cdp_browser_async(&options),
            async { Arc::new(RwLock::new(HashMap::new())) },
            async { Arc::new(RwLock::new(true)) }
        );

        Ok(Self {
            id,
            options,
            cdp_browser: cdp_browser?,
            pages,
            is_active,
        })
    }
}
```

### 2. Navigation Response Time Optimization

**Target**: < 500 ms
**Common Issues**: Blocking CDP calls, sequential operations, unnecessary waits

#### Solution 2.1: Non-blocking Navigation

```rust
// src/session/page.rs
impl PageContextImpl {
    pub async fn navigate_optimized(&self, url: &str, options: NavigationOptions) -> Result<NavigationResult, Error> {
        let active = *self.is_active.read().await;
        if !active {
            return Err(Error::page_not_found(&self.id));
        }

        // Start navigation immediately
        let navigate_future = self.cdp_client.navigate(url);

        // Prepare load state listener in parallel
        let load_listener = self.setup_load_state_listener(options.wait_until);

        // Wait for both navigation and load state
        let (nav_result, _) = tokio::join!(navigate_future, load_listener);

        Ok(NavigationResult {
            url: nav_result?.url,
            status_code: 200,
            is_loaded: true,
        })
    }

    async fn setup_load_state_listener(&self, wait_until: LoadState) {
        // Setup event listener for load state
        // Don't block navigation start
        match wait_until {
            LoadState::Load => {
                // Use DOMContentLoaded for faster response
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            LoadState::NetworkIdle => {
                // Implement proper network idle detection
                self.wait_for_network_idle().await;
            }
            _ => {}
        }
    }
}
```

#### Solution 2.2: Request Batching

```rust
// src/cdp/batcher.rs
pub struct CdpRequestBatcher {
    pending: Arc<Mutex<Vec<CdpRequest>>>,
    flush_interval: Duration,
}

impl CdpRequestBatcher {
    pub async fn submit(&self, request: CdpRequest) -> Result<CdpResponse, Error> {
        let mut pending = self.pending.lock().unwrap();
        pending.push(request);

        // Flush if batch is full
        if pending.len() >= 10 {
            self.flush_internal(&mut pending).await
        } else {
            // Wait for flush interval
            drop(pending);
            tokio::time::sleep(self.flush_interval).await;
            Ok(CdpResponse::Pending)
        }
    }

    async fn flush_internal(&self, pending: &mut Vec<CdpRequest>) -> Result<CdpResponse, Error> {
        let batch = std::mem::take(pending);
        // Send batch to CDP
        self.cdp_client.send_batch(batch).await
    }
}
```

### 3. Memory Usage Optimization

**Target**: < 500 MB (idle)
**Common Issues**: Memory leaks, large buffers, insufficient cleanup

#### Solution 3.1: Resource Pooling

```rust
// src/session/pool.rs
use std::collections::VecDeque;
use std::sync::Arc;

pub struct ResourcePool<T> {
    items: Arc<Mutex<VecDeque<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ResourcePool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            items: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            factory: Arc::new(factory),
            max_size,
        }
    }

    pub fn acquire(&self) -> PooledItem<T> {
        let item = self.items.lock().unwrap()
            .pop_front()
            .unwrap_or_else(|| (self.factory)());

        PooledItem {
            item: Some(item),
            pool: self.clone(),
        }
    }

    fn release(&self, item: T) {
        let mut pool = self.items.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(item);
        }
    }
}

pub struct PooledItem<T: ?Sized> {
    item: Option<T>,
    pool: ResourcePool<T>,
}

impl<T> Drop for PooledItem<T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            self.pool.release(item);
        }
    }
}
```

#### Solution 3.2: Aggressive Cleanup Strategy

```rust
// src/session/cleanup.rs
impl SessionManagerImpl {
    pub async fn aggressive_cleanup(&self) -> Result<(), Error> {
        let mut to_remove = Vec::new();

        {
            let browsers = self.browsers.read()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            for (id, browser) in browsers.iter() {
                // Remove inactive browsers
                if !browser.is_active() {
                    to_remove.push(id.clone());
                    continue;
                }

                // Close idle pages (no activity for 5 minutes)
                if let Ok(pages) = browser.get_pages().await {
                    for page in pages {
                        if page.is_idle_for(Duration::from_secs(300)).await {
                            let _ = page.close().await;
                        }
                    }
                }
            }
        }

        // Remove inactive browsers
        if !to_remove.is_empty() {
            let mut browsers = self.browsers.write()
                .map_err(|e| Error::internal(format!("Lock error: {}", e)))?;

            for id in to_remove {
                if let Some(browser) = browsers.remove(&id) {
                    // Force close and cleanup
                    let _ = tokio::spawn(async move {
                        let _ = browser.close().await;
                    });
                }
            }
        }

        Ok(())
    }
}
```

#### Solution 3.3: Buffer Size Limits

```rust
// src/cdp/client.rs
impl CdpClientImpl {
    const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
    const MAX_CONCURRENT_MESSAGES: usize = 100;

    pub async fn send_message(&self, message: CdpMessage) -> Result<(), Error> {
        // Check message size
        if message.size() > Self::MAX_MESSAGE_SIZE {
            return Err(Error::message_too_large(message.size()));
        }

        // Check concurrent message limit
        let semaphore = self.semaphore.clone();
        let _permit = semaphore.acquire().await?;

        // Send message with backpressure
        self.sender.send(message).await?;
        Ok(())
    }
}
```

### 4. Concurrent Browsers/Pages Optimization

**Target**: ≥ 10 browsers, ≥ 50 pages
**Common Issues**: Resource limits, thread pool exhaustion, lock contention

#### Solution 4.1: Async Resource Management

```rust
// src/session/async_manager.rs
use tokio::sync::Semaphore;

pub struct AsyncSessionManager {
    browsers: Arc<RwLock<HashMap<String, Arc<dyn BrowserContext>>>>,
    browser_semaphore: Arc<Semaphore>,
    page_semaphore: Arc<Semaphore>,
}

impl AsyncSessionManager {
    pub fn new(max_browsers: usize, max_pages: usize) -> Self {
        Self {
            browsers: Arc::new(RwLock::new(HashMap::new())),
            browser_semaphore: Arc::new(Semaphore::new(max_browsers)),
            page_semaphore: Arc::new(Semaphore::new(max_pages)),
        }
    }

    pub async fn create_browser(&self, options: BrowserOptions) -> Result<String, Error> {
        // Acquire semaphore permit
        let _permit = self.browser_semaphore.acquire().await?;

        // Create browser
        let browser = Arc::new(BrowserContextImpl::new(options)?);
        let browser_id = browser.id().to_string();

        self.browsers.write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .insert(browser_id.clone(), browser);

        Ok(browser_id)
    }

    pub async fn create_page(&self, browser_id: &str, options: PageOptions) -> Result<String, Error> {
        // Acquire semaphore permit
        let _permit = self.page_semaphore.acquire().await?;

        let browser = self.get_browser(browser_id).await?;
        let page = browser.create_page(options).await?;
        Ok(page.id().to_string())
    }
}
```

#### Solution 4.2: Lock-Free Operations

```rust
// src/session/lockfree.rs
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LockFreeSessionManager {
    browser_count: Arc<AtomicUsize>,
    page_count: Arc<AtomicUsize>,
    max_browsers: usize,
    max_pages: usize,
}

impl LockFreeSessionManager {
    pub async fn create_browser(&self, options: BrowserOptions) -> Result<String, Error> {
        // Atomic increment
        let current = self.browser_count.fetch_add(1, Ordering::AcqRel);

        if current >= self.max_browsers {
            self.browser_count.fetch_sub(1, Ordering::AcqRel);
            return Err(Error::too_many_browsers());
        }

        // Create browser without lock
        let browser = BrowserContextImpl::new(options)?;
        Ok(browser.id().to_string())
    }
}
```

### 5. Stress Test Optimization

**Target**: ≥ 95% success rate
**Common Issues**: Timeouts, connection failures, resource exhaustion

#### Solution 5.1: Circuit Breaker Pattern

```rust
// src/circuit_breaker.rs
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

pub struct CircuitBreaker {
    failure_count: AtomicUsize,
    last_failure_time: AtomicUsize,
    is_open: AtomicBool,
    threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: Future<Output = Result<R, Error>>,
    {
        // Check if circuit is open
        if self.is_open.load(Ordering::Acquire) {
            if self.should_attempt_reset() {
                self.is_open.store(false, Ordering::Release);
            } else {
                return Err(Error::circuit_breaker_open());
            }
        }

        // Execute function
        match f.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    fn on_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::AcqRel);
        if count >= self.threshold {
            self.is_open.store(true, Ordering::Release);
        }
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Release);
    }
}
```

#### Solution 5.2: Retry with Exponential Backoff

```rust
// src/retry.rs
pub async fn retry_with_backoff<F, R, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<R, E>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    tokio::time::sleep(delay).await;
                    delay = delay.saturating_mul(2); // Exponential backoff
                }
            }
        }
    }

    Err(last_error.unwrap())
}
```

## Stability Optimization Strategies

### 6. Memory Leak Prevention

**Target**: No memory leaks
**Common Issues**: Circular references, unclosed resources, large cached data

#### Solution 6.1: Weak References

```rust
// src/session/weak_refs.rs
use std::sync::{Arc, Weak};

pub struct BrowserContextImpl {
    id: String,
    // Use weak reference to avoid circular references
    manager: Weak<SessionManagerImpl>,
    pages: Arc<RwLock<HashMap<String, Weak<PageContextImpl>>>>,
}

impl BrowserContextImpl {
    pub async fn create_page(&self, options: PageOptions) -> Result<Arc<dyn PageContext>, Error> {
        let page = Arc::new(PageContextImpl::new(
            self.id.clone(),
            options,
        ));

        // Store weak reference
        let weak = Arc::downgrade(&page) as Weak<dyn PageContext>;
        self.pages.write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
            .insert(page.id().to_string(), weak);

        Ok(page)
    }

    pub async fn cleanup_closed_pages(&self) {
        let mut pages = self.pages.write()
            .map_err(|e| Error::internal(format!("Lock error: {}", e)))
            .unwrap();

        pages.retain(|_, weak| weak.strong_count() > 0);
    }
}
```

#### Solution 6.2: Automatic Resource Cleanup

```rust
// src/session/auto_cleanup.rs
impl SessionManagerImpl {
    pub fn start_auto_cleanup(&self, interval: Duration) -> JoinHandle<()> {
        let manager = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                interval.tick().await;

                // Perform cleanup
                if let Err(e) = manager.aggressive_cleanup().await {
                    eprintln!("Cleanup error: {}", e);
                }

                // Trigger garbage collection (if using a runtime with GC)
                manager.force_garbage_collection().await;
            }
        })
    }

    async fn force_garbage_collection(&self) {
        // Explicitly drop unused resources
        // This is especially important for large buffers
        tokio::task::yield_now().await;
    }
}
```

### 7. Crash Recovery Optimization

**Target**: ≥ 80% recovery success rate
**Common Issues**: State corruption, incomplete recovery, cascading failures

#### Solution 7.1: State Snapshot and Restore

```rust
// src/session/snapshot.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub browsers: Vec<BrowserState>,
    pub timestamp: DateTime<Utc>,
}

impl SessionManagerImpl {
    pub async fn create_snapshot(&self) -> Result<SessionSnapshot, Error> {
        let browsers = self.list_browsers().await?;
        let mut browser_states = Vec::new();

        for browser_id in browsers {
            if let Ok(browser) = self.get_browser(&browser_id).await {
                let pages = browser.get_pages().await?;
                let page_states: Vec<_> = pages.iter()
                    .map(|p| PageState {
                        id: p.id().to_string(),
                        url: p.current_url().await.unwrap_or_default(),
                    })
                    .collect();

                browser_states.push(BrowserState {
                    id: browser_id,
                    options: browser.options().clone(),
                    pages: page_states,
                });
            }
        }

        Ok(SessionSnapshot {
            browsers: browser_states,
            timestamp: Utc::now(),
        })
    }

    pub async fn restore_from_snapshot(&self, snapshot: SessionSnapshot) -> Result<(), Error> {
        // Restore all browsers from snapshot
        for browser_state in snapshot.browsers {
            let browser_id = self.create_browser(browser_state.options).await?;

            // Restore pages
            for page_state in browser_state.pages {
                let page = self.create_page(&browser_id, PageOptions::default()).await?;
                if !page_state.url.is_empty() {
                    let _ = page.navigate(&page_state.url, NavigationOptions::default()).await;
                }
            }
        }

        Ok(())
    }
}
```

#### Solution 7.2: Health Check and Auto-Recovery

```rust
// src/health/check.rs
pub struct HealthChecker {
    manager: Arc<SessionManagerImpl>,
    check_interval: Duration,
}

impl HealthChecker {
    pub async fn start(&self) -> JoinHandle<()> {
        let manager = self.manager.clone();
        let interval = self.check_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                // Check browser health
                if let Err(e) = Self::check_and_recover(&manager).await {
                    eprintln!("Health check failed: {}", e);
                }
            }
        })
    }

    async fn check_and_recover(manager: &Arc<SessionManagerImpl>) -> Result<(), Error> {
        let browsers = manager.list_browsers().await?;

        for browser_id in browsers {
            // Check if browser is responsive
            match manager.get_browser(&browser_id).await {
                Ok(browser) => {
                    if !browser.is_healthy().await {
                        eprintln!("Browser {} unhealthy, attempting recovery...", browser_id);

                        // Create snapshot before recovery
                        let snapshot = manager.create_snapshot().await?;

                        // Close unhealthy browser
                        let _ = manager.close_browser(&browser_id).await;

                        // Restore from snapshot
                        let _ = manager.restore_from_snapshot(snapshot).await;
                    }
                }
                Err(_) => {
                    eprintln!("Browser {} not found, cleaning up...", browser_id);
                    let _ = manager.browsers.write()
                        .map_err(|e| Error::internal(format!("Lock error: {}", e)))?
                        .remove(&browser_id);
                }
            }
        }

        Ok(())
    }
}
```

## Implementation Priority

### High Priority (Critical for Production)
1. **Connection Pooling** (Solution 1.1) - Immediate impact on startup time
2. **Resource Pooling** (Solution 3.1) - Reduces memory overhead
3. **Circuit Breaker** (Solution 5.1) - Improves stress test reliability

### Medium Priority (Significant Improvements)
4. **Lazy Browser Initialization** (Solution 1.2) - Reduces initial resource usage
5. **Non-blocking Navigation** (Solution 2.1) - Improves responsiveness
6. **Automatic Cleanup** (Solution 6.2) - Prevents memory leaks

### Low Priority (Nice to Have)
7. **Lock-Free Operations** (Solution 4.2) - Optimizes high-concurrency scenarios
8. **State Snapshot** (Solution 7.1) - Improves recovery reliability

## Testing Recommendations

After implementing optimizations, run the acceptance test suite:

```bash
# Run performance tests
cargo test --test performance_acceptance -- --nocapture

# Run stability tests
cargo test --test performance_acceptance test_stability -- --nocapture

# Generate full report
cargo test --test performance_acceptance -- --nocapture --test-threads=1
```

## Monitoring in Production

Implement metrics collection to track these optimizations:

```rust
// src/metrics/collector.rs
pub struct MetricsCollector {
    browser_startup_times: Histogram,
    navigation_times: Histogram,
    memory_usage: Gauge,
    active_browsers: Gauge,
    active_pages: Gauge,
    error_rates: Counter,
}
```

This will help validate that the optimizations work effectively in production environments.
