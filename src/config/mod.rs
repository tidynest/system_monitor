// src/config/mod.rs

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]  // Will be used for metric collection timing
    pub update_interval: Duration,
    #[allow(dead_code)]  // Will be used in process metrics
    pub max_processes_shown: usize,
    #[allow(dead_code)]  // Will be used in websocket handler
    pub websocket_heartbeat: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            update_interval: Duration::from_secs(1),
            max_processes_shown: 5,
            websocket_heartbeat: Duration::from_secs(30),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("MONITOR_HOST") {
            config.host = host;
        }

        if let Ok(port) = std::env::var("MONITOR_PORT") {
            if let Ok(p) = port.parse() {
                config.port = p;
            }
        }

        if let Ok(update_interval) = std::env::var("MONITOR_UPDATE_INTERVAL") {
            if let Ok(secs) = update_interval.parse() {
                config.update_interval = Duration::from_secs(secs);
            }
        }

        if let Ok(max_processes) = std::env::var("MONITOR_MAX_PROCESSES") {
            if let Ok(max) = max_processes.parse() {
                config.max_processes_shown = max;
            }
        }

        config
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
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
    }

    #[test]
    fn test_bind_address() {
        let config = Config::default();
        assert_eq!(config.bind_address(), "127.0.0.1:8080");
    }
}