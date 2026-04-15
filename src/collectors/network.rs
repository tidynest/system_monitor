//! Network metrics collector
//!
//! Collects network interface statistics including transmitted and received data.

use crate::models::network::{NetworkInterface, NetworkMetrics};
use sysinfo::Networks;

/// Collect metrics for all network interfaces
///
/// Returns aggregated network statistics and per-interface data
/// for all active network interfaces.
pub fn collect_network_metrics() -> NetworkMetrics {
    let networks = Networks::new_with_refreshed_list();

    let mut interfaces = Vec::new();
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for (interface_name, network) in &networks {
        let rx = network.total_received();
        let tx = network.total_transmitted();
        total_rx += rx;
        total_tx += tx;

        interfaces.push(NetworkInterface {
            name: interface_name.to_string(),
            received_mb: rx as f64 / 1_048_576.0,
            transmitted_mb: tx as f64 / 1_048_576.0,
        });
    }

    NetworkMetrics {
        total_received_mb: total_rx as f64 / 1_048_576.0,
        total_transmitted_mb: total_tx as f64 / 1_048_576.0,
        interfaces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics_collection() {
        let metrics = collect_network_metrics();

        assert!(metrics.total_received_mb >= 0.0);
        assert!(metrics.total_transmitted_mb >= 0.0);

        // Should have at least one interface (loopback)
        assert!(!metrics.interfaces.is_empty());

        for interface in &metrics.interfaces {
            assert!(!interface.name.is_empty());
            assert!(interface.received_mb >= 0.0);
            assert!(interface.transmitted_mb >= 0.0);
        }
    }
}
