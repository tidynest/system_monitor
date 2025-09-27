//! Process metrics collector
//!
//! Collects information about running processes including CPU and memory usage.
//! Identifies top consumers of system resources.

use crate::models::process::{ProcessInfo, ProcessMetrics};
use std::collections::HashSet;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Collect metrics for all system processes
///
/// Returns the total process count and lists of top CPU and memory consumers.
/// Filters out processes using less than 10MB of memory to focus on significant processes.
pub fn collect_process_metrics(sys: &mut System) -> ProcessMetrics {
    // Configure refresh to get all needed process data
    let refresh_kind = ProcessRefreshKind::nothing()
        .with_memory()
        .with_cpu()
        .with_disk_usage()
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_cmd(UpdateKind::OnlyIfNotSet);

    // Refresh processes to get current values
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);

    let mut all_processes: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        let cpu_usage = process.cpu_usage();
        let memory = process.memory(); // bytes
        let name = process.name().to_string_lossy().to_string();

        // Filter out small processes (less than 10MB)
        let memory_mb = memory as f64 / 1_048_576.0;
        if memory_mb < 10.0 {
            continue;
        }

        all_processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name,
            cpu_usage,
            memory_mb,
        });
    }

    // Get top 5 CPU consumers
    let top_cpu = get_top_by_cpu(&all_processes, 5);

    // Get top 5 unique memory consumers
    let top_memory = get_top_unique_by_memory(&all_processes, 5);

    ProcessMetrics {
        total_count: sys.processes().len(),
        top_cpu,
        top_memory,
    }
}

/// Get top N processes sorted by CPU usage
fn get_top_by_cpu(processes: &[ProcessInfo], limit: usize) -> Vec<ProcessInfo> {
    let mut cpu_sorted = processes.to_vec();
    cpu_sorted.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    cpu_sorted.into_iter().take(limit).collect()
}

/// Get top N processes by memory with unique values
///
/// Ensures variety in displayed memory values by filtering similar values.
/// This prevents showing multiple processes with identical memory usage.
fn get_top_unique_by_memory(processes: &[ProcessInfo], limit: usize) -> Vec<ProcessInfo> {
    let mut mem_sorted = processes.to_vec();
    mem_sorted.sort_by(|a, b| {
        b.memory_mb
            .partial_cmp(&a.memory_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter for unique memory values (rounded to nearest MB)
    let mut seen_memory_values = HashSet::new();
    let mut top_memory = Vec::new();

    for process in mem_sorted {
        let memory_rounded = process.memory_mb.round() as i64;
        if seen_memory_values.insert(memory_rounded) {
            top_memory.push(process);
            if top_memory.len() >= limit {
                break;
            }
        }
    }

    top_memory
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};

    #[test]
    fn test_process_collection() {
        let mut sys = System::new_all();

        let refresh_kind = ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu();

        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);

        // Wait for CPU calculation
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_processes_specifics(ProcessesToUpdate::All, false, refresh_kind);

        let metrics = collect_process_metrics(&mut sys);

        assert!(metrics.total_count > 0, "Should have at least one process");
        assert!(!metrics.top_memory.is_empty(), "Should have memory processes");

        // Verify unique memory values
        if metrics.top_memory.len() > 1 {
            let unique_memories: HashSet<i64> = metrics
                .top_memory
                .iter()
                .map(|p| (p.memory_mb * 10.0) as i64)
                .collect();

            assert!(
                unique_memories.len() > 1,
                "Processes should have different memory values"
            );
        }
    }
}