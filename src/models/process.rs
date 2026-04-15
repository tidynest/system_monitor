//! Process-related data models
//!
//! Structures for representing process information and metrics.

use serde::{Deserialize, Serialize};

/// Aggregated process metrics
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMetrics {
    /// Total number of running processes
    pub total_count: usize,
    /// Top processes by CPU usage
    pub top_cpu: Vec<ProcessInfo>,
    /// Top processes by memory usage
    pub top_memory: Vec<ProcessInfo>,
}

/// Information about a single process
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in megabytes
    pub memory_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_creation() {
        let process = ProcessInfo {
            pid: 1234,
            name: "test_process".to_string(),
            cpu_usage: 25.5,
            memory_mb: 512.0,
        };

        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test_process");
        assert_eq!(process.cpu_usage, 25.5);
        assert_eq!(process.memory_mb, 512.0);
    }

    #[test]
    fn test_process_metrics_creation() {
        let metrics = ProcessMetrics {
            total_count: 42,
            top_cpu: vec![],
            top_memory: vec![],
        };

        assert_eq!(metrics.total_count, 42);
        assert!(metrics.top_cpu.is_empty());
        assert!(metrics.top_memory.is_empty());
    }
}
