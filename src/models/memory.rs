//! Memory metrics data model
//!
//! Structures for representing system memory and swap usage.

use serde::{Deserialize, Serialize};

/// Memory and swap usage metrics
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemoryMetrics {
    /// Total system memory in GB
    pub total_gb: f64,
    /// Used memory in GB
    pub used_gb: f64,
    /// Available memory in GB
    pub available_gb: f64,
    /// Memory usage percentage
    pub usage_percent: f64,
    /// Total swap space in GB
    pub swap_total_gb: f64,
    /// Used swap space in GB
    pub swap_used_gb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics_serialization() {
        let metrics = MemoryMetrics {
            total_gb: 16.0,
            used_gb: 8.0,
            available_gb: 8.0,
            usage_percent: 50.0,
            swap_total_gb: 8.0,
            swap_used_gb: 4.0,
        };

        // Test serialization
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"total_gb\":16.0"));
        assert!(json.contains("\"usage_percent\":50.0"));

        // Test round-trip
        let deserialized: MemoryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, deserialized);
    }

    #[test]
    fn test_memory_calculations() {
        let metrics = MemoryMetrics {
            total_gb: 32.0,
            used_gb: 24.0,
            available_gb: 8.0,
            usage_percent: 75.0,
            swap_total_gb: 16.0,
            swap_used_gb: 2.0,
        };

        assert_eq!(metrics.total_gb - metrics.available_gb, metrics.used_gb);
        assert_eq!(
            metrics.usage_percent,
            metrics.used_gb / metrics.total_gb * 100.0
        );
    }
}
