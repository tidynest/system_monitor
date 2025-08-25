// src/models/system.rs

use serde::{Deserialize, Serialize};
use super::{CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, ProcessMetrics};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub disk: Vec<DiskMetrics>,
    pub hostname: String,
    pub memory: MemoryMetrics,
    pub network: NetworkMetrics,
    pub process: ProcessMetrics,
    pub timestamp: String,
    pub uptime: u64,
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_cpu_metrics_collection() {
        // Test would go here - this is a placeholder
        // In real tests, you'd create a full SystemMetrics instance
   }
}
