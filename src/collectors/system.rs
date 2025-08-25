// src/collectors/system.rs

use sysinfo::{System, ProcessesToUpdate, ProcessRefreshKind, CpuRefreshKind};
use crate::models::system::SystemMetrics;
use super::{cpu, memory, disk, network, process};

// Store system state with proper timing control
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

struct SystemState {
    system: System,
    last_cpu_refresh: Option<Instant>,
    first_run: bool,
}

static SYSTEM_STATE: Lazy<Mutex<SystemState>> = Lazy::new(|| {
    let sys = System::new();

    // Initial setup - don't do CPU refresh here
    // We'll handle it properly in the collect function

    Mutex::new(SystemState {
        system: sys,
        last_cpu_refresh: None,
        first_run: true,
    })
});

pub fn collect_system_metrics() -> SystemMetrics {
    let mut state = SYSTEM_STATE.lock().unwrap();

    // Handle first run specially
    if state.first_run {
        // On first run, do two refreshes with a delay to get accurate initial reading
        state.system.refresh_cpu_specifics(CpuRefreshKind::everything());
        std::thread::sleep(Duration::from_millis(200));
        state.system.refresh_cpu_specifics(CpuRefreshKind::everything());
        state.last_cpu_refresh = Some(Instant::now());
        state.first_run = false;
    } else {
        // For subsequent runs, check if enough time has passed
        let now = Instant::now();
        let should_refresh_cpu = state.last_cpu_refresh
            .map(|last| now.duration_since(last) >= Duration::from_millis(200))
            .unwrap_or(true);

        if should_refresh_cpu {
            // Single refresh is enough after the initial setup
            state.system.refresh_cpu_specifics(CpuRefreshKind::everything());
            state.last_cpu_refresh = Some(now);
        }
    }

    // Refresh memory data
    state.system.refresh_memory();

    // Refresh process data with proper specifications
    state.system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true, // Remove dead processes
        ProcessRefreshKind::everything(),
    );

    // Debug output
    let process_count = state.system.processes().len();
    println!("Debug: Found {} processes", process_count);

    // Create process metrics (needs mutable reference)
    let process_metrics = process::collect_process_metrics(&mut state.system);

    // Collect all metrics using immutable reference
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

        // CPU usage should be reasonable
        assert!(metrics.cpu.usage_percent >= 0.0);
        assert!(metrics.cpu.usage_percent <= 100.0);

        println!("CPU Usage: {:.1}%", metrics.cpu.usage_percent);
        println!("Process count: {}", metrics.process.total_count);
        println!("Top CPU processes: {}", metrics.process.top_cpu.len());

        // Test a second collection to ensure consistency
        std::thread::sleep(Duration::from_millis(300));
        let metrics2 = collect_system_metrics();

        println!("Second CPU Usage: {:.1}%", metrics2.cpu.usage_percent);

        // Both measurements should be in reasonable range
        assert!(metrics2.cpu.usage_percent >= 0.0);
        assert!(metrics2.cpu.usage_percent <= 100.0);
    }
}