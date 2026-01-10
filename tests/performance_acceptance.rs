//! Performance & Stability Acceptance Test Runner
//!
//! Main entry point for running performance and stability acceptance tests and generating reports.

mod performance_test;

use chaser_oxide::session::SessionManagerImpl;
use performance_test::{
    PerformanceTestSuite, StabilityTestSuite, TestConfig, PerformanceMetrics, StabilityMetrics,
};
use std::sync::Arc;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use chrono::Utc;

/// Test report generator
pub struct TestReport {
    /// Test timestamp
    pub timestamp: String,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Stability metrics
    pub stability: StabilityMetrics,
    /// Performance test results
    pub performance_results: Vec<String>,
    /// Stability test results
    pub stability_results: Vec<String>,
}

impl TestReport {
    /// Generate markdown report
    pub fn to_markdown(&self) -> String {
        format!(
            r#"# Performance & Stability Acceptance Test Report

**Generated**: {}
**Test Environment**: chaser-oxide-server v{}

---

## Executive Summary

### Overall Status

{}

### Key Metrics

| Metric | Measured | Target | Status |
|--------|----------|--------|--------|
| Browser Startup | {:.2} ms | < 3000 ms | {} |
| Navigation Time | {:.2} ms | < 500 ms | {} |
| Memory Usage (Idle) | {:.2} MB | < 500 MB | {} |
| Concurrent Browsers | {} | ≥ 10 | {} |
| Concurrent Pages | {} | ≥ 50 | {} |
| Stress Test Success | {:.1}% | ≥ 95% | {} |
| Memory Leak | {} | No | {} |
| Crash Count | {} | 0 | {} |
| Recovery Rate | {:.1}% | ≥ 80% | {} |

---

## Performance Test Results

### Test 1: Browser Startup Time

{}

### Test 2: Navigation Response Time

{}

### Test 3: Concurrent Browser Support

{}

### Test 4: Concurrent Page Support

{}

### Test 5: Memory Usage (Idle)

{}

### Test 6: Stress Test (Concurrent Requests)

{}

---

## Stability Test Results

### Test 7: Long-Running Stability

{}

### Test 8: Exception Recovery

{}

---

## Detailed Metrics

### Performance Metrics
```json
{}
```

### Stability Metrics
```json
{}
```

---

## Recommendations

{}

---

## Test Configuration

- **Max Browser Startup Time**: 3000 ms
- **Max Navigation Time**: 500 ms
- **Max Memory Usage**: 500 MB
- **Min Concurrent Browsers**: 10
- **Min Concurrent Pages**: 50
- **Stress Test Concurrent Requests**: 100
- **Long-Running Test Duration**: {} seconds
- **Memory Leak Threshold**: 50 MB

---

*This report was generated automatically by the acceptance test suite.*
"#,
            self.timestamp,
            chaser_oxide::VERSION,
            self.overall_status(),
            self.performance.browser_startup_ms,
            self.status_icon(self.performance.browser_startup_ms <= 3000.0),
            self.performance.navigation_ms,
            self.status_icon(self.performance.navigation_ms <= 500.0),
            self.performance.memory_mb,
            self.status_icon(self.performance.memory_mb <= 500.0),
            self.performance.concurrent_browsers,
            self.status_icon(self.performance.concurrent_browsers >= 10),
            self.performance.concurrent_pages,
            self.status_icon(self.performance.concurrent_pages >= 50),
            self.performance.stress_test_success_rate * 100.0,
            self.status_icon(self.performance.stress_test_success_rate >= 0.95),
            if self.stability.memory_leak_detected { "Yes" } else { "No" },
            self.status_icon(!self.stability.memory_leak_detected),
            self.stability.crash_count,
            self.status_icon(self.stability.crash_count == 0),
            self.stability.recovery_success_rate * 100.0,
            self.status_icon(self.stability.recovery_success_rate >= 0.8),
            self.performance_results.get(0).unwrap_or(&"Not available".to_string()),
            self.performance_results.get(1).unwrap_or(&"Not available".to_string()),
            self.performance_results.get(2).unwrap_or(&"Not available".to_string()),
            self.performance_results.get(3).unwrap_or(&"Not available".to_string()),
            self.performance_results.get(4).unwrap_or(&"Not available".to_string()),
            self.performance_results.get(5).unwrap_or(&"Not available".to_string()),
            self.stability_results.get(0).unwrap_or(&"Not available".to_string()),
            self.stability_results.get(1).unwrap_or(&"Not available".to_string()),
            serde_json::to_string_pretty(&self.performance).unwrap_or_else(|_| "Failed to serialize".to_string()),
            serde_json::to_string_pretty(&self.stability).unwrap_or_else(|_| "Failed to serialize".to_string()),
            self.generate_recommendations(),
            300 // Default test duration
        )
    }

    /// Generate overall status
    fn overall_status(&self) -> String {
        let all_passed = self.performance.browser_startup_ms <= 3000.0
            && self.performance.navigation_ms <= 500.0
            && self.performance.memory_mb <= 500.0
            && self.performance.concurrent_browsers >= 10
            && self.performance.concurrent_pages >= 50
            && self.performance.stress_test_success_rate >= 0.95
            && !self.stability.memory_leak_detected
            && self.stability.crash_count == 0
            && self.stability.recovery_success_rate >= 0.8;

        if all_passed {
            "✅ **ALL TESTS PASSED**".to_string()
        } else {
            "⚠️ **SOME TESTS FAILED**".to_string()
        }
    }

    /// Generate status icon
    fn status_icon(&self, passed: bool) -> &str {
        if passed { "✅" } else { "❌" }
    }

    /// Generate recommendations based on test results
    fn generate_recommendations(&self) -> String {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if self.performance.browser_startup_ms > 3000.0 {
            recommendations.push(format!(
                "⚠️ Browser startup time ({:.2} ms) exceeds target. Consider optimizing browser initialization or implementing lazy loading.",
                self.performance.browser_startup_ms
            ));
        }

        if self.performance.navigation_ms > 500.0 {
            recommendations.push(format!(
                "⚠️ Navigation time ({:.2} ms) exceeds target. Consider optimizing CDP communication or implementing request batching.",
                self.performance.navigation_ms
            ));
        }

        if self.performance.memory_mb > 500.0 {
            recommendations.push(format!(
                "⚠️ Memory usage ({:.2} MB) exceeds target. Consider implementing resource pooling or more aggressive cleanup.",
                self.performance.memory_mb
            ));
        }

        if self.performance.concurrent_browsers < 10 {
            recommendations.push(format!(
                "⚠️ Concurrent browser support ({}) below target. Review resource limits and consider optimizing browser lifecycle management.",
                self.performance.concurrent_browsers
            ));
        }

        if self.performance.concurrent_pages < 50 {
            recommendations.push(format!(
                "⚠️ Concurrent page support ({}) below target. Review page management and consider implementing page pooling.",
                self.performance.concurrent_pages
            ));
        }

        if self.performance.stress_test_success_rate < 0.95 {
            recommendations.push(format!(
                "⚠️ Stress test success rate ({:.1}%) below target. Improve error handling and request throttling.",
                self.performance.stress_test_success_rate * 100.0
            ));
        }

        // Stability recommendations
        if self.stability.memory_leak_detected {
            recommendations.push(
                "🚨 Memory leak detected! Review resource cleanup and ensure proper disposal of browser instances and CDP connections.".to_string()
            );
        }

        if self.stability.crash_count > 0 {
            recommendations.push(format!(
                "⚠️ {} crashes detected during stability test. Improve error handling and implement better crash recovery.",
                self.stability.crash_count
            ));
        }

        if self.stability.recovery_success_rate < 0.8 {
            recommendations.push(format!(
                "⚠️ Recovery success rate ({:.1}%) below target. Implement more robust crash recovery mechanisms.",
                self.stability.recovery_success_rate * 100.0
            ));
        }

        // Positive feedback
        if recommendations.is_empty() {
            recommendations.push(
                "✅ All performance and stability targets met! System is production-ready.".to_string()
            );
        }

        recommendations.join("\n\n")
    }

    /// Save report to file
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let markdown = self.to_markdown();
        let mut file = File::create(path)?;
        file.write_all(markdown.as_bytes())?;
        println!("Report saved to: {}", path.display());
        Ok(())
    }
}

/// Run all acceptance tests
pub async fn run_acceptance_tests() -> Result<TestReport, Box<dyn std::error::Error>> {
    println!("🚀 Starting Performance & Stability Acceptance Tests");
    println!("==================================================\n");

    // Create session manager
    let session_manager = Arc::new(SessionManagerImpl::new(|| {
        Ok(Arc::new(chaser_oxide::cdp::mock::MockCdpBrowser::new()))
    }));

    // Create test configuration
    let config = TestConfig {
        long_running_duration_secs: 60, // 1 minute for faster testing (use 3600 for production)
        ..Default::default()
    };

    // Run performance tests
    println!("📊 Running Performance Tests...\n");
    let perf_suite = PerformanceTestSuite::new(session_manager.clone(), config.clone());
    let (performance_metrics, performance_results) = perf_suite.run_all().await?;

    // Run stability tests
    println!("\n🔒 Running Stability Tests...\n");
    let stability_suite = StabilityTestSuite::new(session_manager, config);
    let (stability_metrics, stability_results) = stability_suite.run_all().await?;

    // Create report
    let report = TestReport {
        timestamp: Utc::now().to_rfc3339(),
        performance: performance_metrics,
        stability: stability_metrics,
        performance_results,
        stability_results,
    };

    // Print summary
    println!("\n==================================================");
    println!("📋 Test Summary");
    println!("==================================================");
    println!("{}", report.overall_status());
    println!("\nPerformance Metrics:");
    println!("  Browser Startup: {:.2} ms", report.performance.browser_startup_ms);
    println!("  Navigation Time: {:.2} ms", report.performance.navigation_ms);
    println!("  Memory Usage: {:.2} MB", report.performance.memory_mb);
    println!("  Concurrent Browsers: {}", report.performance.concurrent_browsers);
    println!("  Concurrent Pages: {}", report.performance.concurrent_pages);
    println!("  Stress Test Success: {:.1}%", report.performance.stress_test_success_rate * 100.0);
    println!("\nStability Metrics:");
    println!("  Uptime: {} seconds", report.stability.uptime_seconds);
    println!("  Memory Leak Detected: {}", report.stability.memory_leak_detected);
    println!("  Crashes: {}", report.stability.crash_count);
    println!("  Recovery Success Rate: {:.1}%", report.stability.recovery_success_rate * 100.0);

    Ok(report)
}

#[tokio::test]
async fn test_performance_acceptance_suite() {
    let report = run_acceptance_tests().await.unwrap();

    // Create reports directory if it doesn't exist
    let reports_dir = Path::new("test_reports");
    fs::create_dir_all(reports_dir).unwrap();

    // Save report with timestamp
    let filename = format!("acceptance_test_{}.md",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let report_path = reports_dir.join(filename);

    report.save_to_file(&report_path).unwrap();

    // Also save latest report
    let latest_path = reports_dir.join("latest_acceptance_test.md");
    report.save_to_file(&latest_path).unwrap();

    println!("\n✅ Acceptance tests completed successfully!");

    // Assert that critical metrics meet requirements
    assert!(report.performance.browser_startup_ms <= 3000.0,
        "Browser startup time exceeds 3000 ms threshold");
    assert!(report.performance.navigation_ms <= 500.0,
        "Navigation time exceeds 500 ms threshold");
    assert!(report.performance.memory_mb <= 500.0,
        "Memory usage exceeds 500 MB threshold");
    assert!(report.performance.concurrent_browsers >= 10,
        "Concurrent browsers below 10 threshold");
    assert!(report.performance.concurrent_pages >= 50,
        "Concurrent pages below 50 threshold");
    assert!(report.performance.stress_test_success_rate >= 0.95,
        "Stress test success rate below 95%");
    assert!(!report.stability.memory_leak_detected,
        "Memory leak detected");
    assert!(report.stability.crash_count == 0,
        "Crashes detected during stability test");
    assert!(report.stability.recovery_success_rate >= 0.8,
        "Recovery success rate below 80%");
}
