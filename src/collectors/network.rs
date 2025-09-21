// ========== src/collectors/network.rs  ========== 

use sysinfo::Networks;
use crate::models::network::{NetworkMetrics, NetworkInterface};

pub fn collect_network_metrics() -> NetworkMetrics {
    // Create Networks instance with data preloaded
    // This automatically discovers all network interfaces and loads their initial statistics

    let networks = Networks::new_with_refreshed_list();

    let mut interfaces = Vec::new();
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

        // Iterate directly over the Networks instance - no .list() needed
        // The Networks struct implements IntoIterator, allowing direct iteration
        // Each iteration yields (interface_name: &String, network_data: &NetworkData)

        for (interface_name, network) in &networks {
            let rx = network.total_received();
            let tx = network.total_transmitted();
            total_rx += rx;
            total_tx += tx;

            interfaces.push(NetworkInterface {
                name: interface_name.to_string(),
                received_mb:        rx as f64 / 1_048_576.0,
                transmitted_mb:     tx as f64 / 1_048_576.0,
            });
        }

    NetworkMetrics {
        total_received_mb:      total_rx as f64 / 1_048_576.0,
        total_transmitted_mb:   total_tx as f64 / 1_048_576.0,
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

        println!("Network interfaces found: {}", metrics.interfaces.len());
        for interface in &metrics.interfaces {
            println!("  {}: RX: {:.2}MB, TX: {:.2}MB",
                     interface.name,
                     interface.received_mb,
                     interface.transmitted_mb);
        }
    }
}
