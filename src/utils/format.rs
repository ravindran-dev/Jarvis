/// Format bytes into human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log10() / 3.0).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    
    let value = bytes_f / 1024_f64.powi(unit_index as i32);
    
    if value < 10.0 {
        format!("{:.2} {}", value, UNITS[unit_index])
    } else if value < 100.0 {
        format!("{:.1} {}", value, UNITS[unit_index])
    } else {
        format!("{:.0} {}", value, UNITS[unit_index])
    }
}

/// Format duration in seconds to human-readable format
#[allow(dead_code)]
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format percentage with color coding
#[allow(dead_code)]
pub fn format_percentage(value: f64, total: f64) -> String {
    if total == 0.0 {
        return "0%".to_string();
    }
    let percentage = (value / total) * 100.0;
    format!("{:.1}%", percentage)
}

/// Truncate string to specified length with ellipsis
#[allow(dead_code)]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Convert timestamp to human-readable format
#[allow(dead_code)]
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{Local, TimeZone};
    
    if let Some(dt) = Local.timestamp_opt(timestamp, 0).single() {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid timestamp".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(90000), "1d 1h 0m");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
    }
}
