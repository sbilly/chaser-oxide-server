//! Configuration management for Chaser-Oxide

use crate::{Error, Result};
use serde::Deserialize;
use std::env;

/// Macro for parsing environment variables with type safety and consistent error handling
macro_rules! parse_env_var {
    ($config:ident, $field:ident, $env_var:expr, $ty:ty) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = value
                .parse::<$ty>()
                .map_err(|_| Error::configuration(concat!("Invalid ", $env_var)))?;
        }
    };

    ($config:ident, $field:ident, $env_var:expr) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = value;
        }
    };

    (opt $config:ident, $field:ident, $env_var:expr) => {
        if let Ok(value) = env::var($env_var) {
            $config.$field = Some(value);
        }
    };
}

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

        // Use macro for DRY environment variable parsing
        parse_env_var!(config, host, "CHASER_HOST");
        parse_env_var!(config, port, "CHASER_PORT", u16);
        parse_env_var!(opt config, chrome_path, "CHASER_CHROME_PATH");
        parse_env_var!(opt config, chrome_data_dir, "CHASER_DATA_DIR");
        parse_env_var!(config, max_browsers, "CHASER_MAX_BROWSERS", usize);
        parse_env_var!(config, max_pages_per_browser, "CHASER_MAX_PAGES", usize);
        parse_env_var!(config, session_timeout, "CHASER_SESSION_TIMEOUT", u64);
        parse_env_var!(config, default_timeout, "CHASER_DEFAULT_TIMEOUT", u64);
        parse_env_var!(config, stealth_enabled, "CHASER_STEALTH", bool);
        parse_env_var!(config, log_level, "CHASER_LOG_LEVEL");

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
