//! CPU metrics data model
//!
//! Structures for representing CPU usage and information.

use serde::{Deserialize, Serialize};

/// CPU metrics and information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuMetrics {
    /// Overall CPU usage percentage
    pub usage_percent: f32,
    /// Number of CPU cores
    pub core_count: usize,
    /// CPU frequency in MHz
    pub frequency: u64,
    /// CPU brand/model string
    pub brand: String,
    /// Per-core usage percentages
    pub per_core_usage: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_metrics_creation() {
        let metrics = CpuMetrics {
            usage_percent: 45.5,
            core_count: 8,
            frequency: 3600,
            brand: "Intel Core i7".to_string(),
            per_core_usage: vec![40.0, 45.0, 50.0, 42.0, 48.0, 43.0, 46.0, 44.0],
        };

        assert_eq!(metrics.core_count, 8);
        assert_eq!(metrics.frequency, 3600);
        assert_eq!(metrics.brand, "Intel Core i7");
        assert_eq!(metrics.per_core_usage.len(), 8);
    }
}