# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UDisk Inspector — a cross-platform (Windows + macOS) USB flash drive quality testing CLI tool built in pure Rust. Tests for fake capacity, read/write speed, bad blocks, and thermal/stability risk. Designed for AI CLI tool invocation (e.g. Claude Code) with `--json` output mode.

## Build & Run Commands

```bash
cargo check          # Fast type check
cargo build          # Debug build
cargo build --release

# Usage
udisk-inspector list                           # List USB devices
udisk-inspector list --json                    # JSON output for AI tools
udisk-inspector info E:                        # Device details
udisk-inspector test E:\ --all --json          # Run all tests, JSON output
udisk-inspector test E:\ --speed --capacity    # Selective tests
udisk-inspector report list                    # History
udisk-inspector report show <ID>               # Report details
udisk-inspector report export <ID> --html out.html
udisk-inspector report delete <ID>
```

## Architecture

Pure Rust CLI (clap + tokio), no GUI.

### Source Layout (`src/`)

- **`main.rs`** — CLI entry: tokio runtime, Ctrl+C handler, error formatting
- **`lib.rs`** — Module declarations + `STOP_FLAG: AtomicBool` for test cancellation
- **`types.rs`** — Shared types: `ReportDetail`, `ReportSummary`, `TestResult`, `SpeedSample`
- **`cli/mod.rs`** — clap command definitions + all subcommand implementations
- **`cli/output.rs`** — JSON envelope (`{"success":true/false, "data":..., "error":...}`) + human output formatting
- **`disk/`** — Platform-specific device detection
  - `detect.rs` — Windows: wmic + WMI (wrapped in `catch_unwind`); macOS: `diskutil` plist parsing
  - `info.rs` — Formatting helpers (`format_capacity`, `drive_summary`)
  - `io.rs` — Block-level file I/O for tests
- **`test/`** — Core test algorithms, all file-based (write test files to mount point, not raw device)
  - `common.rs` — XorShift64 PRNG, block generation/verification, `should_stop()` check
  - `capacity.rs` — Write numbered 1MB blocks -> flush -> read back to detect fake capacity
  - `speed.rs` — Sequential R/W (32MB chunks) + random 4K IOPS + stability analysis
  - `badblock.rs` — Full write+verify scan
  - `thermal.rs` — Sustained write stress test measuring speed degradation
- **`report/`** — Scoring (capacity 35 + speed 25 + stability 15 + badblock 25 = 100) and HTML report generation with embedded ECharts
- **`db.rs`** — SQLite via rusqlite (bundled), stores test reports in `udisk_reports.db`

## Key Design Decisions

- **`--json` mode**: stdout outputs only `{"success":true/false, "data":..., "error":...}`, progress goes to stderr. AI tools parse stdout only.
- **Ctrl+C**: Sets `STOP_FLAG` to gracefully stop running tests, cleanup test files, return partial results
- **`--db PATH`**: Custom database location (default: `udisk_reports.db` in CWD)
- Crate name is `udisk_inspector_lib`

## Critical Gotchas

- **WMI is unreliable** — COM initialization can panic; always wrap in `catch_unwind` and fall back to wmic
- **Windows paths with Chinese characters** — always quote paths
- **All disk I/O runs in `spawn_blocking`** to keep tokio runtime responsive
