use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub id: String,
    pub drive_name: String,
    pub test_date: String,
    pub total_score: u32,
}

#[derive(Debug, Serialize)]
pub struct ReportDetail {
    pub id: String,
    pub drive_name: String,
    pub drive_serial: String,
    pub claimed_capacity_bytes: u64,
    pub test_date: String,
    pub total_score: u32,
    pub capacity_score: u32,
    pub speed_score: u32,
    pub stability_score: u32,
    pub badblock_score: u32,
    pub real_capacity_bytes: Option<u64>,
    pub seq_read_speed: Option<f64>,
    pub seq_write_speed: Option<f64>,
    pub random_read_iops: Option<f64>,
    pub random_write_iops: Option<f64>,
    pub speed_stability: Option<f64>,
    pub bad_block_count: Option<u64>,
    pub total_blocks: Option<u64>,
    pub test_duration_secs: Option<u64>,
    pub details_json: Option<String>,
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
