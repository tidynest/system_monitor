// ========== src/models/cpu.rs ========== 

use serde::{ Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub core_count: usize,
    pub frequency: u64,
    pub brand: String,
    pub per_core_usage: Vec<f32>,
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_cpu_metrics_collection() {
        // Test implementation
    }
}
