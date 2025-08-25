// src/models/disk.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
    pub file_system: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_metrics_calculation() {
        let disk = DiskMetrics {
            name: "sda1".to_string(),
            mount_point: "/".to_string(),
            total_gb: 100.0,
            available_gb: 40.0,
            usage_percent: 60.0,
            file_system: "ext4".to_string(),
        };

        assert_eq!(disk.usage_percent, 60.0);
    }
}

