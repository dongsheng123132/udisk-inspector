use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

pub static STOP_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub drive_path: String,
    pub mount_point: String,
    pub test_capacity: bool,
    pub test_speed: bool,
    pub test_badblock: bool,
    pub destructive: bool,
    pub test_size_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestProgress {
    pub test_type: String,
    pub phase: String,
    pub progress: f64,
    pub current_speed: f64,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub report_id: String,
    pub drive_name: String,
    pub claimed_capacity: u64,
    pub real_capacity: Option<u64>,
    pub seq_read_speed: Option<f64>,
    pub seq_write_speed: Option<f64>,
    pub random_read_iops: Option<f64>,
    pub random_write_iops: Option<f64>,
    pub speed_stability: Option<f64>,
    pub speed_samples: Vec<SpeedSample>,
    pub bad_block_count: Option<u64>,
    pub total_blocks: Option<u64>,
    pub bad_block_positions: Vec<u64>,
    pub total_score: u32,
    pub capacity_score: u32,
    pub speed_score: u32,
    pub stability_score: u32,
    pub badblock_score: u32,
    pub test_duration_secs: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeedSample {
    pub offset_mb: u64,
    pub write_speed: f64,
    pub read_speed: f64,
}

#[tauri::command]
pub async fn start_test(
    config: TestConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<TestResult, String> {
    STOP_FLAG.store(false, Ordering::Relaxed);

    let start_time = std::time::Instant::now();

    // Get drive info
    let drive_info = crate::disk::detect::list_usb_drives()?
        .into_iter()
        .find(|d| d.path == config.drive_path || d.mount_point == config.mount_point)
        .ok_or_else(|| format!("Drive not found: {}", config.drive_path))?;

    let drive_name = drive_info.name.clone();
    let claimed_capacity = drive_info.capacity_bytes;

    let mut real_capacity: Option<u64> = None;
    let mut seq_read_speed: Option<f64> = None;
    let mut seq_write_speed: Option<f64> = None;
    let mut random_read_iops: Option<f64> = None;
    let mut random_write_iops: Option<f64> = None;
    let mut speed_stability: Option<f64> = None;
    let mut speed_samples: Vec<SpeedSample> = Vec::new();
    let mut bad_block_count: Option<u64> = None;
    let mut total_blocks: Option<u64> = None;
    let mut bad_block_positions: Vec<u64> = Vec::new();
    let mut speed_drops: Option<u32> = None;

    // Capacity test
    if config.test_capacity {
        let handle = app_handle.clone();
        let mount = config.mount_point.clone();

        let result = tokio::task::spawn_blocking(move || {
            crate::test::capacity::run_capacity_test(&mount, claimed_capacity, |progress, msg, spd| {
                let _ = handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "capacity".to_string(),
                        phase: if progress < 0.5 {
                            "writing".to_string()
                        } else {
                            "verifying".to_string()
                        },
                        progress,
                        current_speed: spd,
                        message: msg.to_string(),
                        error: None,
                    },
                );
            })
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok(cap_result) => {
                real_capacity = Some(cap_result.real_bytes);
                total_blocks = Some(cap_result.total_blocks);
                bad_block_positions.extend(cap_result.bad_blocks.iter());
            }
            Err(e) => {
                let _ = app_handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "capacity".to_string(),
                        phase: "error".to_string(),
                        progress: 0.0,
                        current_speed: 0.0,
                        message: e.clone(),
                        error: Some(e),
                    },
                );
            }
        }
    }

    // Speed test
    if config.test_speed && !STOP_FLAG.load(Ordering::Relaxed) {
        let handle = app_handle.clone();
        let mount = config.mount_point.clone();
        let test_size = config.test_size_mb.unwrap_or(1024);

        let result = tokio::task::spawn_blocking(move || {
            crate::test::speed::run_speed_test(&mount, test_size, |progress, msg, spd| {
                let _ = handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "speed".to_string(),
                        phase: if progress < 0.33 {
                            "writing".to_string()
                        } else if progress < 0.67 {
                            "reading".to_string()
                        } else {
                            "random".to_string()
                        },
                        progress,
                        current_speed: spd,
                        message: msg.to_string(),
                        error: None,
                    },
                );
            })
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok(spd_result) => {
                seq_write_speed = Some(spd_result.seq_write_speed);
                seq_read_speed = Some(spd_result.seq_read_speed);
                random_read_iops = Some(spd_result.random_read_iops);
                random_write_iops = Some(spd_result.random_write_iops);
                speed_stability = Some(spd_result.stability);
                speed_drops = Some(spd_result.speed_drops);

                // Merge write and read samples
                let max_len = spd_result.write_samples.len().max(spd_result.read_samples.len());
                for i in 0..max_len {
                    let ws = spd_result
                        .write_samples
                        .get(i)
                        .map(|s| s.speed_mbps)
                        .unwrap_or(0.0);
                    let rs = spd_result
                        .read_samples
                        .get(i)
                        .map(|s| s.speed_mbps)
                        .unwrap_or(0.0);
                    let offset = spd_result
                        .write_samples
                        .get(i)
                        .or_else(|| spd_result.read_samples.get(i))
                        .map(|s| s.offset_mb)
                        .unwrap_or(0);

                    speed_samples.push(SpeedSample {
                        offset_mb: offset,
                        write_speed: ws,
                        read_speed: rs,
                    });
                }
            }
            Err(e) => {
                let _ = app_handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "speed".to_string(),
                        phase: "error".to_string(),
                        progress: 0.0,
                        current_speed: 0.0,
                        message: e.clone(),
                        error: Some(e),
                    },
                );
            }
        }
    }

    // Bad block test
    if config.test_badblock && !STOP_FLAG.load(Ordering::Relaxed) {
        let handle = app_handle.clone();
        let mount = config.mount_point.clone();

        let result = tokio::task::spawn_blocking(move || {
            crate::test::badblock::run_badblock_test(&mount, |progress, msg, spd| {
                let _ = handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "badblock".to_string(),
                        phase: if progress < 0.5 {
                            "writing".to_string()
                        } else {
                            "verifying".to_string()
                        },
                        progress,
                        current_speed: spd,
                        message: msg.to_string(),
                        error: None,
                    },
                );
            })
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;

        match result {
            Ok(bb_result) => {
                bad_block_count = Some(bb_result.bad_blocks.len() as u64);
                if total_blocks.is_none() {
                    total_blocks = Some(bb_result.total_blocks);
                }
                bad_block_positions.extend(bb_result.bad_blocks.iter());
            }
            Err(e) => {
                let _ = app_handle.emit(
                    "test-progress",
                    TestProgress {
                        test_type: "badblock".to_string(),
                        phase: "error".to_string(),
                        progress: 0.0,
                        current_speed: 0.0,
                        message: e.clone(),
                        error: Some(e),
                    },
                );
            }
        }
    }

    let test_duration_secs = start_time.elapsed().as_secs();

    // Calculate scores
    let score = crate::report::score::calculate_score(
        claimed_capacity,
        real_capacity,
        seq_read_speed,
        seq_write_speed,
        speed_stability,
        speed_drops,
        bad_block_count,
        total_blocks,
    );

    let report_id = uuid::Uuid::new_v4().to_string();
    let test_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Build details JSON (speed samples)
    let details_json = serde_json::to_string(&speed_samples).unwrap_or_else(|_| "[]".to_string());

    // Save to DB
    let report_detail = crate::commands::report::ReportDetail {
        id: report_id.clone(),
        drive_name: drive_name.clone(),
        drive_serial: drive_info.serial.clone(),
        claimed_capacity_bytes: claimed_capacity,
        test_date,
        total_score: score.total,
        capacity_score: score.capacity_score,
        speed_score: score.speed_score,
        stability_score: score.stability_score,
        badblock_score: score.badblock_score,
        real_capacity_bytes: real_capacity,
        seq_read_speed,
        seq_write_speed,
        random_read_iops,
        random_write_iops,
        speed_stability,
        bad_block_count,
        total_blocks,
        test_duration_secs: Some(test_duration_secs),
        details_json: Some(details_json),
    };

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.save_report(&report_detail)?;
    }

    let test_result = TestResult {
        report_id,
        drive_name,
        claimed_capacity,
        real_capacity,
        seq_read_speed,
        seq_write_speed,
        random_read_iops,
        random_write_iops,
        speed_stability,
        speed_samples,
        bad_block_count,
        total_blocks,
        bad_block_positions,
        total_score: score.total,
        capacity_score: score.capacity_score,
        speed_score: score.speed_score,
        stability_score: score.stability_score,
        badblock_score: score.badblock_score,
        test_duration_secs,
    };

    let _ = app_handle.emit("test-complete", test_result.clone());

    Ok(test_result)
}

#[tauri::command]
pub fn stop_test() -> Result<(), String> {
    STOP_FLAG.store(true, Ordering::Relaxed);
    Ok(())
}
