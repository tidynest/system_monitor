// ========== src/collectors/process.rs ========== 

use sysinfo::{System, UpdateKind};
use crate::models::process::{ProcessMetrics, ProcessInfo};

pub fn collect_process_metrics(sys: &mut System) -> ProcessMetrics {
    // IMPORTANT: Force a fresh process refresh with memory data
    // Some systems need an explicit refresh to get accurate per-process memory
    use sysinfo::{ProcessesToUpdate, ProcessRefreshKind};

    let refresh_kind = ProcessRefreshKind::nothing()
        .with_memory()
        .with_cpu()
        .with_disk_usage()
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_cmd(UpdateKind::OnlyIfNotSet);

    // Refresh processes to get current memory values
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        refresh_kind
    );

    let mut all_processes: Vec<ProcessInfo> = Vec::new();

    for (pid, process) in sys.processes() {
        let cpu_usage = process.cpu_usage();
        let memory = process.memory(); // This is in bytes
        let name = process.name().to_string_lossy().to_string();

        // Skip very small processes (less than 10MB)
        let memory_mb = memory as f64 / 1_048_576.0;
        if memory_mb < 10.0 {
            continue;
        }

        // Debug: Print first 5 processes to check memory values
        if cfg!(debug_assertions) && all_processes.len() < 5 {
            println!("Process: {} (PID {}): {:.1} MB (raw: {} bytes)",
                     name, pid.as_u32(), memory_mb, memory);
        }

        all_processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name,
            cpu_usage,
            memory_mb,
        });
    }

    // Check if all processes have the same memory value (indicates a bug)
    if cfg!(debug_assertions) && all_processes.len() > 2 {
        let first_mem = all_processes[0].memory_mb;
        let all_same = all_processes.iter()
            .all(|p| (p.memory_mb - first_mem).abs() < 0.1);

        if all_same {
            println!("WARNING: All {} processes have the same memory value ({:.1} MB)!",
                     all_processes.len(), first_mem);
            println!("This indicates memory data is not being properly collected.");

            // Try to get at least some variation by using the raw process list
            println!("Attempting fallback memory collection...");
            for (pid, process) in sys.processes().iter().take(10) {
                let mem_bytes = process.memory();
                println!("  {} ({}): {} bytes = {:.1} MB",
                         process.name().to_string_lossy(),
                         pid.as_u32(),
                         mem_bytes,
                         mem_bytes as f64 / 1_048_576.0);
            }
        }
    }

    // Sort by CPU usage (highest first)
    let mut cpu_sorted = all_processes.clone();
    cpu_sorted.sort_by(|a, b| {
        b.cpu_usage.partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_cpu: Vec<ProcessInfo> = cpu_sorted
        .into_iter()
        .take(5)
        .collect();

    // Sort by memory usage (highest first)
    let mut mem_sorted = all_processes.clone();
    mem_sorted.sort_by(|a, b| {
        b.memory_mb.partial_cmp(&a.memory_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top 5 UNIQUE memory values to show variety
    let mut seen_memory_values = std::collections::HashSet::new();
    let mut top_memory: Vec<ProcessInfo> = Vec::new();

    for process in mem_sorted.into_iter() {
        // Round to nearest MB to group similar values
        let memory_rounded = process.memory_mb.round() as i64;

        // If we haven't seen this memory value (or very close), add it
        if seen_memory_values.insert(memory_rounded) {
            top_memory.push(process);
            if top_memory.len() >= 5 {
                break;
            }
        }
    }

    // If we still don't have 5 unique ones, just take what we have
    if top_memory.len() < 5 {
        println!("Warning: Only found {} unique memory values in top processes", top_memory.len());
    }

    // Debug: Print top memory processes
    if cfg!(debug_assertions) {
        println!("\nTop 5 Memory Processes:");
        for p in &top_memory {
            println!("  {} (PID {}): {:.1} MB", p.name, p.pid, p.memory_mb);
        }

        // Check for unique values
        let unique_memories: std::collections::HashSet<i64> = top_memory
            .iter()
            .map(|p| (p.memory_mb * 10.0) as i64)
            .collect();

        if unique_memories.len() == 1 && top_memory.len() > 1 {
            println!("ERROR: All top memory processes have the same value!");
        }
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

        // Refresh with memory explicitly requested
        // FIXED: Changed from ProcessRefreshKind::new() to nothing()
        let refresh_kind = ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu();

        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            refresh_kind,
        );

        // Wait and refresh again for CPU calculation
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            false,
            refresh_kind,
        );

        let metrics = collect_process_metrics(&mut sys);

        println!("\n=== Process Collection Test ===");
        println!("Total processes: {}", metrics.total_count);

        println!("\nTop Memory Processes:");
        for p in &metrics.top_memory {
            println!("  {} (PID {}): {:.1} MB", p.name, p.pid, p.memory_mb);
        }

        // Verify we have different memory values
        let unique_memories: std::collections::HashSet<i64> = metrics.top_memory
            .iter()
            .map(|p| (p.memory_mb * 10.0) as i64)
            .collect();

        println!("\nUnique memory values: {}", unique_memories.len());

        assert!(metrics.total_count > 0);
        assert!(!metrics.top_memory.is_empty(), "Should have memory processes");

        if metrics.top_memory.len() > 1 {
            assert!(unique_memories.len() > 1,
                    "Processes should have different memory values, not all the same!");
        }
    }
}