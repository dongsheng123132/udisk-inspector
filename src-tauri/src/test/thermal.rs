use super::common::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

pub struct ThermalResult {
    pub sustained_write_samples: Vec<(f64, f64)>, // (elapsed_secs, speed_mbps)
    pub avg_speed_first_minute: f64,
    pub avg_speed_last_minute: f64,
    pub speed_degradation: f64,
    pub thermal_risk: ThermalRisk,
    pub drop_count: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ThermalRisk {
    Low,
    Medium,
    High,
}

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB chunks for sustained write

pub fn run_thermal_test<F>(
    mount_point: &str,
    test_duration_secs: u64,
    progress_cb: F,
) -> Result<ThermalResult, String>
where
    F: Fn(f64, &str, f64) + Send,
{
    let test_file = PathBuf::from(mount_point).join("_udisk_thermal_test_.dat");
    let mut rng = PseudoRandom::new(0x7748524D414C);
    let mut chunk_data = vec![0u8; CHUNK_SIZE];
    rng.fill_buffer(&mut chunk_data);

    let mut file = fs::File::create(&test_file)
        .map_err(|e| format!("Cannot create thermal test file: {}", e))?;

    let test_start = Instant::now();
    let mut samples = Vec::new();
    let mut total_written = 0u64;

    progress_cb(0.0, "持续写入压力测试...", 0.0);

    loop {
        let elapsed = test_start.elapsed().as_secs_f64();
        if elapsed >= test_duration_secs as f64 || should_stop() {
            break;
        }

        let chunk_start = Instant::now();
        match file.write_all(&chunk_data) {
            Ok(()) => {
                let _ = file.sync_all();
                let chunk_time = chunk_start.elapsed().as_secs_f64();
                let speed = if chunk_time > 0.0 {
                    CHUNK_SIZE as f64 / chunk_time / 1_000_000.0
                } else {
                    0.0
                };
                samples.push((elapsed, speed));
                total_written += CHUNK_SIZE as u64;
            }
            Err(_) => {
                // Disk might be full, stop
                break;
            }
        }

        let progress = elapsed / test_duration_secs as f64;
        let current_speed = samples.last().map(|s| s.1).unwrap_or(0.0);
        progress_cb(
            progress.min(1.0),
            &format!(
                "持续写入 {:.0}s / {}s | 已写入 {} MB",
                elapsed,
                test_duration_secs,
                total_written / (1024 * 1024)
            ),
            current_speed,
        );
    }

    drop(file);
    let _ = fs::remove_file(&test_file);

    // Analyze results
    let (avg_first, avg_last, degradation, drops) =
        analyze_thermal(&samples, test_duration_secs as f64);

    let thermal_risk = if degradation > 40.0 || drops > 5 {
        ThermalRisk::High
    } else if degradation > 20.0 || drops > 2 {
        ThermalRisk::Medium
    } else {
        ThermalRisk::Low
    };

    Ok(ThermalResult {
        sustained_write_samples: samples,
        avg_speed_first_minute: avg_first,
        avg_speed_last_minute: avg_last,
        speed_degradation: degradation,
        thermal_risk,
        drop_count: drops,
    })
}

fn analyze_thermal(samples: &[(f64, f64)], total_secs: f64) -> (f64, f64, f64, u32) {
    if samples.is_empty() {
        return (0.0, 0.0, 0.0, 0);
    }

    let first_minute: Vec<f64> = samples
        .iter()
        .filter(|(t, _)| *t < 60.0)
        .map(|(_, s)| *s)
        .collect();

    let last_start = (total_secs - 60.0).max(60.0);
    let last_minute: Vec<f64> = samples
        .iter()
        .filter(|(t, _)| *t >= last_start)
        .map(|(_, s)| *s)
        .collect();

    let avg_first = if first_minute.is_empty() {
        0.0
    } else {
        first_minute.iter().sum::<f64>() / first_minute.len() as f64
    };
    let avg_last = if last_minute.is_empty() {
        avg_first
    } else {
        last_minute.iter().sum::<f64>() / last_minute.len() as f64
    };

    let degradation = if avg_first > 0.0 {
        ((avg_first - avg_last) / avg_first * 100.0).max(0.0)
    } else {
        0.0
    };

    // Count significant drops
    let mut drops = 0u32;
    let speeds: Vec<f64> = samples.iter().map(|(_, s)| *s).collect();
    for window in speeds.windows(2) {
        if window[1] < window[0] * 0.5 {
            drops += 1;
        }
    }

    (avg_first, avg_last, degradation, drops)
}
