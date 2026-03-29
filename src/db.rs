use rusqlite::{params, Connection};
use thiserror::Error;

use crate::types::{ReportDetail, ReportSummary};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Report not found: {0}")]
    NotFound(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, DbError> {
        Self::open("udisk_reports.db")
    }

    pub fn open(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                drive_name TEXT NOT NULL,
                drive_serial TEXT NOT NULL DEFAULT '',
                claimed_capacity_bytes INTEGER NOT NULL DEFAULT 0,
                test_date TEXT NOT NULL,
                total_score INTEGER NOT NULL DEFAULT 0,
                capacity_score INTEGER NOT NULL DEFAULT 0,
                speed_score INTEGER NOT NULL DEFAULT 0,
                stability_score INTEGER NOT NULL DEFAULT 0,
                badblock_score INTEGER NOT NULL DEFAULT 0,
                real_capacity_bytes INTEGER,
                seq_read_speed REAL,
                seq_write_speed REAL,
                random_read_iops REAL,
                random_write_iops REAL,
                speed_stability REAL,
                bad_block_count INTEGER,
                total_blocks INTEGER,
                test_duration_secs INTEGER,
                details_json TEXT
            );",
        )?;
        Ok(())
    }

    pub fn save_report(&self, report: &ReportDetail) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO reports (
                    id, drive_name, drive_serial, claimed_capacity_bytes, test_date,
                    total_score, capacity_score, speed_score, stability_score, badblock_score,
                    real_capacity_bytes, seq_read_speed, seq_write_speed,
                    random_read_iops, random_write_iops, speed_stability,
                    bad_block_count, total_blocks, test_duration_secs, details_json
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                params![
                    report.id,
                    report.drive_name,
                    report.drive_serial,
                    report.claimed_capacity_bytes,
                    report.test_date,
                    report.total_score,
                    report.capacity_score,
                    report.speed_score,
                    report.stability_score,
                    report.badblock_score,
                    report.real_capacity_bytes,
                    report.seq_read_speed,
                    report.seq_write_speed,
                    report.random_read_iops,
                    report.random_write_iops,
                    report.speed_stability,
                    report.bad_block_count.map(|v| v as i64),
                    report.total_blocks.map(|v| v as i64),
                    report.test_duration_secs.map(|v| v as i64),
                    report.details_json,
                ],
            )
            .map_err(|e| format!("Failed to save report: {}", e))?;
        Ok(())
    }

    pub fn get_report(&self, id: &str) -> Result<ReportDetail, String> {
        self.conn
            .query_row(
                "SELECT id, drive_name, drive_serial, claimed_capacity_bytes, test_date,
                        total_score, capacity_score, speed_score, stability_score, badblock_score,
                        real_capacity_bytes, seq_read_speed, seq_write_speed,
                        random_read_iops, random_write_iops, speed_stability,
                        bad_block_count, total_blocks, test_duration_secs, details_json
                 FROM reports WHERE id = ?1",
                params![id],
                |row| {
                    Ok(ReportDetail {
                        id: row.get(0)?,
                        drive_name: row.get(1)?,
                        drive_serial: row.get(2)?,
                        claimed_capacity_bytes: row.get::<_, i64>(3)? as u64,
                        test_date: row.get(4)?,
                        total_score: row.get::<_, i64>(5)? as u32,
                        capacity_score: row.get::<_, i64>(6)? as u32,
                        speed_score: row.get::<_, i64>(7)? as u32,
                        stability_score: row.get::<_, i64>(8)? as u32,
                        badblock_score: row.get::<_, i64>(9)? as u32,
                        real_capacity_bytes: row.get::<_, Option<i64>>(10)?.map(|v| v as u64),
                        seq_read_speed: row.get(11)?,
                        seq_write_speed: row.get(12)?,
                        random_read_iops: row.get(13)?,
                        random_write_iops: row.get(14)?,
                        speed_stability: row.get(15)?,
                        bad_block_count: row.get::<_, Option<i64>>(16)?.map(|v| v as u64),
                        total_blocks: row.get::<_, Option<i64>>(17)?.map(|v| v as u64),
                        test_duration_secs: row.get::<_, Option<i64>>(18)?.map(|v| v as u64),
                        details_json: row.get(19)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => format!("Report not found: {}", id),
                other => format!("Database error: {}", other),
            })
    }

    pub fn list_reports(&self) -> Result<Vec<ReportSummary>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, drive_name, test_date, total_score FROM reports ORDER BY test_date DESC")
            .map_err(|e| format!("Database error: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ReportSummary {
                    id: row.get(0)?,
                    drive_name: row.get(1)?,
                    test_date: row.get(2)?,
                    total_score: row.get::<_, i64>(3)? as u32,
                })
            })
            .map_err(|e| format!("Database error: {}", e))?;

        let mut reports = Vec::new();
        for row in rows {
            reports.push(row.map_err(|e| format!("Database error: {}", e))?);
        }
        Ok(reports)
    }

    pub fn delete_report(&self, id: &str) -> Result<(), String> {
        let affected = self
            .conn
            .execute("DELETE FROM reports WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete report: {}", e))?;

        if affected == 0 {
            Err(format!("Report not found: {}", id))
        } else {
            Ok(())
        }
    }
}
