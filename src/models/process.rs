// ========== src/models/process.rs ==========

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMetrics {
    pub total_count: usize,
    pub top_cpu: Vec<ProcessInfo>,
    pub top_memory: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
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
        assert_eq!(process.cpu_usage, 25.5);
    }
}
