pub mod output;

use crate::db::Database;
use crate::disk::{detect, info};
use crate::types::*;
use crate::STOP_FLAG;
use clap::{Parser, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use indicatif::{ProgressBar, ProgressStyle};
use output::{print_error, print_success, OutputMode};
use std::sync::atomic::Ordering;

#[derive(Parser)]
#[command(name = "udisk-inspector", version, about = "U盘质量检测 CLI 工具")]
pub struct Cli {
    /// 以 JSON 格式输出（适合 AI/脚本调用）
    #[arg(long, global = true)]
    json: bool,

    /// 数据库路径
    #[arg(long, global = true, default_value = "udisk_reports.db")]
    db: String,

    /// 详细日志
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出 USB 设备
    List,
    /// 设备详情
    Info {
        /// 驱动器路径（如 E: 或 /dev/disk2）
        drive: String,
    },
    /// 运行测试
    Test {
        /// 挂载点（如 E:\ 或 /Volumes/USB）
        mount: String,
        /// 容量测试
        #[arg(long)]
        capacity: bool,
        /// 速度测试
        #[arg(long)]
        speed: bool,
        /// 坏块测试
        #[arg(long)]
        badblock: bool,
        /// 热稳定性测试（指定持续秒数）
        #[arg(long)]
        thermal: Option<u64>,
        /// 运行所有测试
        #[arg(long)]
        all: bool,
        /// 不保存到数据库
        #[arg(long)]
        no_save: bool,
        /// 导出 HTML 报告
        #[arg(long)]
        export_html: Option<String>,
        /// 速度测试大小（MB）
        #[arg(long, default_value = "1024")]
        test_size_mb: u64,
    },
    /// 报告管理
    Report {
        #[command(subcommand)]
        action: ReportAction,
    },
}

#[derive(Subcommand)]
enum ReportAction {
    /// 列出历史报告
    List,
    /// 查看报告详情
    Show {
        /// 报告 ID
        id: String,
    },
    /// 导出 HTML 报告
    Export {
        /// 报告 ID
        id: String,
        /// 输出 HTML 文件路径
        #[arg(long)]
        html: String,
    },
    /// 删除报告
    Delete {
        /// 报告 ID
        id: String,
    },
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mode = if cli.json {
        OutputMode::Json
    } else {
        OutputMode::Human
    };

    match cli.command {
        Commands::List => cmd_list(mode).await,
        Commands::Info { drive } => cmd_info(mode, &drive).await,
        Commands::Test {
            mount,
            capacity,
            speed,
            badblock,
            thermal,
            all,
            no_save,
            export_html,
            test_size_mb,
        } => {
            cmd_test(
                mode,
                &cli.db,
                &mount,
                capacity || all,
                speed || all,
                badblock || all,
                if all { Some(120) } else { thermal },
                no_save,
                export_html.as_deref(),
                test_size_mb,
            )
            .await
        }
        Commands::Report { action } => match action {
            ReportAction::List => cmd_report_list(mode, &cli.db),
            ReportAction::Show { id } => cmd_report_show(mode, &cli.db, &id),
            ReportAction::Export { id, html } => cmd_report_export(mode, &cli.db, &id, &html),
            ReportAction::Delete { id } => cmd_report_delete(mode, &cli.db, &id),
        },
    }
}

async fn cmd_list(mode: OutputMode) -> Result<(), Box<dyn std::error::Error>> {
    let drives = tokio::task::spawn_blocking(detect::list_usb_drives)
        .await?
        .map_err(|e| e)?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &drives);
        }
        OutputMode::Human => {
            if drives.is_empty() {
                println!("No USB drives found.");
                return Ok(());
            }
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec![
                "Name", "Path", "Mount", "Capacity", "Free", "FS", "Serial",
            ]);
            for d in &drives {
                table.add_row(vec![
                    &d.name,
                    &d.path,
                    &d.mount_point,
                    &info::format_capacity(d.capacity_bytes),
                    &info::format_capacity(d.free_bytes),
                    &d.file_system,
                    &d.serial,
                ]);
            }
            println!("{table}");
        }
    }
    Ok(())
}

async fn cmd_info(mode: OutputMode, drive: &str) -> Result<(), Box<dyn std::error::Error>> {
    let drive_str = drive.to_string();
    let drives = tokio::task::spawn_blocking(detect::list_usb_drives)
        .await?
        .map_err(|e| e)?;

    let found = drives
        .into_iter()
        .find(|d| d.path == drive_str || d.mount_point == drive_str)
        .ok_or_else(|| format!("Drive not found: {}", drive_str))?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &found);
        }
        OutputMode::Human => {
            println!("{}", info::drive_summary(&found));
            println!("  Path:       {}", found.path);
            println!("  Mount:      {}", found.mount_point);
            println!(
                "  Capacity:   {}",
                info::format_capacity(found.capacity_bytes)
            );
            println!("  Free:       {}", info::format_capacity(found.free_bytes));
            println!("  FS:         {}", found.file_system);
            println!("  Serial:     {}", found.serial);
            println!("  Removable:  {}", found.is_removable);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cmd_test(
    mode: OutputMode,
    db_path: &str,
    mount: &str,
    test_capacity: bool,
    test_speed: bool,
    test_badblock: bool,
    test_thermal: Option<u64>,
    no_save: bool,
    export_html: Option<&str>,
    test_size_mb: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if !test_capacity && !test_speed && !test_badblock && test_thermal.is_none() {
        let msg = "No test selected. Use --capacity, --speed, --badblock, --thermal SECS, or --all";
        print_error(mode, msg);
        return Err(msg.into());
    }

    STOP_FLAG.store(false, Ordering::Relaxed);

    let start_time = std::time::Instant::now();

    // Get drive info
    let mount_str = mount.to_string();
    let drives = tokio::task::spawn_blocking(detect::list_usb_drives)
        .await?
        .map_err(|e| e)?;

    let drive_info = drives
        .into_iter()
        .find(|d| d.path == mount_str || d.mount_point == mount_str)
        .ok_or_else(|| format!("Drive not found at mount point: {}", mount_str))?;

    let drive_name = drive_info.name.clone();
    let claimed_capacity = drive_info.capacity_bytes;
    let mount_point = drive_info.mount_point.clone();

    if let OutputMode::Human = mode {
        println!(
            "Testing: {} ({})",
            drive_name,
            info::format_capacity(claimed_capacity)
        );
    }

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
    if test_capacity {
        let mount_c = mount_point.clone();
        let pb = make_progress_bar(mode, "Capacity test");

        let result = tokio::task::spawn_blocking(move || {
            crate::test::capacity::run_capacity_test(&mount_c, claimed_capacity, |progress, msg, _spd| {
                eprint_progress(&pb, progress, msg);
            })
        })
        .await?;

        match result {
            Ok(cap) => {
                real_capacity = Some(cap.real_bytes);
                total_blocks = Some(cap.total_blocks);
                bad_block_positions.extend(cap.bad_blocks.iter());
                if let OutputMode::Human = mode {
                    eprintln!(
                        "  Capacity: {} real / {} claimed",
                        info::format_capacity(cap.real_bytes),
                        info::format_capacity(claimed_capacity)
                    );
                }
            }
            Err(e) => {
                eprintln!("  Capacity test error: {}", e);
            }
        }
    }

    // Speed test
    if test_speed && !STOP_FLAG.load(Ordering::Relaxed) {
        let mount_c = mount_point.clone();
        let pb = make_progress_bar(mode, "Speed test");

        let result = tokio::task::spawn_blocking(move || {
            crate::test::speed::run_speed_test(&mount_c, test_size_mb, |progress, msg, _spd| {
                eprint_progress(&pb, progress, msg);
            })
        })
        .await?;

        match result {
            Ok(spd) => {
                seq_write_speed = Some(spd.seq_write_speed);
                seq_read_speed = Some(spd.seq_read_speed);
                random_read_iops = Some(spd.random_read_iops);
                random_write_iops = Some(spd.random_write_iops);
                speed_stability = Some(spd.stability);
                speed_drops = Some(spd.speed_drops);

                let max_len = spd.write_samples.len().max(spd.read_samples.len());
                for i in 0..max_len {
                    let ws = spd.write_samples.get(i).map(|s| s.speed_mbps).unwrap_or(0.0);
                    let rs = spd.read_samples.get(i).map(|s| s.speed_mbps).unwrap_or(0.0);
                    let offset = spd
                        .write_samples
                        .get(i)
                        .or_else(|| spd.read_samples.get(i))
                        .map(|s| s.offset_mb)
                        .unwrap_or(0);
                    speed_samples.push(SpeedSample {
                        offset_mb: offset,
                        write_speed: ws,
                        read_speed: rs,
                    });
                }

                if let OutputMode::Human = mode {
                    eprintln!(
                        "  Seq Write: {:.1} MB/s | Seq Read: {:.1} MB/s",
                        spd.seq_write_speed, spd.seq_read_speed
                    );
                    eprintln!(
                        "  4K Write: {:.0} IOPS | 4K Read: {:.0} IOPS",
                        spd.random_write_iops, spd.random_read_iops
                    );
                }
            }
            Err(e) => {
                eprintln!("  Speed test error: {}", e);
            }
        }
    }

    // Bad block test
    if test_badblock && !STOP_FLAG.load(Ordering::Relaxed) {
        let mount_c = mount_point.clone();
        let pb = make_progress_bar(mode, "Bad block test");

        let result = tokio::task::spawn_blocking(move || {
            crate::test::badblock::run_badblock_test(&mount_c, |progress, msg, _spd| {
                eprint_progress(&pb, progress, msg);
            })
        })
        .await?;

        match result {
            Ok(bb) => {
                bad_block_count = Some(bb.bad_blocks.len() as u64);
                if total_blocks.is_none() {
                    total_blocks = Some(bb.total_blocks);
                }
                bad_block_positions.extend(bb.bad_blocks.iter());
                if let OutputMode::Human = mode {
                    eprintln!("  Bad blocks: {} / {}", bb.bad_blocks.len(), bb.total_blocks);
                }
            }
            Err(e) => {
                eprintln!("  Bad block test error: {}", e);
            }
        }
    }

    // Thermal test
    if let Some(secs) = test_thermal {
        if !STOP_FLAG.load(Ordering::Relaxed) {
            let mount_c = mount_point.clone();
            let pb = make_progress_bar(mode, "Thermal test");

            let result = tokio::task::spawn_blocking(move || {
                crate::test::thermal::run_thermal_test(&mount_c, secs, |progress, msg, _spd| {
                    eprint_progress(&pb, progress, msg);
                })
            })
            .await?;

            match result {
                Ok(th) => {
                    if let OutputMode::Human = mode {
                        eprintln!(
                            "  Thermal: {:.1} MB/s -> {:.1} MB/s ({:.0}% degradation, risk: {:?})",
                            th.avg_speed_first_minute,
                            th.avg_speed_last_minute,
                            th.speed_degradation,
                            th.thermal_risk
                        );
                    }
                }
                Err(e) => {
                    eprintln!("  Thermal test error: {}", e);
                }
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
    let details_json = serde_json::to_string(&speed_samples).unwrap_or_else(|_| "[]".to_string());

    // Save to DB
    if !no_save {
        let report_detail = ReportDetail {
            id: report_id.clone(),
            drive_name: drive_name.clone(),
            drive_serial: drive_info.serial.clone(),
            claimed_capacity_bytes: claimed_capacity,
            test_date: test_date.clone(),
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
            details_json: Some(details_json.clone()),
        };

        let db = Database::open(db_path)?;
        db.save_report(&report_detail)?;
    }

    // Export HTML if requested
    if let Some(html_path) = export_html {
        let html = crate::report::html::generate_html_report(
            &drive_name,
            &test_date,
            claimed_capacity as f64 / 1_000_000_000.0,
            real_capacity.map(|v| v as f64 / 1_000_000_000.0),
            seq_read_speed,
            seq_write_speed,
            random_read_iops,
            random_write_iops,
            speed_stability,
            bad_block_count.unwrap_or(0),
            total_blocks.unwrap_or(0),
            score.total,
            &details_json,
            "[]",
        );
        std::fs::write(html_path, &html)?;
        if let OutputMode::Human = mode {
            eprintln!("HTML report saved to: {}", html_path);
        }
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

    match mode {
        OutputMode::Json => {
            print_success(mode, &test_result);
        }
        OutputMode::Human => {
            println!();
            println!("=== Test Results ===");
            println!("Score: {}/100 ({:?})", score.total, score.grade);
            println!(
                "  Capacity: {}/35  Speed: {}/25  Stability: {}/15  BadBlock: {}/25",
                score.capacity_score, score.speed_score, score.stability_score, score.badblock_score
            );
            println!("Duration: {}s", test_duration_secs);
            if !no_save {
                println!("Report ID: {}", test_result.report_id);
            }
        }
    }

    Ok(())
}

fn cmd_report_list(mode: OutputMode, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open(db_path)?;
    let reports = db.list_reports()?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &reports);
        }
        OutputMode::Human => {
            if reports.is_empty() {
                println!("No reports found.");
                return Ok(());
            }
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["ID", "Drive", "Date", "Score"]);
            for r in &reports {
                table.add_row(vec![
                    &r.id[..8],
                    &r.drive_name,
                    &r.test_date,
                    &r.total_score.to_string(),
                ]);
            }
            println!("{table}");
        }
    }
    Ok(())
}

fn cmd_report_show(
    mode: OutputMode,
    db_path: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open(db_path)?;
    let report = db.get_report(id)?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &report);
        }
        OutputMode::Human => {
            println!("Report: {}", report.id);
            println!("Drive:  {} ({})", report.drive_name, report.drive_serial);
            println!("Date:   {}", report.test_date);
            println!("Score:  {}/100", report.total_score);
            println!(
                "  Capacity: {}/35  Speed: {}/25  Stability: {}/15  BadBlock: {}/25",
                report.capacity_score, report.speed_score, report.stability_score, report.badblock_score
            );
            if let Some(real) = report.real_capacity_bytes {
                println!(
                    "Capacity: {} real / {} claimed",
                    info::format_capacity(real),
                    info::format_capacity(report.claimed_capacity_bytes)
                );
            }
            if let Some(r) = report.seq_read_speed {
                println!("Seq Read:  {:.1} MB/s", r);
            }
            if let Some(w) = report.seq_write_speed {
                println!("Seq Write: {:.1} MB/s", w);
            }
            if let Some(ri) = report.random_read_iops {
                println!("4K Read:   {:.0} IOPS", ri);
            }
            if let Some(wi) = report.random_write_iops {
                println!("4K Write:  {:.0} IOPS", wi);
            }
            if let Some(s) = report.speed_stability {
                println!("Stability: {:.0}%", s * 100.0);
            }
            if let Some(bb) = report.bad_block_count {
                println!(
                    "Bad blocks: {} / {}",
                    bb,
                    report.total_blocks.unwrap_or(0)
                );
            }
            if let Some(d) = report.test_duration_secs {
                println!("Duration:  {}s", d);
            }
        }
    }
    Ok(())
}

fn cmd_report_export(
    mode: OutputMode,
    db_path: &str,
    id: &str,
    html_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open(db_path)?;
    let report = db.get_report(id)?;

    let speed_samples = report.details_json.as_deref().unwrap_or("[]");
    let html = crate::report::html::generate_html_report(
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
    );

    std::fs::write(html_path, &html)?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &serde_json::json!({"path": html_path}));
        }
        OutputMode::Human => {
            println!("HTML report exported to: {}", html_path);
        }
    }
    Ok(())
}

fn cmd_report_delete(
    mode: OutputMode,
    db_path: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open(db_path)?;
    db.delete_report(id)?;

    match mode {
        OutputMode::Json => {
            print_success(mode, &serde_json::json!({"deleted": id}));
        }
        OutputMode::Human => {
            println!("Report {} deleted.", id);
        }
    }
    Ok(())
}

fn make_progress_bar(mode: OutputMode, prefix: &str) -> Option<ProgressBar> {
    match mode {
        OutputMode::Human => {
            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{pos}}% {{msg}}",
                        prefix
                    ))
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
            Some(pb)
        }
        OutputMode::Json => None,
    }
}

fn eprint_progress(pb: &Option<ProgressBar>, progress: f64, msg: &str) {
    if let Some(pb) = pb {
        pb.set_position((progress * 100.0) as u64);
        pb.set_message(msg.to_string());
        if progress >= 1.0 {
            pb.finish_and_clear();
        }
    }
}
