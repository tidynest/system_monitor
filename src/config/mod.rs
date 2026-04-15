//! Application configuration module
//!
//! Manages configuration settings from environment variables with sensible defaults.
//!
//! # Environment Variables
//! - `MONITOR_HOST` - Server host address (default: 127.0.0.1)
//! - `MONITOR_PORT` - Server port (default: 8080)
//! - `MONITOR_UPDATE_INTERVAL` - Metrics update interval in seconds (default: 1)
//! - `MONITOR_MAX_PROCESSES` - Maximum processes to show in lists (default: 5)
//! - `MONITOR_LOG_LEVEL` - Log level: error, warn, info, debug (default: warn)

use std::time::Duration;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Interval for metrics updates
    pub update_interval: Duration,
    /// Maximum number of processes to display
    pub max_processes_shown: usize,
    /// Log level
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            update_interval: Duration::from_secs(1),
            max_processes_shown: 5,
            log_level: "warn".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Falls back to defaults if environment variables are not set or invalid.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("MONITOR_HOST") {
            config.host = host;
        }

        if let Ok(port) = std::env::var("MONITOR_PORT") {
            if let Ok(p) = port.parse() {
                config.port = p;
            } else {
                log::warn!("Invalid MONITOR_PORT value: {}", port);
            }
        }

        if let Ok(update_interval) = std::env::var("MONITOR_UPDATE_INTERVAL") {
            if let Ok(secs) = update_interval.parse::<u64>() {
                config.update_interval = Duration::from_secs(secs);
            } else {
                log::warn!("Invalid MONITOR_UPDATE_INTERVAL value: {}", update_interval);
            }
        }

        if let Ok(max_processes) = std::env::var("MONITOR_MAX_PROCESSES") {
            if let Ok(max) = max_processes.parse() {
                config.max_processes_shown = max;
            } else {
                log::warn!("Invalid MONITOR_MAX_PROCESSES value: {}", max_processes);
            }
        }

        if let Ok(log_level) = std::env::var("MONITOR_LOG_LEVEL") {
            let valid_levels = ["error", "warn", "info", "debug", "trace"];
            if valid_levels.contains(&log_level.as_str()) {
                config.log_level = log_level;
            } else {
                log::warn!("Invalid MONITOR_LOG_LEVEL value: {}", log_level);
            }
        }

        config
    }

    /// Get the bind address for the server
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the log filter string for env_logger
    pub fn log_filter(&self) -> String {
        format!("{},system_monitor={}", self.log_level, self.log_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.update_interval, Duration::from_secs(1));
        assert_eq!(config.max_processes_shown, 5);
        assert_eq!(config.log_level, "warn");
    }

    #[test]
    fn test_bind_address() {
        let config = Config::default();
        assert_eq!(config.bind_address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_log_filter() {
        let config = Config::default();
        assert_eq!(config.log_filter(), "warn,system_monitor=warn");
    }

    #[test]
    fn test_env_override() {
        unsafe {
            std::env::set_var("MONITOR_PORT", "9000");
        }
        let config = Config::from_env();
        assert_eq!(config.port, 9000);
        unsafe {
            std::env::remove_var("MONITOR_PORT");
        }
    }
}
