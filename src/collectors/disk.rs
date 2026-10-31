//! Disk metrics collector
//!
//! Collects disk usage statistics for all mounted filesystems.

use crate::models::disk::DiskMetrics;
use sysinfo::Disks;

/// Collect metrics for all mounted disks
///
/// Returns a vector containing usage statistics for each mounted filesystem
/// including total space, available space, and usage percentage. Mounts that
/// report no capacity (FUSE and portal mounts such as `/run/user/1000/doc`)
/// are skipped, since they hold no data of their own.
pub fn collect_disk_metrics() -> Vec<DiskMetrics> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .list()
        .iter()
        .filter(|disk| disk.total_space() > 0)
        .map(|disk| {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;
            let usage_percent = (used / total) * 100.0;

            DiskMetrics {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_gb: total / 1_073_741_824.0,
                available_gb: available / 1_073_741_824.0,
                usage_percent,
                file_system: disk.file_system().to_string_lossy().to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_collection() {
        let metrics = collect_disk_metrics();

        assert!(!metrics.is_empty(), "Should have at least one disk");

        for disk in &metrics {
            assert!(disk.usage_percent >= 0.0);
            assert!(disk.usage_percent <= 100.0);
            assert!(disk.total_gb > 0.0);
            assert!(!disk.mount_point.is_empty());
        }
    }
}
