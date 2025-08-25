// src/models/network.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkMetrics {
    pub total_received_mb: f64,
    pub total_transmitted_mb: f64,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub received_mb: f64,
    pub transmitted_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_metrics_creation() {
        let interface = NetworkInterface {
            name: "eth0".to_string(),
            received_mb: 100.0,
            transmitted_mb: 50.0,
        };

        assert_eq!(interface.name, "eth0");
        assert_eq!(interface.received_mb, 100.0);
    }
}
