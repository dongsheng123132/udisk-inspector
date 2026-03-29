use super::common::*;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

pub struct SpeedResult {
    pub seq_write_speed: f64,
    pub seq_read_speed: f64,
    pub random_read_iops: f64,
    pub random_write_iops: f64,
    pub write_samples: Vec<SpeedSample>,
    pub read_samples: Vec<SpeedSample>,
    pub stability: f64,
    pub speed_drops: u32,
    pub min_write_speed: f64,
    pub max_write_speed: f64,
}

pub struct SpeedSample {
    pub offset_mb: u64,
    pub speed_mbps: f64,
}

const CHUNK_SIZE: usize = 32 * 1024 * 1024; // 32MB sampling chunks
const RANDOM_BLOCK_SIZE: usize = 4096; // 4K for random IO
const RANDOM_ITERATIONS: u32 = 1000;

pub fn run_speed_test<F>(
    mount_point: &str,
    test_size_mb: u64,
    progress_cb: F,
) -> Result<SpeedResult, String>
where
    F: Fn(f64, &str, f64) + Send,
{
    let test_file = PathBuf::from(mount_point).join("_udisk_speed_test_.dat");
    let total_bytes = test_size_mb * 1024 * 1024;
    let num_chunks = (total_bytes as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

    // Generate a reusable data chunk
    let mut rng = PseudoRandom::new(0xDEADBEEF);
    let mut chunk_data = vec![0u8; CHUNK_SIZE];
    rng.fill_buffer(&mut chunk_data);

    // === Sequential Write ===
    progress_cb(0.0, "顺序写入测速...", 0.0);
    let mut write_samples = Vec::new();
    let mut file =
        fs::File::create(&test_file).map_err(|e| format!("Cannot create test file: {}", e))?;

    let write_start = Instant::now();
    for i in 0..num_chunks {
        if should_stop() {
            let _ = fs::remove_file(&test_file);
            return Err("Test stopped by user".to_string());
        }

        let actual_size = if i == num_chunks - 1 {
            (total_bytes as usize) - i * CHUNK_SIZE
        } else {
            CHUNK_SIZE
        };

        let chunk_start = Instant::now();
        file.write_all(&chunk_data[..actual_size])
            .map_err(|e| format!("Write error: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Sync error: {}", e))?;
        let chunk_elapsed = chunk_start.elapsed().as_secs_f64();

        let speed = if chunk_elapsed > 0.0 {
            actual_size as f64 / chunk_elapsed / 1_000_000.0
        } else {
            0.0
        };
        write_samples.push(SpeedSample {
            offset_mb: (i * CHUNK_SIZE / (1024 * 1024)) as u64,
            speed_mbps: speed,
        });

        let progress = (i + 1) as f64 / (num_chunks * 3) as f64;
        progress_cb(
            progress,
            &format!(
                "写入 {} MB / {} MB",
                (i + 1) * CHUNK_SIZE / (1024 * 1024),
                test_size_mb
            ),
            speed,
        );
    }
    let write_total = write_start.elapsed().as_secs_f64();
    let seq_write_speed = if write_total > 0.0 {
        total_bytes as f64 / write_total / 1_000_000.0
    } else {
        0.0
    };
    drop(file);

    // === Sequential Read ===
    progress_cb(0.33, "顺序读取测速...", 0.0);
    let mut read_samples = Vec::new();
    let mut file =
        fs::File::open(&test_file).map_err(|e| format!("Cannot open test file: {}", e))?;
    let mut read_buf = vec![0u8; CHUNK_SIZE];

    let read_start = Instant::now();
    for i in 0..num_chunks {
        if should_stop() {
            let _ = fs::remove_file(&test_file);
            return Err("Test stopped by user".to_string());
        }

        let actual_size = if i == num_chunks - 1 {
            (total_bytes as usize) - i * CHUNK_SIZE
        } else {
            CHUNK_SIZE
        };

        let chunk_start = Instant::now();
        file.read_exact(&mut read_buf[..actual_size])
            .map_err(|e| format!("Read error: {}", e))?;
        let chunk_elapsed = chunk_start.elapsed().as_secs_f64();

        let speed = if chunk_elapsed > 0.0 {
            actual_size as f64 / chunk_elapsed / 1_000_000.0
        } else {
            0.0
        };
        read_samples.push(SpeedSample {
            offset_mb: (i * CHUNK_SIZE / (1024 * 1024)) as u64,
            speed_mbps: speed,
        });

        let progress = 0.33 + (i + 1) as f64 / (num_chunks * 3) as f64;
        progress_cb(
            progress,
            &format!(
                "读取 {} MB / {} MB",
                (i + 1) * CHUNK_SIZE / (1024 * 1024),
                test_size_mb
            ),
            speed,
        );
    }
    let read_total = read_start.elapsed().as_secs_f64();
    let seq_read_speed = if read_total > 0.0 {
        total_bytes as f64 / read_total / 1_000_000.0
    } else {
        0.0
    };
    drop(file);

    // === Random 4K Read/Write ===
    progress_cb(0.67, "随机4K读写测试...", 0.0);
    let file_size = total_bytes;
    let max_offset = if file_size > RANDOM_BLOCK_SIZE as u64 {
        file_size - RANDOM_BLOCK_SIZE as u64
    } else {
        0
    };

    // Random Write
    let mut rng = PseudoRandom::new(0xCAFEBABE);
    let mut small_buf = vec![0u8; RANDOM_BLOCK_SIZE];
    rng.fill_buffer(&mut small_buf);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(&test_file)
        .map_err(|e| format!("Cannot open for random write: {}", e))?;

    let rw_start = Instant::now();
    let mut rng_pos = PseudoRandom::new(42);
    for i in 0..RANDOM_ITERATIONS {
        if should_stop() {
            let _ = fs::remove_file(&test_file);
            return Err("Test stopped by user".to_string());
        }
        let offset = rng_pos.next() % (max_offset.max(1));
        let aligned_offset = (offset / 4096) * 4096;
        file.seek(SeekFrom::Start(aligned_offset))
            .map_err(|e| e.to_string())?;
        file.write_all(&small_buf)
            .map_err(|e| e.to_string())?;

        if i % 100 == 0 {
            let progress = 0.67 + (i as f64 / RANDOM_ITERATIONS as f64) * 0.165;
            progress_cb(
                progress,
                &format!("随机写入 {}/{}", i, RANDOM_ITERATIONS),
                0.0,
            );
        }
    }
    file.sync_all().map_err(|e| e.to_string())?;
    let rw_elapsed = rw_start.elapsed().as_secs_f64();
    let random_write_iops = if rw_elapsed > 0.0 {
        RANDOM_ITERATIONS as f64 / rw_elapsed
    } else {
        0.0
    };
    drop(file);

    // Random Read
    let mut file =
        fs::File::open(&test_file).map_err(|e| format!("Cannot open for random read: {}", e))?;

    let rr_start = Instant::now();
    let mut rng_pos = PseudoRandom::new(42);
    for i in 0..RANDOM_ITERATIONS {
        if should_stop() {
            let _ = fs::remove_file(&test_file);
            return Err("Test stopped by user".to_string());
        }
        let offset = rng_pos.next() % (max_offset.max(1));
        let aligned_offset = (offset / 4096) * 4096;
        file.seek(SeekFrom::Start(aligned_offset))
            .map_err(|e| e.to_string())?;
        file.read_exact(&mut small_buf)
            .map_err(|e| e.to_string())?;

        if i % 100 == 0 {
            let progress = 0.835 + (i as f64 / RANDOM_ITERATIONS as f64) * 0.165;
            progress_cb(
                progress,
                &format!("随机读取 {}/{}", i, RANDOM_ITERATIONS),
                0.0,
            );
        }
    }
    let rr_elapsed = rr_start.elapsed().as_secs_f64();
    let random_read_iops = if rr_elapsed > 0.0 {
        RANDOM_ITERATIONS as f64 / rr_elapsed
    } else {
        0.0
    };
    drop(file);

    // Cleanup
    let _ = fs::remove_file(&test_file);

    // Stability analysis
    let (stability, speed_drops, min_write, max_write) = analyze_stability(&write_samples);

    Ok(SpeedResult {
        seq_write_speed,
        seq_read_speed,
        random_read_iops,
        random_write_iops,
        write_samples,
        read_samples,
        stability,
        speed_drops,
        min_write_speed: min_write,
        max_write_speed: max_write,
    })
}

fn analyze_stability(samples: &[SpeedSample]) -> (f64, u32, f64, f64) {
    if samples.is_empty() {
        return (1.0, 0, 0.0, 0.0);
    }

    let speeds: Vec<f64> = samples.iter().map(|s| s.speed_mbps).collect();
    let avg = speeds.iter().sum::<f64>() / speeds.len() as f64;
    let min = speeds.iter().cloned().fold(f64::MAX, f64::min);
    let max = speeds.iter().cloned().fold(f64::MIN, f64::max);

    // Standard deviation
    let variance = speeds.iter().map(|s| (s - avg).powi(2)).sum::<f64>() / speeds.len() as f64;
    let std_dev = variance.sqrt();

    // Coefficient of variation (lower = more stable)
    let cv = if avg > 0.0 { std_dev / avg } else { 1.0 };
    let stability = (1.0 - cv).max(0.0).min(1.0);

    // Count significant drops (>50% from previous sample)
    let mut drops = 0u32;
    for window in speeds.windows(2) {
        if window[1] < window[0] * 0.5 {
            drops += 1;
        }
    }

    (stability, drops, min, max)
}
