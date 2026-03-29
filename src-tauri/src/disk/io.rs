use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub const BLOCK_SIZE: usize = 1_048_576; // 1MB
pub const ALIGN_SIZE: usize = 4096;

/// Write a block of data to a test file on the drive's mount point.
/// Uses file-based approach with sync to bypass write cache as much as possible.
pub fn write_block(path: &Path, offset: u64, data: &[u8]) -> Result<(), String> {
    let block_index = offset / BLOCK_SIZE as u64;
    let file_path = path.join(format!("_udisk_test_block_{:06}.dat", block_index));

    let mut file =
        fs::File::create(&file_path).map_err(|e| format!("Failed to create block file: {}", e))?;
    file.write_all(data)
        .map_err(|e| format!("Failed to write block: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync block: {}", e))?;
    drop(file);

    Ok(())
}

/// Read a block of data from a test file on the drive's mount point.
/// Drops and reopens the file handle to avoid OS read cache.
pub fn read_block(path: &Path, offset: u64, size: usize) -> Result<Vec<u8>, String> {
    let block_index = offset / BLOCK_SIZE as u64;
    let file_path = path.join(format!("_udisk_test_block_{:06}.dat", block_index));

    // Open fresh handle to bypass read cache
    let mut file =
        fs::File::open(&file_path).map_err(|e| format!("Failed to open block file: {}", e))?;
    let mut buf = vec![0u8; size];
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read block: {}", e))?;
    drop(file);

    Ok(buf)
}

/// Attempt to flush/sync the entire drive. On most OSes this is a best-effort operation.
pub fn sync_drive(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, we rely on File::sync_all() per file.
        // Optionally call FlushFileBuffers on the volume handle, but that requires admin.
        let _ = path;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        let _ = path;
        Command::new("sync")
            .output()
            .map_err(|e| format!("sync failed: {}", e))?;
        Ok(())
    }
}
