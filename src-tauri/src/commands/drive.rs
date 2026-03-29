use crate::disk::detect;

#[tauri::command]
pub async fn list_drives() -> Result<Vec<detect::DriveInfo>, String> {
    tokio::task::spawn_blocking(|| detect::list_usb_drives())
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
pub async fn get_drive_info(path: String) -> Result<detect::DriveInfo, String> {
    let drives = tokio::task::spawn_blocking(|| detect::list_usb_drives())
        .await
        .map_err(|e| format!("Task error: {}", e))??;
    drives
        .into_iter()
        .find(|d| d.path == path || d.mount_point == path)
        .ok_or_else(|| format!("Drive not found: {}", path))
}
