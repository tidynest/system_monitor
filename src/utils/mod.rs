// src/utils/mod.rs

#![allow(dead_code)]
/// Format bytes into human-readable format
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

/// Format uptime from seconds
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = seconds % 86400 / 60;

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
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB"); // 1024^2
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB"); // 1024^3
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(3700), "1h 1m");
        assert_eq!(format_uptime(3600), "1d 1h 0m");
    }
}
