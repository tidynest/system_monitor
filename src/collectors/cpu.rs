// ========== src/collectors/cpu.rs ==========
use sysinfo::System;
use crate::models::cpu::CpuMetrics;

pub fn collect_cpu_metrics(sys: &System) -> CpuMetrics {
    // Use the built-in global CPU usage (properly averaged)
    let global_usage = sys.global_cpu_usage();

    // Get per-core usage
    let per_core_usage: Vec<f32> = sys.cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect();

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
        // Must use new_all() for tests too
        let mut sys = System::new_all();

        // Initial CPU refresh
        sys.refresh_cpu_usage();
        thread::sleep(Duration::from_millis(200));
        sys.refresh_cpu_frequency();

        let metrics = collect_cpu_metrics(&sys);

        assert!(metrics.core_count > 0);
        assert!(metrics.usage_percent >= 0.0);
        assert!(metrics.usage_percent <= 100.0);

        println!("CPU cores: {}", metrics.core_count);
        println!("CPU usage: {:.1}%", metrics.usage_percent);
        println!("CPU brand: {}", metrics.brand);
    }
}