//! CPU metrics collector
//!
//! Collects CPU usage statistics including global usage, per-core usage,
//! frequency, and CPU brand information.

use crate::models::cpu::CpuMetrics;
use sysinfo::System;

/// Collect current CPU metrics from the system
///
/// Returns metrics including overall CPU usage, per-core usage,
/// frequency, and CPU model information.
pub fn collect_cpu_metrics(sys: &System) -> CpuMetrics {
    let global_usage = sys.global_cpu_usage();
    let per_core_usage: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    let (frequency, brand) = if let Some(cpu) = sys.cpus().first() {
        (cpu.frequency(), cpu.brand().to_string())
    } else {
        (0, "Unknown".to_string())
    };

    CpuMetrics {
        usage_percent: global_usage,
        core_count: sys.cpus().len(),
        frequency,
        brand,
        per_core_usage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cpu_collection() {
        let mut sys = System::new_all();

        sys.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(200));
        sys.refresh_cpu_frequency();

        let metrics = collect_cpu_metrics(&sys);

        assert!(metrics.core_count > 0);
        assert!(metrics.usage_percent >= 0.0);
        assert!(metrics.usage_percent <= 100.0);
        assert!(!metrics.brand.is_empty());
        assert_eq!(metrics.per_core_usage.len(), metrics.core_count);
    }
}