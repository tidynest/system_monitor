// src/collectors/cpu.rs

use sysinfo::System;
use crate::models::cpu::CpuMetrics;

pub fn collect_cpu_metrics(sys: &System) -> CpuMetrics {
    // Get individual core usage - these are already averaged properly
    let per_core_usage: Vec<f32> = sys.cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect();

    // Calculate average CPU usage across all cores manually
    // This gives us more control over the calculation
    let global_usage = if !per_core_usage.is_empty() {
        per_core_usage.iter().sum::<f32>() / per_core_usage.len() as f32
    } else {
        0.0
    };

    // Get core counts for information
    let physical_cores = System::physical_core_count().unwrap_or(sys.cpus().len());
    let logical_cores = sys.cpus().len();

    // Debug output with more detail
    println!("Debug CPU - Physical cores: {}, Logical cores: {}", physical_cores, logical_cores);
    println!("Debug CPU - Calculated average: {:.1}%", global_usage);
    println!("Debug CPU - Per core: {:?}", per_core_usage.iter().take(4).collect::<Vec<_>>());

    CpuMetrics {
        usage_percent: global_usage,
        core_count: logical_cores,
        frequency: sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0),
        brand: System::cpu_arch(),
        per_core_usage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::CpuRefreshKind;

    #[test]
    fn test_cpu_collection() {
        let mut sys = System::new();

        // Initial CPU refresh with specific refresh kind
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        // Wait for measurement interval
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Second refresh for accurate measurement
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        let metrics = collect_cpu_metrics(&sys);

        assert!(metrics.core_count > 0);
        assert!(metrics.usage_percent >= 0.0);
        assert!(metrics.usage_percent <= 100.0);

        println!("CPU cores: {}", metrics.core_count);
        println!("CPU usage: {:.1}%", metrics.usage_percent);
        println!("CPU brand: {}", metrics.brand);
    }
}