// ========== src/collectors/system.rs ==========
use sysinfo::{ProcessesToUpdate, ProcessRefreshKind, System, UpdateKind};
use crate::models::system::SystemMetrics;
use super::{cpu, memory, disk, network, process};

// Store system state with proper timing control
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

struct SystemState {
    system: System,
    last_cpu_refresh: Instant,
    last_process_refresh: Instant,
    first_run: bool,
}

static SYSTEM_STATE: Lazy<Mutex<SystemState>> = Lazy::new(|| {
    // Use new_all() to properly initialize all components
    let mut sys = System::new_all();

    // Initial CPU refresh sequence
    sys.refresh_cpu_usage();
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_usage();

    // Initial process refresh with PROPER memory data
    // Use ProcessRefreshKind::new() and configure what we need
    let process_refresh_kind = ProcessRefreshKind::everything();

    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        process_refresh_kind,
    );

    Mutex::new(SystemState {
        system: sys,
        last_cpu_refresh: Instant::now(),
        last_process_refresh: Instant::now(),
        first_run: true,
    })
});

pub fn collect_system_metrics() -> SystemMetrics {
    let mut state = SYSTEM_STATE.lock().unwrap();
    let now = Instant::now();

    // Only refresh CPU if enough time has passed
    if now.duration_since(state.last_cpu_refresh) >= Duration::from_millis(200) {
        state.system.refresh_cpu_usage();
        state.last_cpu_refresh = now;
    }

    // Refresh memory data
    state.system.refresh_memory();

    // Create proper refresh kind with memory explicitly enabled
    let process_refresh_kind = ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()  // Explicitly request memory
        .with_disk_usage()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);

    // Always refresh processes to get current memory usage
    state.system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        process_refresh_kind,
    );

    // For CPU calculation, we need two measurements
    if state.first_run || now.duration_since(state.last_process_refresh) < Duration::from_millis(100) {
        std::thread::sleep(Duration::from_millis(100));
        state.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            false,  // Don't remove on second pass
            process_refresh_kind,
        );
        state.first_run = false;
    }

    state.last_process_refresh = now;

    // Debug output for memory specifically
    if cfg!(debug_assertions) {
        println!("\n=== Process Memory Debug ===");
        let mut unique_memory_values = std::collections::HashSet::new();
        let mut sample_count = 0;

        for (pid, proc) in state.system.processes() {
            let mem_bytes = proc.memory();
            let mem_mb = mem_bytes as f64 / 1_048_576.0;

            if mem_mb > 10.0 && sample_count < 5 {  // Show first 5 processes over 10MB
                println!("  {} (PID {}): {:.1} MB (raw: {} bytes)",
                         proc.name().to_string_lossy(),
                         pid.as_u32(),
                         mem_mb,
                         mem_bytes
                );
                sample_count += 1;
            }

            if mem_mb > 1.0 {
                unique_memory_values.insert((mem_mb * 10.0) as i64); // Round to 0.1 MB
            }
        }

        println!("  Unique memory values: {}", unique_memory_values.len());
        println!("========================\n");
    }

    // Create process metrics
    let process_metrics = process::collect_process_metrics(&mut state.system);

    // Collect from all individual collectors
    let sys = &state.system;

    SystemMetrics {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        cpu: cpu::collect_cpu_metrics(sys),
        memory: memory::collect_memory_metrics(sys),
        disk: disk::collect_disk_metrics(),
        network: network::collect_network_metrics(),
        process: process_metrics,
        uptime: System::uptime(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_collection() {
        // Allow system to initialize
        std::thread::sleep(Duration::from_millis(500));

        let metrics = collect_system_metrics();
        assert!(!metrics.timestamp.is_empty());
        assert!(!metrics.hostname.is_empty());

        println!("\n=== Test Results ===");
        println!("Process count: {}", metrics.process.total_count);

        println!("\nTop Memory Processes:");
        for p in &metrics.process.top_memory {
            println!("  {} (PID {}): {:.1} MB", p.name, p.pid, p.memory_mb);
        }

        // Check that we have different memory values
        let memory_values: std::collections::HashSet<i64> = metrics.process.top_memory
            .iter()
            .map(|p| (p.memory_mb * 10.0) as i64)  // Round to 0.1 MB
            .collect();

        println!("\nUnique memory values in top 5: {}", memory_values.len());
        assert!(memory_values.len() > 1, "All processes shouldn't have the same memory!");
    }
}