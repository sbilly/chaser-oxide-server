//! Configuration management for Chaser-Oxide

use crate::{Error, Result};
use serde::Deserialize;
use std::env;

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Chrome executable path
    pub chrome_path: Option<String>,

    /// Chrome data directory
    pub chrome_data_dir: Option<String>,

    /// Maximum concurrent browsers
    pub max_browsers: usize,

    /// Maximum concurrent pages per browser
    pub max_pages_per_browser: usize,

    /// Session timeout in seconds
    pub session_timeout: u64,

    /// Default timeout for operations in milliseconds
    pub default_timeout: u64,

    /// Enable stealth mode by default
    pub stealth_enabled: bool,

    /// Log level
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 50051,
            chrome_path: None,
            chrome_data_dir: None,
            max_browsers: 10,
            max_pages_per_browser: 20,
            session_timeout: 3600,
            default_timeout: 30000,
            stealth_enabled: true,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(host) = env::var("CHASER_HOST") {
            config.host = host;
        }

        if let Ok(port) = env::var("CHASER_PORT") {
            config.port = port
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_PORT"))?;
        }

        if let Ok(chrome_path) = env::var("CHASER_CHROME_PATH") {
            config.chrome_path = Some(chrome_path);
        }

        if let Ok(data_dir) = env::var("CHASER_DATA_DIR") {
            config.chrome_data_dir = Some(data_dir);
        }

        if let Ok(max_browsers) = env::var("CHASER_MAX_BROWSERS") {
            config.max_browsers = max_browsers
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_MAX_BROWSERS"))?;
        }

        if let Ok(max_pages) = env::var("CHASER_MAX_PAGES") {
            config.max_pages_per_browser = max_pages
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_MAX_PAGES"))?;
        }

        if let Ok(timeout) = env::var("CHASER_SESSION_TIMEOUT") {
            config.session_timeout = timeout
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_SESSION_TIMEOUT"))?;
        }

        if let Ok(default_timeout) = env::var("CHASER_DEFAULT_TIMEOUT") {
            config.default_timeout = default_timeout
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_DEFAULT_TIMEOUT"))?;
        }

        if let Ok(stealth) = env::var("CHASER_STEALTH") {
            config.stealth_enabled = stealth
                .parse()
                .map_err(|_| Error::configuration("Invalid CHASER_STEALTH"))?;
        }

        if let Ok(log_level) = env::var("CHASER_LOG_LEVEL") {
            config.log_level = log_level;
        }

        Ok(config)
    }

    /// Load configuration from a file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::configuration(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::configuration(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }
}
