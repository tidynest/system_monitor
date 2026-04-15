//! Network metrics data model
//!
//! Structures for representing network interface statistics.

use serde::{Deserialize, Serialize};

/// Aggregated network metrics for all interfaces
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkMetrics {
    /// Total data received across all interfaces in MB
    pub total_received_mb: f64,
    /// Total data transmitted across all interfaces in MB
    pub total_transmitted_mb: f64,
    /// List of individual network interfaces
    pub interfaces: Vec<NetworkInterface>,
}

/// Network statistics for a single interface
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0", "wlan0")
    pub name: String,
    /// Data received in MB
    pub received_mb: f64,
    /// Data transmitted in MB
    pub transmitted_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_interface_creation() {
        let interface = NetworkInterface {
            name: "eth0".to_string(),
            received_mb: 100.0,
            transmitted_mb: 50.0,
        };

        assert_eq!(interface.name, "eth0");
        assert_eq!(interface.received_mb, 100.0);
        assert_eq!(interface.transmitted_mb, 50.0);
    }

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics {
            total_received_mb: 150.0,
            total_transmitted_mb: 75.0,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    received_mb: 100.0,
                    transmitted_mb: 50.0,
                },
                NetworkInterface {
                    name: "lo".to_string(),
                    received_mb: 50.0,
                    transmitted_mb: 25.0,
                },
            ],
        };

        assert_eq!(metrics.total_received_mb, 150.0);
        assert_eq!(metrics.total_transmitted_mb, 75.0);
        assert_eq!(metrics.interfaces.len(), 2);
    }
}
