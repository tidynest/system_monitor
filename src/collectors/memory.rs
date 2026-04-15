//! Memory metrics collector
//!
//! Collects system memory and swap usage statistics.

use crate::models::memory::MemoryMetrics;
use sysinfo::System;

/// Collect current memory metrics from the system
///
/// Returns metrics including RAM and swap usage in GB and as percentages.
pub fn collect_memory_metrics(sys: &System) -> MemoryMetrics {
    let available_memory = sys.available_memory() as f64;
    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let total_swap = sys.total_swap() as f64;
    let used_swap = sys.used_swap() as f64;

    MemoryMetrics {
        total_gb: total_memory / 1_073_741_824.0,
        used_gb: used_memory / 1_073_741_824.0,
        available_gb: available_memory / 1_073_741_824.0,
        usage_percent: (used_memory / total_memory) * 100.0,
        swap_total_gb: total_swap / 1_073_741_824.0,
        swap_used_gb: used_swap / 1_073_741_824.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_collection() {
        let sys = System::new_all();
        let metrics = collect_memory_metrics(&sys);

        assert!(metrics.total_gb > 0.0);
        assert!(metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0);
        assert!(metrics.available_gb > 0.0);
        assert!(metrics.used_gb + metrics.available_gb <= metrics.total_gb * 1.1); // Allow 10% tolerance
    }
}
