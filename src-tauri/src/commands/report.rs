use crate::AppState;
use serde::Serialize;
use tauri::State;

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

#[tauri::command]
pub fn get_report(id: String, state: State<AppState>) -> Result<ReportDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_report(&id)
}

#[tauri::command]
pub fn list_reports(state: State<AppState>) -> Result<Vec<ReportSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_reports()
}

#[tauri::command]
pub fn export_html(id: String, state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let report = db.get_report(&id)?;

    let speed_samples = report.details_json.as_deref().unwrap_or("[]");

    Ok(crate::report::html::generate_html_report(
        &report.drive_name,
        &report.test_date,
        report.claimed_capacity_bytes as f64 / 1_000_000_000.0,
        report.real_capacity_bytes.map(|v| v as f64 / 1_000_000_000.0),
        report.seq_read_speed,
        report.seq_write_speed,
        report.random_read_iops,
        report.random_write_iops,
        report.speed_stability,
        report.bad_block_count.unwrap_or(0),
        report.total_blocks.unwrap_or(0),
        report.total_score,
        speed_samples,
        "[]",
    ))
}

#[tauri::command]
pub fn delete_report(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_report(&id)
}
