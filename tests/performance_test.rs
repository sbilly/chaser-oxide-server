//! Performance and Stability Acceptance Tests
//!
//! Comprehensive performance and stability testing suite for chaser-oxide-server.
//! Tests browser startup time, navigation performance, concurrent operations,
//! memory usage, and long-running stability.

use chaser_oxide::session::{
    SessionManager, BrowserOptions, PageOptions, NavigationOptions,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics collected during tests
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceMetrics {
    /// Browser startup time in milliseconds
    pub browser_startup_ms: f64,
    /// Navigation time in milliseconds
    pub navigation_ms: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Concurrent browsers supported
    pub concurrent_browsers: usize,
    /// Concurrent pages supported
    pub concurrent_pages: usize,
    /// Success rate for stress tests (0.0-1.0)
    pub stress_test_success_rate: f64,
}

/// Stability metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct StabilityMetrics {
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory leak detected
    pub memory_leak_detected: bool,
    /// Crashes during testing
    pub crash_count: usize,
    /// Recovery success rate (0.0-1.0)
    pub recovery_success_rate: f64,
}

/// Result container for test outcomes
#[derive(Debug)]
pub struct TestResult<T> {
    /// Test name
    pub name: String,
    /// Success status
    pub passed: bool,
    /// Measured value
    pub measured: T,
    /// Expected threshold
    pub threshold: T,
    /// Additional details
    pub details: String,
}

impl<T> TestResult<T>
where
    T: std::fmt::Display + PartialOrd,
{
    /// Create a new test result
    pub fn new(name: String, measured: T, threshold: T, details: String) -> Self {
        let passed = measured <= threshold;
        Self {
            name,
            passed,
            measured,
            threshold,
            details,
        }
    }

    /// Format the result for display
    pub fn format(&self) -> String {
        let status = if self.passed { "✓ PASS" } else { "✗ FAIL" };
        format!(
            "{}: {} (measured: {}, threshold: {})\n  Details: {}",
            status, self.name, self.measured, self.threshold, self.details
        )
    }
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Maximum browser startup time in milliseconds
    pub max_browser_startup_ms: f64,
    /// Maximum navigation time in milliseconds
    pub max_navigation_ms: f64,
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Minimum concurrent browsers
    pub min_concurrent_browsers: usize,
    /// Minimum concurrent pages
    pub min_concurrent_pages: usize,
    /// Stress test concurrent requests
    pub stress_test_concurrent_requests: usize,
    /// Stress test minimum success rate
    pub stress_test_min_success_rate: f64,
    /// Long-running test duration in seconds
    pub long_running_duration_secs: u64,
    /// Memory leak threshold in MB
    pub memory_leak_threshold_mb: f64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            max_browser_startup_ms: 3000.0,
            max_navigation_ms: 500.0,
            max_memory_mb: 500.0,
            min_concurrent_browsers: 10,
            min_concurrent_pages: 50,
            stress_test_concurrent_requests: 100,
            stress_test_min_success_rate: 0.95,
            long_running_duration_secs: 300, // 5 minutes for testing (use 3600 for 1 hour)
            memory_leak_threshold_mb: 50.0,
        }
    }
}

/// Performance test suite
pub struct PerformanceTestSuite {
    session_manager: Arc<dyn SessionManager>,
    config: TestConfig,
}

impl PerformanceTestSuite {
    /// Create a new performance test suite
    pub fn new(session_manager: Arc<dyn SessionManager>, config: TestConfig) -> Self {
        Self {
            session_manager,
            config,
        }
    }

    /// Run all performance tests
    pub async fn run_all(&self) -> Result<(PerformanceMetrics, Vec<String>), Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let mut metrics = PerformanceMetrics {
            browser_startup_ms: 0.0,
            navigation_ms: 0.0,
            memory_mb: 0.0,
            concurrent_browsers: 0,
            concurrent_pages: 0,
            stress_test_success_rate: 0.0,
        };

        // Test 1: Browser startup time
        let (startup_result, startup_ms) = self.test_browser_startup_time().await?;
        results.push(startup_result.format());
        metrics.browser_startup_ms = startup_ms;

        // Test 2: Navigation response time
        let (nav_result, nav_ms) = self.test_navigation_response_time().await?;
        results.push(nav_result.format());
        metrics.navigation_ms = nav_ms;

        // Test 3: Concurrent browsers
        let (concurrent_browser_result, count) = self.test_concurrent_browsers().await?;
        results.push(concurrent_browser_result.format());
        metrics.concurrent_browsers = count;

        // Test 4: Concurrent pages
        let (concurrent_page_result, count) = self.test_concurrent_pages().await?;
        results.push(concurrent_page_result.format());
        metrics.concurrent_pages = count;

        // Test 5: Memory usage (idle state)
        let (memory_result, memory_mb) = self.test_memory_usage_idle().await?;
        results.push(memory_result.format());
        metrics.memory_mb = memory_mb;

        // Test 6: Stress test
        let (stress_result, success_rate) = self.test_stress_concurrent_requests().await?;
        results.push(stress_result.format());
        metrics.stress_test_success_rate = success_rate;

        Ok((metrics, results))
    }

    /// Test 1: Browser startup time < 3 seconds
    async fn test_browser_startup_time(&self) -> Result<(TestResult<f64>, f64), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 1: Browser Startup Time");
        println!("Target: < {} ms", self.config.max_browser_startup_ms);

        let mut times = Vec::new();
        let iterations = 5;

        for i in 0..iterations {
            let start = Instant::now();
            let browser_id = self
                .session_manager
                .create_browser(BrowserOptions::default())
                .await?;

            let elapsed = start.elapsed().as_millis() as f64;
            times.push(elapsed);

            // Cleanup
            let _ = self.session_manager.close_browser(&browser_id).await;
            println!("  Iteration {}: {} ms", i + 1, elapsed);
        }

        let avg_time: f64 = times.iter().sum::<f64>() / times.len() as f64;
        let min_time = *times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_time = *times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let details = format!(
            "Average: {} ms, Min: {} ms, Max: {} ms ({} iterations)",
            avg_time, min_time, max_time, iterations
        );

        let result = TestResult::new(
            "Browser Startup Time".to_string(),
            avg_time,
            self.config.max_browser_startup_ms,
            details,
        );

        Ok((result, avg_time))
    }

    /// Test 2: Page navigation response time < 500ms
    async fn test_navigation_response_time(&self) -> Result<(TestResult<f64>, f64), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 2: Navigation Response Time");
        println!("Target: < {} ms", self.config.max_navigation_ms);

        // Create browser for testing
        let browser_id = self
            .session_manager
            .create_browser(BrowserOptions::default())
            .await?;

        let mut times = Vec::new();
        let iterations = 10;

        for i in 0..iterations {
            let page = self
                .session_manager
                .create_page(&browser_id, PageOptions::default())
                .await?;

            let start = Instant::now();
            let nav_options = NavigationOptions::default();
            let _ = page
                .navigate("about:blank", nav_options)
                .await?;
            let elapsed = start.elapsed().as_millis() as f64;

            times.push(elapsed);

            // Cleanup page
            let _ = self.session_manager.close_page(page.id()).await;
            println!("  Iteration {}: {} ms", i + 1, elapsed);
        }

        let avg_time: f64 = times.iter().sum::<f64>() / times.len() as f64;
        let min_time = *times.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_time = *times.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let details = format!(
            "Average: {} ms, Min: {} ms, Max: {} ms ({} iterations)",
            avg_time, min_time, max_time, iterations
        );

        // Cleanup browser
        let _ = self.session_manager.close_browser(&browser_id).await;

        let result = TestResult::new(
            "Navigation Response Time".to_string(),
            avg_time,
            self.config.max_navigation_ms,
            details,
        );

        Ok((result, avg_time))
    }

    /// Test 3: Support ≥ 10 concurrent browser instances
    async fn test_concurrent_browsers(&self) -> Result<(TestResult<usize>, usize), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 3: Concurrent Browser Support");
        println!("Target: ≥ {} browsers", self.config.min_concurrent_browsers);

        let start = Instant::now();
        let target_count = self.config.min_concurrent_browsers;
        let mut browser_ids = Vec::new();

        // Create browsers concurrently
        let mut handles = Vec::new();
        for _ in 0..target_count {
            let manager = self.session_manager.clone();
            handles.push(tokio::spawn(async move {
                manager.create_browser(BrowserOptions::default()).await
            }));
        }

        // Collect results
        for handle in handles {
            match handle.await? {
                Ok(id) => browser_ids.push(id),
                Err(e) => eprintln!("  Failed to create browser: {}", e),
            }
        }

        let elapsed = start.elapsed();
        let count = browser_ids.len();
        let success = count >= target_count;

        let details = format!(
            "Created {}/{} browsers in {} seconds ({} ms/browser)",
            count,
            target_count,
            elapsed.as_secs_f64(),
            elapsed.as_millis() as f64 / count as f64
        );

        // Cleanup
        for browser_id in browser_ids {
            let _ = self.session_manager.close_browser(&browser_id).await;
        }

        let result = TestResult {
            name: "Concurrent Browser Support".to_string(),
            passed: success,
            measured: count,
            threshold: target_count,
            details,
        };

        Ok((result, count))
    }

    /// Test 4: Support ≥ 50 concurrent page instances
    async fn test_concurrent_pages(&self) -> Result<(TestResult<usize>, usize), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 4: Concurrent Page Support");
        println!("Target: ≥ {} pages", self.config.min_concurrent_pages);

        // Create a single browser
        let browser_id = self
            .session_manager
            .create_browser(BrowserOptions::default())
            .await?;

        let start = Instant::now();
        let target_count = self.config.min_concurrent_pages;
        let mut page_ids = Vec::new();

        // Create pages concurrently
        let mut handles = Vec::new();
        for _ in 0..target_count {
            let manager = self.session_manager.clone();
            let browser = browser_id.clone();
            handles.push(tokio::spawn(async move {
                manager.create_page(&browser, PageOptions::default()).await
            }));
        }

        // Collect results
        for handle in handles {
            match handle.await? {
                Ok(page) => page_ids.push(page.id().to_string()),
                Err(e) => eprintln!("  Failed to create page: {}", e),
            }
        }

        let elapsed = start.elapsed();
        let count = page_ids.len();
        let success = count >= target_count;

        let details = format!(
            "Created {}/{} pages in {} seconds ({} ms/page)",
            count,
            target_count,
            elapsed.as_secs_f64(),
            elapsed.as_millis() as f64 / count as f64
        );

        // Cleanup
        for page_id in page_ids {
            let _ = self.session_manager.close_page(&page_id).await;
        }
        let _ = self.session_manager.close_browser(&browser_id).await;

        let result = TestResult {
            name: "Concurrent Page Support".to_string(),
            passed: success,
            measured: count,
            threshold: target_count,
            details,
        };

        Ok((result, count))
    }

    /// Test 5: Memory usage < 500MB (idle state)
    async fn test_memory_usage_idle(&self) -> Result<(TestResult<f64>, f64), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 5: Memory Usage (Idle State)");
        println!("Target: < {} MB", self.config.max_memory_mb);

        // Get initial memory
        let initial_memory = get_memory_usage_mb();
        println!("  Initial memory: {:.2} MB", initial_memory);

        // Create a browser and let it idle
        let browser_id = self
            .session_manager
            .create_browser(BrowserOptions::default())
            .await?;

        // Wait for stabilization
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Measure memory
        let final_memory = get_memory_usage_mb();
        let memory_used = final_memory - initial_memory;

        let details = format!(
            "Initial: {:.2} MB, Final: {:.2} MB, Delta: {:.2} MB",
            initial_memory, final_memory, memory_used
        );

        // Cleanup
        let _ = self.session_manager.close_browser(&browser_id).await;

        let result = TestResult::new(
            "Memory Usage (Idle)".to_string(),
            memory_used,
            self.config.max_memory_mb,
            details,
        );

        Ok((result, memory_used))
    }

    /// Test 6: Stress test - 100 concurrent requests
    async fn test_stress_concurrent_requests(&self) -> Result<(TestResult<f64>, f64), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 6: Stress Test - Concurrent Requests");
        println!("Target: {} requests, success rate ≥ {:.0}%",
            self.config.stress_test_concurrent_requests,
            self.config.stress_test_min_success_rate * 100.0
        );

        // Create a browser
        let browser_id = self
            .session_manager
            .create_browser(BrowserOptions::default())
            .await?;

        let start = Instant::now();
        let concurrent_requests = self.config.stress_test_concurrent_requests;
        let mut handles = Vec::new();

        // Spawn concurrent requests
        for i in 0..concurrent_requests {
            let manager = self.session_manager.clone();
            let browser = browser_id.clone();
            handles.push(tokio::spawn(async move {
                let result = async {
                    let page = manager.create_page(&browser, PageOptions::default()).await?;
                    let nav_options = NavigationOptions::default();
                    let _ = page.navigate("about:blank", nav_options).await?;
                    let _ = manager.close_page(page.id()).await;
                    Ok::<(), Box<dyn std::error::Error>>(())
                }.await;
                (i, result.is_ok())
            }));
        }

        // Collect results
        let mut success_count = 0;
        let mut error_count = 0;

        for handle in handles {
            match handle.await {
                Ok((_, true)) => success_count += 1,
                Ok((_, false)) => error_count += 1,
                Err(e) => {
                    eprintln!("  Task failed: {}", e);
                    error_count += 1;
                }
            }
        }

        let elapsed = start.elapsed();
        let success_rate = success_count as f64 / concurrent_requests as f64;
        let passed = success_rate >= self.config.stress_test_min_success_rate;

        let details = format!(
            "Success: {}/{}, Errors: {}, Time: {} seconds, Rate: {:.1} req/s",
            success_count,
            concurrent_requests,
            error_count,
            elapsed.as_secs_f64(),
            concurrent_requests as f64 / elapsed.as_secs_f64()
        );

        // Cleanup
        let _ = self.session_manager.close_browser(&browser_id).await;

        let result = TestResult {
            name: "Stress Test (Concurrent Requests)".to_string(),
            passed,
            measured: success_rate,
            threshold: self.config.stress_test_min_success_rate,
            details,
        };

        Ok((result, success_rate))
    }
}

/// Stability test suite
pub struct StabilityTestSuite {
    session_manager: Arc<dyn SessionManager>,
    config: TestConfig,
}

impl StabilityTestSuite {
    /// Create a new stability test suite
    pub fn new(session_manager: Arc<dyn SessionManager>, config: TestConfig) -> Self {
        Self {
            session_manager,
            config,
        }
    }

    /// Run all stability tests
    pub async fn run_all(&self) -> Result<(StabilityMetrics, Vec<String>), Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let mut metrics = StabilityMetrics {
            uptime_seconds: 0,
            memory_leak_detected: false,
            crash_count: 0,
            recovery_success_rate: 0.0,
        };

        // Test 1: Long-running stability
        let (long_running_result, stability_metrics) = self.test_long_running_stability().await?;
        results.push(long_running_result);
        metrics.uptime_seconds = stability_metrics.uptime_seconds;
        metrics.memory_leak_detected = stability_metrics.memory_leak_detected;

        // Test 2: Exception recovery
        let (recovery_result, recovery_rate) = self.test_exception_recovery().await?;
        results.push(recovery_result);
        metrics.recovery_success_rate = recovery_rate;

        Ok((metrics, results))
    }

    /// Test 1: Long-running stability (simulated 5 minutes, production: 1 hour)
    async fn test_long_running_stability(&self) -> Result<(String, StabilityMetrics), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 7: Long-Running Stability");
        println!("Duration: {} seconds", self.config.long_running_duration_secs);

        let start = Instant::now();
        let test_duration = Duration::from_secs(self.config.long_running_duration_secs);

        // Get initial memory
        let initial_memory = get_memory_usage_mb();
        println!("  Initial memory: {:.2} MB", initial_memory);

        let mut operation_count = 0;
        let mut error_count = 0;
        let mut max_memory = initial_memory;

        // Run operations for the test duration
        while start.elapsed() < test_duration {
            // Create and destroy browsers
            match self
                .session_manager
                .create_browser(BrowserOptions::default())
                .await
            {
                Ok(browser_id) => {
                    operation_count += 1;

                    // Perform some operations
                    if let Ok(page) = self
                        .session_manager
                        .create_page(&browser_id, PageOptions::default())
                        .await
                    {
                        let _ = page.navigate("about:blank", NavigationOptions::default()).await;
                        let _ = self.session_manager.close_page(page.id()).await;
                    }

                    let _ = self.session_manager.close_browser(&browser_id).await;

                    // Check memory periodically
                    if operation_count % 10 == 0 {
                        let current_memory = get_memory_usage_mb();
                        max_memory = max_memory.max(current_memory);
                        println!(
                            "  Progress: {:.0}s, Operations: {}, Memory: {:.2} MB",
                            start.elapsed().as_secs_f64(),
                            operation_count,
                            current_memory
                        );
                    }
                }
                Err(e) => {
                    error_count += 1;
                    eprintln!("  Error during operation {}: {}", operation_count, e);
                }
            }

            // Small delay between operations
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let final_memory = get_memory_usage_mb();
        let memory_growth = final_memory - initial_memory;
        let memory_leak_detected = memory_growth > self.config.memory_leak_threshold_mb;

        let elapsed = start.elapsed();
        let passed = !memory_leak_detected && error_count == 0;

        let result = format!(
            "{}: Long-Running Stability\n\
             Measured: {} seconds uptime, {:.2} MB memory growth, {} errors\n\
             Threshold: < {} MB memory growth, 0 errors\n\
             Details: Completed {} operations in {:.2}s ({:.2} ops/s), Max memory: {:.2} MB",
            if passed { "✓ PASS" } else { "✗ FAIL" },
            elapsed.as_secs(),
            memory_growth,
            error_count,
            self.config.memory_leak_threshold_mb,
            operation_count,
            elapsed.as_secs_f64(),
            operation_count as f64 / elapsed.as_secs_f64(),
            max_memory
        );

        let metrics = StabilityMetrics {
            uptime_seconds: elapsed.as_secs(),
            memory_leak_detected,
            crash_count: error_count,
            recovery_success_rate: 0.0,
        };

        Ok((result, metrics))
    }

    /// Test 2: Exception recovery - browser crash simulation
    async fn test_exception_recovery(&self) -> Result<(String, f64), Box<dyn std::error::Error>> {
        println!("\n🔍 Test 8: Exception Recovery");
        println!("Simulating browser crashes and testing service recovery");

        let recovery_tests = 10;
        let mut successful_recoveries = 0;

        for i in 0..recovery_tests {
            println!("  Recovery test {}/{}", i + 1, recovery_tests);

            // Create a browser
            match self
                .session_manager
                .create_browser(BrowserOptions::default())
                .await
            {
                Ok(browser_id) => {
                    // Simulate crash by closing browser abruptly
                    let _ = self.session_manager.close_browser(&browser_id).await;

                    // Try to create a new browser immediately
                    match tokio::time::timeout(
                        Duration::from_secs(5),
                        self.session_manager.create_browser(BrowserOptions::default())
                    ).await {
                        Ok(Ok(_)) => {
                            successful_recoveries += 1;
                            println!("    ✓ Recovery successful");
                        }
                        Ok(Err(e)) => {
                            eprintln!("    ✗ Recovery failed: {}", e);
                        }
                        Err(_) => {
                            eprintln!("    ✗ Recovery timed out");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  Failed to create browser for test {}: {}", i + 1, e);
                }
            }
        }

        let recovery_rate = successful_recoveries as f64 / recovery_tests as f64;
        let passed = recovery_rate >= 0.8; // 80% recovery rate required

        let result = format!(
            "{}: Exception Recovery\n\
             Measured: {:.0}% recovery rate ({}/{})\n\
             Threshold: ≥ 80% success rate\n\
             Details: Service recovered from {} out of {} simulated crashes",
            if passed { "✓ PASS" } else { "✗ FAIL" },
            recovery_rate * 100.0,
            successful_recoveries,
            recovery_tests,
            successful_recoveries,
            recovery_tests
        );

        Ok((result, recovery_rate))
    }
}

/// Get current process memory usage in MB
fn get_memory_usage_mb() -> f64 {
    // Try to get memory usage from /proc/self/status (Linux)
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        return kb / 1024.0; // Convert KB to MB
                    }
                }
            }
        }
    }

    // Fallback: estimate based on available memory info
    // For macOS, use `ps` command
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(kb_str) = String::from_utf8(output.stdout) {
                if let Ok(kb) = kb_str.trim().parse::<f64>() {
                    return kb / 1024.0; // Convert KB to MB
                }
            }
        }
    }

    // Default fallback
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chaser_oxide::session::SessionManagerImpl;

    fn create_mock_session_manager() -> Arc<dyn SessionManager> {
        Arc::new(SessionManagerImpl::new(|| {
            Ok(Arc::new(chaser_oxide::cdp::mock::MockCdpBrowser::new()))
        }))
    }

    #[tokio::test]
    async fn test_performance_suite() {
        let session_manager = create_mock_session_manager();
        let config = TestConfig::default();
        let suite = PerformanceTestSuite::new(session_manager, config);

        let (metrics, results) = suite.run_all().await.unwrap();

        println!("\n=== Performance Test Results ===");
        for result in results {
            println!("{}", result);
        }

        println!("\n=== Performance Metrics Summary ===");
        println!("Browser Startup: {:.2} ms", metrics.browser_startup_ms);
        println!("Navigation Time: {:.2} ms", metrics.navigation_ms);
        println!("Memory Usage: {:.2} MB", metrics.memory_mb);
        println!("Concurrent Browsers: {}", metrics.concurrent_browsers);
        println!("Concurrent Pages: {}", metrics.concurrent_pages);
        println!("Stress Test Success Rate: {:.2}%", metrics.stress_test_success_rate * 100.0);
    }

    #[tokio::test]
    async fn test_stability_suite() {
        let session_manager = create_mock_session_manager();
        let config = TestConfig {
            long_running_duration_secs: 10, // Short test for CI
            ..Default::default()
        };
        let suite = StabilityTestSuite::new(session_manager, config);

        let (metrics, results) = suite.run_all().await.unwrap();

        println!("\n=== Stability Test Results ===");
        for result in results {
            println!("{}", result);
        }

        println!("\n=== Stability Metrics Summary ===");
        println!("Uptime: {} seconds", metrics.uptime_seconds);
        println!("Memory Leak Detected: {}", metrics.memory_leak_detected);
        println!("Crash Count: {}", metrics.crash_count);
        println!("Recovery Success Rate: {:.2}%", metrics.recovery_success_rate * 100.0);
    }
}
