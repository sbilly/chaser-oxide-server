//! Configuration management for Chaser-Oxide

use crate::{Error, Result};
use serde::Deserialize;
use std::{env, time::Duration};

/// Helper to parse optional environment variable
fn parse_optional<T>(key: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    env::var(key).ok().and_then(|v| v.parse().ok())
}

/// Helper to parse required environment variable with default
fn parse_with_default<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    parse_optional(key).unwrap_or(default)
}

/// Helper to parse optional string environment variable
fn parse_optional_string(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Chrome executable path (optional, uses system default if not set)
    pub chrome_path: Option<String>,

    /// Chrome data directory (optional, uses temp directory if not set)
    pub chrome_data_dir: Option<String>,

    /// Session timeout in seconds
    pub session_timeout: u64,

    /// Default operation timeout in milliseconds
    pub default_timeout: u64,

    /// CDP port range start (for ProcessManager)
    pub cdp_port_start: u16,

    /// CDP port range end (for ProcessManager)
    pub cdp_port_end: u16,

    /// Health check interval in seconds (for ProcessManager)
    pub health_check_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            chrome_path: None,
            chrome_data_dir: None,
            session_timeout: 3600,
            default_timeout: 30000,
            cdp_port_start: 9000,
            cdp_port_end: 9900,
            health_check_interval: 30,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: parse_optional_string("CHASER_HOST").unwrap_or_else(|| Config::default().host),
            port: parse_with_default("CHASER_PORT", 50051),
            chrome_path: parse_optional_string("CHASER_BROWSER_PATH"),
            chrome_data_dir: parse_optional_string("CHASER_DATA_DIR"),
            session_timeout: parse_with_default("CHASER_SESSION_TIMEOUT", 3600),
            default_timeout: parse_with_default("CHASER_DEFAULT_TIMEOUT", 30000),
            cdp_port_start: parse_with_default("CHASER_CDP_PORT_START", 9000),
            cdp_port_end: parse_with_default("CHASER_CDP_PORT_END", 9900),
            health_check_interval: parse_with_default("CHASER_HEALTH_CHECK_INTERVAL", 30),
        })
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::configuration(format!("Failed to read config file: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| Error::configuration(format!("Failed to parse config: {}", e)))
    }

    /// Get session timeout as Duration
    pub fn session_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.session_timeout)
    }

    /// Get default timeout as Duration
    pub fn default_timeout_duration(&self) -> Duration {
        Duration::from_millis(self.default_timeout)
    }

    /// Get health check interval as Duration
    pub fn health_check_interval_duration(&self) -> Duration {
        Duration::from_secs(self.health_check_interval)
    }
}
