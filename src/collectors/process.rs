// src/collectors/process.rs

use sysinfo::System;
use crate::models::process::{ProcessMetrics, ProcessInfo};

pub fn collect_process_metrics(sys: &mut System) -> ProcessMetrics {
    // Processes should already be refreshed, but ensure we have CPU data
    let mut all_processes: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        let cpu_usage = process.cpu_usage();
        let memory = process.memory();

        // Only include processes with some resource usage
        if cpu_usage > 0.0 || memory > 1024 * 1024 { // > 1MB
            all_processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage,
                memory_mb: memory as f64 / 1_048_576.0,
            });
        }
    }

    // Clone for separate sorting
    let mut cpu_sorted = all_processes.clone();
    let mut mem_sorted = all_processes.clone();

    // Sort by CPU usage
    cpu_sorted.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
    let top_cpu: Vec<ProcessInfo> = cpu_sorted.into_iter().take(5).collect();

    // Sort by memory usage
    mem_sorted.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap());
    let top_memory: Vec<ProcessInfo> = mem_sorted.into_iter().take(5).collect();

    // Debug output
    if top_cpu.is_empty() && top_memory.is_empty() {
        println!("Warning: No processes found with CPU/memory usage");
    }

    ProcessMetrics {
        total_count: sys.processes().len(),
        top_cpu,
        top_memory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::{ProcessesToUpdate, ProcessRefreshKind};

    #[test]
    fn test_process_collection() {
        let mut sys = System::new_all();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything()
        );
        std::thread::sleep(std::time::Duration::from_millis(500));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything()
        );

        let metrics = collect_process_metrics(&mut sys);

        println!("Total processes: {}", metrics.total_count);
        println!("Top CPU processes: {}", metrics.top_cpu.len());
        println!("Top Memory processes: {}", metrics.top_memory.len());

        assert!(metrics.total_count > 0);
    }
}