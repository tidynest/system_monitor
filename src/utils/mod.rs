//! Utility functions for the system monitor
//!
//! Provides formatting helpers for displaying system metrics in human-readable formats.

/// Format bytes into human-readable format with appropriate units
///
/// # Examples
/// ```
/// use system_monitor::utils::format_bytes;
/// assert_eq!(format_bytes(1024), "1.00 KB");
/// assert_eq!(format_bytes(1_048_576), "1.00 MB");
/// ```
#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Format system uptime from seconds into a readable duration
///
/// Formats as:
/// - Less than 1 hour: "Xm"
/// - Less than 1 day: "Xh Ym"
/// - 1 day or more: "Xd Yh Zm"
///
/// # Examples
/// ```
/// use system_monitor::utils::format_uptime;
/// assert_eq!(format_uptime(60), "1m");
/// assert_eq!(format_uptime(3700), "1h 1m");
/// assert_eq!(format_uptime(90000), "1d 1h 0m");
/// ```
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
        assert_eq!(format_bytes(1_099_511_627_776), "1.00 TB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "0m");
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(90), "1m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3660), "1h 1m");
        assert_eq!(format_uptime(7200), "2h 0m");
        assert_eq!(format_uptime(86400), "1d 0h 0m");
        assert_eq!(format_uptime(90000), "1d 1h 0m");
        assert_eq!(format_uptime(176400), "2d 1h 0m");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_uptime(0), "0m");
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(86399), "23h 59m");
    }
}
