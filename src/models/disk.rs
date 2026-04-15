//! Disk metrics data model
//!
//! Structures for representing disk/filesystem usage information.

use serde::{Deserialize, Serialize};

/// Disk usage metrics for a single filesystem
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskMetrics {
    /// Disk name (e.g., "sda1")
    pub name: String,
    /// Mount point path (e.g., "/", "/home")
    pub mount_point: String,
    /// Total disk space in GB
    pub total_gb: f64,
    /// Available disk space in GB
    pub available_gb: f64,
    /// Usage percentage (0.0 - 100.0)
    pub usage_percent: f64,
    /// Filesystem type (e.g., "ext4", "ntfs")
    pub file_system: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_metrics_creation() {
        let disk = DiskMetrics {
            name: "sda1".to_string(),
            mount_point: "/".to_string(),
            total_gb: 100.0,
            available_gb: 40.0,
            usage_percent: 60.0,
            file_system: "ext4".to_string(),
        };

        assert_eq!(disk.name, "sda1");
        assert_eq!(disk.mount_point, "/");
        assert_eq!(disk.total_gb, 100.0);
        assert_eq!(disk.available_gb, 40.0);
        assert_eq!(disk.usage_percent, 60.0);
        assert_eq!(disk.file_system, "ext4");
    }
}
