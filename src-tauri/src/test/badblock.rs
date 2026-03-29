use super::common::*;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct BadBlockResult {
    pub total_blocks: u64,
    pub bad_blocks: Vec<u64>,
    pub tested_blocks: u64,
}

const BLOCK_SIZE: usize = 1_048_576; // 1MB

pub fn run_badblock_test<F>(
    mount_point: &str,
    progress_cb: F,
) -> Result<BadBlockResult, String>
where
    F: Fn(f64, &str, f64) + Send,
{
    let test_dir = PathBuf::from(mount_point).join("_udisk_badblock_test_");
    fs::create_dir_all(&test_dir).map_err(|e| format!("Cannot create test dir: {}", e))?;

    // Determine test size from free space
    let free_space = super::capacity::fs_free_space(mount_point)?;
    let num_blocks = free_space / BLOCK_SIZE as u64;

    if num_blocks == 0 {
        return Err("No free space for bad block test".to_string());
    }

    let mut bad_blocks = Vec::new();
    let mut tested_blocks = 0u64;

    // Write phase
    progress_cb(0.0, "坏块扫描：写入测试数据...", 0.0);
    for i in 0..num_blocks {
        if should_stop() {
            let _ = fs::remove_dir_all(&test_dir);
            return Err("Test stopped by user".to_string());
        }

        let block_data = generate_test_block(i, BLOCK_SIZE);
        let file_path = test_dir.join(format!("bb_{:08}.dat", i));

        match fs::File::create(&file_path).and_then(|mut f| {
            f.write_all(&block_data)?;
            f.sync_all()?;
            Ok(())
        }) {
            Ok(()) => {}
            Err(e) => {
                log::warn!("Write failed at block {}: {}", i, e);
                bad_blocks.push(i);
            }
        }

        let progress = (i + 1) as f64 / (num_blocks * 2) as f64;
        if i % 10 == 0 {
            progress_cb(
                progress,
                &format!("写入 {}/{} 块", i + 1, num_blocks),
                0.0,
            );
        }
    }

    // Read and verify phase
    progress_cb(0.5, "坏块扫描：验证数据...", 0.0);
    for i in 0..num_blocks {
        if should_stop() {
            let _ = fs::remove_dir_all(&test_dir);
            return Err("Test stopped by user".to_string());
        }

        let file_path = test_dir.join(format!("bb_{:08}.dat", i));

        match fs::File::open(&file_path).and_then(|mut f| {
            let mut buf = vec![0u8; BLOCK_SIZE];
            f.read_exact(&mut buf)?;
            Ok(buf)
        }) {
            Ok(data) => {
                match verify_test_block(&data, i) {
                    BlockVerifyResult::Ok => {}
                    _ => {
                        if !bad_blocks.contains(&i) {
                            bad_blocks.push(i);
                        }
                    }
                }
                tested_blocks += 1;
            }
            Err(e) => {
                log::warn!("Read failed at block {}: {}", i, e);
                if !bad_blocks.contains(&i) {
                    bad_blocks.push(i);
                }
            }
        }

        let progress = 0.5 + (i + 1) as f64 / (num_blocks * 2) as f64;
        if i % 10 == 0 {
            progress_cb(
                progress,
                &format!(
                    "验证 {}/{} 块 | 坏块: {}",
                    i + 1,
                    num_blocks,
                    bad_blocks.len()
                ),
                0.0,
            );
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);

    Ok(BadBlockResult {
        total_blocks: num_blocks,
        bad_blocks,
        tested_blocks,
    })
}
