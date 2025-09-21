// ==========  src/collectors/disk.rs ========== 

use sysinfo::Disks;
use crate::models::disk::DiskMetrics;

pub fn collect_disk_metrics() -> Vec<DiskMetrics> {
    let disks = Disks::new_with_refreshed_list();

    disks.list().iter().map(|disk| {
        let total = disk.total_space() as f64;
        let available = disk.available_space() as f64;
        let used = total - available;
        let usage_percent = if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        };

        DiskMetrics {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_gb: disk.total_space() as f64 / 1_073_741_824.0,
            available_gb: disk.available_space() as f64 / 1_073_741_824.0,
            usage_percent,
            file_system: disk.file_system().to_string_lossy().to_string(),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_collection() {
        let metrics = collect_disk_metrics();

        // Should have at least one disk
        assert!(!metrics.is_empty());

        // Verify percentages are valid
        for disk in &metrics {
            assert!(disk.usage_percent >= 0.0);
            assert!(disk.usage_percent <= 100.0);
        }
    }
}
