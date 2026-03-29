use super::detect::DriveInfo;

pub fn format_capacity(bytes: u64) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.1} TB", bytes as f64 / 1_000_000_000_000.0)
    } else if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    }
}

pub fn drive_summary(info: &DriveInfo) -> String {
    format!(
        "{} ({}) - {} [{}]",
        info.name,
        info.path,
        format_capacity(info.capacity_bytes),
        info.file_system
    )
}
