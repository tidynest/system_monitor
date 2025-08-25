// src/models/memory.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemoryMetrics {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
    pub swap_total_gb: f64,
    pub swap_used_gb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_metrics_serialization() {
        let metrics = MemoryMetrics {
            total_gb: 16.0,
            used_gb: 8.0,
            available_gb: 8.0,
            usage_percent: 50.0,
            swap_total_gb: 8.0,
            swap_used_gb: 8.0,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: MemoryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, deserialized);
    }
}
