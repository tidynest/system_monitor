//! System-wide metrics collector
//!
//! Orchestrates collection of all system metrics and manages the global
//! System instance for efficient resource usage.

use crate::models::system::SystemMetrics;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::{cpu, disk, memory, network, process};

/// Global system state for efficient metric collection
struct SystemState {
    system: System,
    last_cpu_refresh: Instant,
    last_process_refresh: Instant,
    first_run: bool,
}

/// Singleton system state to avoid recreating System instances
static SYSTEM_STATE: Lazy<Mutex<SystemState>> = Lazy::new(|| {
    let mut sys = System::new_all();

    //  CPU metrics with proper interval
    sys.refresh_cpu_usage();
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_usage();

    //  process data
    let process_refresh_kind = ProcessRefreshKind::everything();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, process_refresh_kind);

    Mutex::new(SystemState {
        system: sys,
        last_cpu_refresh: Instant::now(),
        last_process_refresh: Instant::now(),
        first_run: true,
    })
});

/// Collect all system metrics
///
/// Efficiently gathers CPU, memory, disk, network, and process metrics
/// using a shared System instance to minimize overhead.
pub fn collect_system_metrics() -> SystemMetrics {
    let mut state = SYSTEM_STATE.lock().unwrap();
    let now = Instant::now();

    // Refresh CPU data if enough time has passed
    if now.duration_since(state.last_cpu_refresh) >= Duration::from_millis(200) {
        state.system.refresh_cpu_usage();
        state.last_cpu_refresh = now;
    }

    // Always refresh memory for current data
    state.system.refresh_memory();

    // Configure process refresh with necessary data
    let process_refresh_kind = ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_disk_usage()
        .with_cmd(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet);

    // Refresh process data
    state.system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        process_refresh_kind,
    );

    // Ensure proper CPU calculation interval on first run
    if state.first_run
        || now.duration_since(state.last_process_refresh) < Duration::from_millis(100)
    {
        std::thread::sleep(Duration::from_millis(100));
        state.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            false,
            process_refresh_kind,
        );
        state.first_run = false;
    }

    state.last_process_refresh = now;

    // Collect metrics from all subsystems
    let process_metrics = process::collect_process_metrics(&mut state.system);

    SystemMetrics {
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        cpu: cpu::collect_cpu_metrics(&state.system),
        memory: memory::collect_memory_metrics(&state.system),
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
    use std::collections::HashSet;

    #[test]
    fn test_system_metrics_collection() {
        // Allow system to 
        std::thread::sleep(Duration::from_millis(500));

        let metrics = collect_system_metrics();

        assert!(!metrics.timestamp.is_empty());
        assert!(!metrics.hostname.is_empty());
        assert!(metrics.process.total_count > 0);

        // Verify unique memory values in processes
        if metrics.process.top_memory.len() > 1 {
            let memory_values: HashSet<i64> = metrics
                .process
                .top_memory
                .iter()
                .map(|p| (p.memory_mb * 10.0) as i64)
                .collect();

            assert!(
                memory_values.len() > 1,
                "Processes should have varied memory usage"
            );
        }
    }
}