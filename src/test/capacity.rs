use super::common::*;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct CapacityResult {
    pub claimed_bytes: u64,
    pub real_bytes: u64,
    pub total_blocks: u64,
    pub good_blocks: u64,
    pub bad_blocks: Vec<u64>,
    pub remapped_blocks: Vec<(u64, u64)>,
}

const BLOCK_SIZE: usize = 1_048_576; // 1MB

pub fn run_capacity_test<F>(
    mount_point: &str,
    claimed_capacity: u64,
    progress_cb: F,
) -> Result<CapacityResult, String>
where
    F: Fn(f64, &str, f64) + Send,
{
    let test_dir = PathBuf::from(mount_point).join("_udisk_test_");
    fs::create_dir_all(&test_dir).map_err(|e| format!("Cannot create test dir: {}", e))?;

    // Calculate how many blocks we can write based on free space
    let free_space = fs_free_space(mount_point)?;
    let num_blocks = (free_space / BLOCK_SIZE as u64).min(claimed_capacity / BLOCK_SIZE as u64);

    if num_blocks == 0 {
        return Err("No free space available for testing".to_string());
    }

    // Phase 1: Write
    progress_cb(0.0, "开始写入测试数据...", 0.0);
    let mut written_blocks = 0u64;

    for i in 0..num_blocks {
        if should_stop() {
            cleanup_test_files(&test_dir);
            return Err("Test stopped by user".to_string());
        }

        let block_data = generate_test_block(i, BLOCK_SIZE);
        let file_path = test_dir.join(format!("block_{:06}.dat", i));

        let start = Instant::now();
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Write error at block {}: {}", i, e))?;
        file.write_all(&block_data)
            .map_err(|e| format!("Write error at block {}: {}", i, e))?;
        file.sync_all()
            .map_err(|e| format!("Sync error at block {}: {}", i, e))?;
        drop(file);

        let elapsed = start.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            BLOCK_SIZE as f64 / elapsed / 1_000_000.0
        } else {
            0.0
        };

        written_blocks += 1;
        let progress = written_blocks as f64 / (num_blocks * 2) as f64;
        progress_cb(
            progress,
            &format!("写入块 {}/{}", written_blocks, num_blocks),
            speed,
        );
    }

    // Phase 2: Verify
    progress_cb(0.5, "开始验证数据...", 0.0);
    let mut good_blocks = 0u64;
    let mut bad_blocks = Vec::new();
    let mut remapped_blocks = Vec::new();

    for i in 0..written_blocks {
        if should_stop() {
            cleanup_test_files(&test_dir);
            return Err("Test stopped by user".to_string());
        }

        let file_path = test_dir.join(format!("block_{:06}.dat", i));
        let start = Instant::now();

        let mut file =
            fs::File::open(&file_path).map_err(|e| format!("Read error at block {}: {}", i, e))?;
        let mut read_data = vec![0u8; BLOCK_SIZE];
        file.read_exact(&mut read_data)
            .map_err(|e| format!("Read error at block {}: {}", i, e))?;
        drop(file);

        let elapsed = start.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            BLOCK_SIZE as f64 / elapsed / 1_000_000.0
        } else {
            0.0
        };

        match verify_test_block(&read_data, i) {
            BlockVerifyResult::Ok => good_blocks += 1,
            BlockVerifyResult::WrongMapping { expected, got } => {
                remapped_blocks.push((expected, got));
                bad_blocks.push(i);
            }
            BlockVerifyResult::DataCorruption | BlockVerifyResult::Error => {
                bad_blocks.push(i);
            }
        }

        let progress = 0.5 + (i + 1) as f64 / (written_blocks * 2) as f64;
        progress_cb(
            progress,
            &format!("验证块 {}/{}", i + 1, written_blocks),
            speed,
        );
    }

    // Cleanup
    cleanup_test_files(&test_dir);

    Ok(CapacityResult {
        claimed_bytes: claimed_capacity,
        real_bytes: good_blocks * BLOCK_SIZE as u64,
        total_blocks: written_blocks,
        good_blocks,
        bad_blocks,
        remapped_blocks,
    })
}

fn cleanup_test_files(test_dir: &Path) {
    let _ = fs::remove_dir_all(test_dir);
}

pub fn fs_free_space(mount_point: &str) -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let drive_id = mount_point.trim_end_matches('\\');
        let output = Command::new("wmic")
            .args([
                "logicaldisk",
                "where",
                &format!("DeviceID='{}'", drive_id),
                "get",
                "FreeSpace",
                "/value",
            ])
            .output()
            .map_err(|e| format!("Failed to get free space: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(val) = line.strip_prefix("FreeSpace=") {
                if let Ok(bytes) = val.trim().parse::<u64>() {
                    return Ok(bytes);
                }
            }
        }
        Err("Could not determine free space".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        let output = Command::new("df")
            .args(["-B1", mount_point])
            .output()
            .map_err(|e| format!("Failed to get free space: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(bytes) = parts[3].parse::<u64>() {
                    return Ok(bytes);
                }
            }
        }
        Err("Could not determine free space".to_string())
    }
}
