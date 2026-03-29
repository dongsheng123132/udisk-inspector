use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub mount_point: String,
    pub serial: String,
    pub capacity_bytes: u64,
    pub free_bytes: u64,
    pub file_system: String,
    pub is_removable: bool,
}

pub fn list_usb_drives() -> Result<Vec<DriveInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        list_usb_drives_windows()
    }
    #[cfg(target_os = "macos")]
    {
        list_usb_drives_macos()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

#[cfg(target_os = "windows")]
fn list_usb_drives_windows() -> Result<Vec<DriveInfo>, String> {
    // Use wmic first (more stable, no COM issues), WMI only as enhancement
    let mut drives = list_usb_drives_wmic_fallback().unwrap_or_default();

    // Try WMI to get better device names/serials, but don't fail if it errors
    if let Ok(wmi_drives) = std::panic::catch_unwind(|| list_usb_drives_wmi()) {
        if let Ok(wmi_drives) = wmi_drives {
            for wmi_drive in &wmi_drives {
                if let Some(existing) = drives.iter_mut().find(|d| d.path == wmi_drive.path) {
                    if !wmi_drive.name.is_empty() && wmi_drive.name != "Removable Disk" {
                        existing.name = wmi_drive.name.clone();
                    }
                    if !wmi_drive.serial.is_empty() && existing.serial.is_empty() {
                        existing.serial = wmi_drive.serial.clone();
                    }
                }
            }
        }
    }

    Ok(drives)
}

#[cfg(target_os = "windows")]
fn list_usb_drives_wmi() -> Result<Vec<DriveInfo>, String> {
    use std::collections::HashMap;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    #[allow(dead_code)]
    struct Win32DiskDrive {
        #[serde(default)]
        caption: Option<String>,
        #[serde(default)]
        device_id: Option<String>,
        #[serde(default)]
        interface_type: Option<String>,
        #[serde(default)]
        serial_number: Option<String>,
        #[serde(default)]
        size: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct Win32DiskDriveToDiskPartition {
        #[serde(default)]
        antecedent: Option<String>,
        #[serde(default)]
        dependent: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct Win32LogicalDiskToPartition {
        #[serde(default)]
        antecedent: Option<String>,
        #[serde(default)]
        dependent: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    #[allow(dead_code)]
    struct Win32LogicalDisk {
        #[serde(default)]
        device_id: Option<String>,
        #[serde(default)]
        file_system: Option<String>,
        #[serde(default)]
        free_space: Option<String>,
        #[serde(default)]
        size: Option<String>,
        #[serde(default)]
        volume_name: Option<String>,
        #[serde(default)]
        volume_serial_number: Option<String>,
        #[serde(default)]
        drive_type: Option<u32>,
    }

    let com_con = COMLibrary::new().map_err(|e| format!("COM init failed: {}", e))?;
    let wmi_con =
        WMIConnection::new(com_con).map_err(|e| format!("WMI connection failed: {}", e))?;

    // Get USB disk drives
    let disk_drives: Vec<Win32DiskDrive> = wmi_con
        .raw_query("SELECT Caption, DeviceID, InterfaceType, SerialNumber, Size FROM Win32_DiskDrive WHERE InterfaceType='USB'")
        .map_err(|e| format!("WMI query failed: {}", e))?;

    if disk_drives.is_empty() {
        return Ok(Vec::new());
    }

    // Get partition mappings
    let drive_to_part: Vec<Win32DiskDriveToDiskPartition> = wmi_con
        .raw_query("SELECT Antecedent, Dependent FROM Win32_DiskDriveToDiskPartition")
        .map_err(|e| format!("WMI query failed: {}", e))?;

    let part_to_logical: Vec<Win32LogicalDiskToPartition> = wmi_con
        .raw_query("SELECT Antecedent, Dependent FROM Win32_LogicalDiskToPartition")
        .map_err(|e| format!("WMI query failed: {}", e))?;

    // Get logical disks
    let logical_disks: Vec<Win32LogicalDisk> = wmi_con
        .raw_query("SELECT DeviceID, FileSystem, FreeSpace, Size, VolumeName, VolumeSerialNumber, DriveType FROM Win32_LogicalDisk")
        .map_err(|e| format!("WMI query failed: {}", e))?;

    let logical_map: HashMap<String, &Win32LogicalDisk> = logical_disks
        .iter()
        .filter_map(|ld| ld.device_id.as_ref().map(|id| (id.clone(), ld)))
        .collect();

    let mut drives = Vec::new();

    for disk in &disk_drives {
        let disk_device_id = match &disk.device_id {
            Some(id) => id.clone(),
            None => continue,
        };

        // Find partitions for this disk drive
        for d2p in &drive_to_part {
            let antecedent = match &d2p.antecedent {
                Some(a) => a,
                None => continue,
            };
            if !antecedent.contains(&disk_device_id.replace("\\", "\\\\")) {
                continue;
            }
            let partition_ref = match &d2p.dependent {
                Some(d) => d,
                None => continue,
            };

            // Find logical disk for this partition
            for p2l in &part_to_logical {
                let p_ant = match &p2l.antecedent {
                    Some(a) => a,
                    None => continue,
                };
                if !partition_ref
                    .split('"')
                    .nth(1)
                    .map(|pname| p_ant.contains(pname))
                    .unwrap_or(false)
                {
                    continue;
                }

                let logical_ref = match &p2l.dependent {
                    Some(d) => d,
                    None => continue,
                };
                // Extract drive letter like "C:"
                let drive_letter = logical_ref
                    .split('"')
                    .nth(1)
                    .unwrap_or("")
                    .to_string();

                if let Some(ld) = logical_map.get(&drive_letter) {
                    let capacity: u64 = ld
                        .size
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let free: u64 = ld
                        .free_space
                        .as_ref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    drives.push(DriveInfo {
                        name: disk.caption.clone().unwrap_or_else(|| "USB Drive".to_string()),
                        path: drive_letter.clone(),
                        mount_point: format!("{}\\", drive_letter),
                        serial: disk
                            .serial_number
                            .clone()
                            .or_else(|| ld.volume_serial_number.clone())
                            .unwrap_or_default(),
                        capacity_bytes: capacity,
                        free_bytes: free,
                        file_system: ld.file_system.clone().unwrap_or_default(),
                        is_removable: true,
                    });
                }
            }
        }
    }

    Ok(drives)
}

#[cfg(target_os = "windows")]
fn list_usb_drives_wmic_fallback() -> Result<Vec<DriveInfo>, String> {
    use std::process::Command;

    let output = Command::new("wmic")
        .args([
            "logicaldisk",
            "where",
            "drivetype=2",
            "get",
            "Caption,Description,FileSystem,FreeSpace,Size,VolumeName,VolumeSerialNumber",
            "/format:csv",
        ])
        .output()
        .map_err(|e| format!("Failed to run wmic: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut drives = Vec::new();

    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        // CSV format: Node,Caption,Description,FileSystem,FreeSpace,Size,VolumeName,VolumeSerialNumber
        if parts.len() < 8 {
            continue;
        }

        let caption = parts[1].trim();
        let description = parts[2].trim();
        let file_system = parts[3].trim();
        let free_space: u64 = parts[4].trim().parse().unwrap_or(0);
        let size: u64 = parts[5].trim().parse().unwrap_or(0);
        let volume_name = parts[6].trim();
        let volume_serial = parts[7].trim();

        if size == 0 {
            continue;
        }

        let name = if !volume_name.is_empty() {
            volume_name.to_string()
        } else if !description.is_empty() {
            description.to_string()
        } else {
            "Removable Disk".to_string()
        };

        drives.push(DriveInfo {
            name,
            path: caption.to_string(),
            mount_point: format!("{}\\", caption),
            serial: volume_serial.to_string(),
            capacity_bytes: size,
            free_bytes: free_space,
            file_system: file_system.to_string(),
            is_removable: true,
        });
    }

    // Also try to get physical disk names from USB disk drives
    if let Ok(phys_output) = Command::new("wmic")
        .args([
            "diskdrive",
            "where",
            "interfacetype='USB'",
            "get",
            "Caption,SerialNumber",
            "/format:csv",
        ])
        .output()
    {
        let phys_stdout = String::from_utf8_lossy(&phys_output.stdout);
        let mut physical_names: Vec<(String, String)> = Vec::new();
        for line in phys_stdout.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                physical_names.push((parts[1].trim().to_string(), parts[2].trim().to_string()));
            }
        }

        // If we have exactly one physical USB drive and one logical removable, match them
        if physical_names.len() == 1 && drives.len() == 1 {
            if !physical_names[0].0.is_empty() {
                drives[0].name = physical_names[0].0.clone();
            }
            if !physical_names[0].1.is_empty() && drives[0].serial.is_empty() {
                drives[0].serial = physical_names[0].1.clone();
            }
        }
    }

    Ok(drives)
}

#[cfg(target_os = "macos")]
fn list_usb_drives_macos() -> Result<Vec<DriveInfo>, String> {
    use std::process::Command;

    // Get list of external disks
    let output = Command::new("diskutil")
        .args(["list", "-plist", "external"])
        .output()
        .map_err(|e| format!("Failed to run diskutil: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Simple plist XML parsing to extract disk identifiers
    let mut disk_ids: Vec<String> = Vec::new();
    let mut in_whole_disks = false;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("WholeDisks") {
            in_whole_disks = true;
            continue;
        }
        if in_whole_disks {
            if trimmed == "</array>" {
                in_whole_disks = false;
                continue;
            }
            if trimmed.starts_with("<string>") && trimmed.ends_with("</string>") {
                let disk_id = trimmed
                    .trim_start_matches("<string>")
                    .trim_end_matches("</string>");
                disk_ids.push(disk_id.to_string());
            }
        }
    }

    let mut drives = Vec::new();

    for disk_id in &disk_ids {
        let info_output = Command::new("diskutil")
            .args(["info", "-plist", disk_id])
            .output()
            .map_err(|e| format!("Failed to get disk info: {}", e))?;

        let info_str = String::from_utf8_lossy(&info_output.stdout);

        let name = extract_plist_string(&info_str, "MediaName")
            .unwrap_or_else(|| "USB Drive".to_string());
        let mount_point =
            extract_plist_string(&info_str, "MountPoint").unwrap_or_else(|| String::new());
        let size = extract_plist_integer(&info_str, "TotalSize").unwrap_or(0);
        let free = extract_plist_integer(&info_str, "FreeSpace")
            .or_else(|| extract_plist_integer(&info_str, "APFSContainerFree"))
            .unwrap_or(0);
        let fs = extract_plist_string(&info_str, "FilesystemType").unwrap_or_default();
        let removable = extract_plist_bool(&info_str, "Removable").unwrap_or(true);

        // Also check partitions for mount points
        if mount_point.is_empty() {
            // Try to find mounted partitions: disk2s1, disk2s2, etc.
            let list_output = Command::new("diskutil")
                .args(["list", "-plist", disk_id])
                .output()
                .ok();

            if let Some(lo) = list_output {
                let lo_str = String::from_utf8_lossy(&lo.stdout);
                // Extract partition identifiers and check each
                let mut part_ids: Vec<String> = Vec::new();
                let mut in_all_disks = false;
                for line in lo_str.lines() {
                    let t = line.trim();
                    if t.contains("AllDisksAndPartitions") || t.contains("Partitions") {
                        in_all_disks = true;
                        continue;
                    }
                    if in_all_disks && t.starts_with("<string>") && t.ends_with("</string>") {
                        let pid = t
                            .trim_start_matches("<string>")
                            .trim_end_matches("</string>");
                        if pid.contains('s') && pid != *disk_id {
                            part_ids.push(pid.to_string());
                        }
                    }
                }

                for pid in &part_ids {
                    if let Ok(pinfo) = Command::new("diskutil")
                        .args(["info", "-plist", pid])
                        .output()
                    {
                        let pinfo_str = String::from_utf8_lossy(&pinfo.stdout);
                        if let Some(mp) = extract_plist_string(&pinfo_str, "MountPoint") {
                            if !mp.is_empty() {
                                let p_fs = extract_plist_string(&pinfo_str, "FilesystemType")
                                    .unwrap_or_default();
                                let p_size =
                                    extract_plist_integer(&pinfo_str, "TotalSize").unwrap_or(size);
                                let p_free =
                                    extract_plist_integer(&pinfo_str, "FreeSpace").unwrap_or(free);

                                drives.push(DriveInfo {
                                    name: name.clone(),
                                    path: format!("/dev/{}", disk_id),
                                    mount_point: mp,
                                    serial: String::new(),
                                    capacity_bytes: p_size,
                                    free_bytes: p_free,
                                    file_system: p_fs,
                                    is_removable: removable,
                                });
                            }
                        }
                    }
                }
            }
        } else {
            drives.push(DriveInfo {
                name,
                path: format!("/dev/{}", disk_id),
                mount_point,
                serial: String::new(),
                capacity_bytes: size,
                free_bytes: free,
                file_system: fs,
                is_removable: removable,
            });
        }
    }

    Ok(drives)
}

#[cfg(target_os = "macos")]
fn extract_plist_string(plist: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    let mut found_key = false;
    for line in plist.lines() {
        let trimmed = line.trim();
        if trimmed == key_tag {
            found_key = true;
            continue;
        }
        if found_key {
            if trimmed.starts_with("<string>") && trimmed.ends_with("</string>") {
                return Some(
                    trimmed
                        .trim_start_matches("<string>")
                        .trim_end_matches("</string>")
                        .to_string(),
                );
            }
            return None;
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn extract_plist_integer(plist: &str, key: &str) -> Option<u64> {
    let key_tag = format!("<key>{}</key>", key);
    let mut found_key = false;
    for line in plist.lines() {
        let trimmed = line.trim();
        if trimmed == key_tag {
            found_key = true;
            continue;
        }
        if found_key {
            if trimmed.starts_with("<integer>") && trimmed.ends_with("</integer>") {
                return trimmed
                    .trim_start_matches("<integer>")
                    .trim_end_matches("</integer>")
                    .parse()
                    .ok();
            }
            return None;
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn extract_plist_bool(plist: &str, key: &str) -> Option<bool> {
    let key_tag = format!("<key>{}</key>", key);
    let mut found_key = false;
    for line in plist.lines() {
        let trimmed = line.trim();
        if trimmed == key_tag {
            found_key = true;
            continue;
        }
        if found_key {
            if trimmed == "<true/>" {
                return Some(true);
            }
            if trimmed == "<false/>" {
                return Some(false);
            }
            return None;
        }
    }
    None
}
