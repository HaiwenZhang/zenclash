/// Format bytes to human-readable string
pub fn format_traffic(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    const TB: u64 = 1024 * 1024 * 1024 * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format bytes per second to speed string
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_traffic(bytes_per_sec))
}

/// Calculate percentage
pub fn calc_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    ((used as f64 / total as f64) * 100.0).min(100.0) as f32
}

/// Get delay color based on latency
pub fn delay_color(delay: i32) -> &'static str {
    if delay < 0 {
        "default"
    } else if delay == 0 {
        "danger"
    } else if delay < 500 {
        "success"
    } else {
        "warning"
    }
}

/// Format delay for display
pub fn format_delay(delay: i32) -> String {
    if delay < 0 {
        "Test".to_string()
    } else if delay == 0 {
        "Timeout".to_string()
    } else {
        format!("{}ms", delay)
    }
}

/// Format duration to human-readable string
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format timestamp to relative time
pub fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_traffic() {
        assert_eq!(format_traffic(500), "500 B");
        assert_eq!(format_traffic(1024), "1.00 KB");
        assert_eq!(format_traffic(1024 * 1024), "1.00 MB");
        assert_eq!(format_traffic(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(1024), "1.00 KB/s");
        assert_eq!(format_speed(1024 * 1024), "1.00 MB/s");
    }

    #[test]
    fn test_calc_percent() {
        assert_eq!(calc_percent(50, 100), 50.0);
        assert_eq!(calc_percent(0, 100), 0.0);
        assert_eq!(calc_percent(100, 0), 0.0);
    }

    #[test]
    fn test_delay_color() {
        assert_eq!(delay_color(-1), "default");
        assert_eq!(delay_color(0), "danger");
        assert_eq!(delay_color(100), "success");
        assert_eq!(delay_color(600), "warning");
    }
}
