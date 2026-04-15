//! System-wide metrics model
//!
//! Root structure that aggregates all system metrics into a single snapshot.

use serde::{Deserialize, Serialize};

use super::{CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics};

/// Complete system metrics snapshot
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemMetrics {
    /// CPU usage and information
    pub cpu: CpuMetrics,
    /// Disk usage for all mounted filesystems
    pub disk: Vec<DiskMetrics>,
    /// System hostname
    pub hostname: String,
    /// Memory and swap usage
    pub memory: MemoryMetrics,
    /// Network interface statistics
    pub network: NetworkMetrics,
    /// Process information and top consumers
    pub process: ProcessMetrics,
    /// Timestamp of metric collection
    pub timestamp: String,
    /// System uptime in seconds
    pub uptime: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_serialization() {
        let metrics = SystemMetrics {
            cpu: CpuMetrics {
                usage_percent: 25.5,
                core_count: 8,
                frequency: 2400, // Changed from 2400.0 to 2400 (u64)
                brand: "Test CPU".to_string(),
                per_core_usage: vec![25.0, 30.0, 20.0, 35.0],
            },
            disk: vec![],
            hostname: "test-host".to_string(),
            memory: MemoryMetrics {
                total_gb: 16.0,
                used_gb: 8.0,
                available_gb: 8.0,
                usage_percent: 50.0,
                swap_total_gb: 4.0,
                swap_used_gb: 0.0,
            },
            network: NetworkMetrics {
                total_received_mb: 1024.0,
                total_transmitted_mb: 512.0,
                interfaces: vec![],
            },
            process: ProcessMetrics {
                total_count: 150,
                top_cpu: vec![],
                top_memory: vec![],
            },
            timestamp: "2024-01-01 12:00:00".to_string(),
            uptime: 3600,
        };

        // Test that the struct can be serialized to JSON
        let json = serde_json::to_string(&metrics).expect("Failed to serialize");
        assert!(json.contains("test-host"));
        assert!(json.contains("\"uptime\":3600"));

        // Test deserialization
        let deserialized: SystemMetrics =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.hostname, "test-host");
        assert_eq!(deserialized.uptime, 3600);
    }
}
