# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UDisk Inspector — a cross-platform (Windows + macOS) USB flash drive quality testing tool built with Tauri 2 + React. Tests for fake capacity, read/write speed, bad blocks, and thermal/stability risk. Chinese (中文) UI.

## Build & Run Commands

```bash
# Development (starts both Vite dev server and Rust backend)
npx tauri dev

# Production build (outputs .exe + .msi + .nsis installer)
npx tauri build

# Frontend only
npm run dev          # Vite dev server on :1420
npm run build        # tsc + vite build → dist/
npx tsc --noEmit     # Type check only

# Rust only (from src-tauri/)
cargo check          # Fast type check
cargo build          # Debug build
cargo build --release
```

## Architecture

**Tauri 2 app**: Rust backend (`src-tauri/`) exposes commands via `#[tauri::command]`, React frontend (`src/`) calls them via `@tauri-apps/api` invoke.

### Rust Backend (`src-tauri/src/`)

- **`lib.rs`** — App entry: registers all Tauri commands, initializes SQLite DB via `AppState`
- **`commands/`** — Tauri command handlers (drive detection, test orchestration, report CRUD)
  - `test.rs` contains `STOP_FLAG: AtomicBool` for test cancellation; tests run via `tokio::task::spawn_blocking` and emit progress via Tauri events (`test-progress`, `test-complete`)
- **`disk/`** — Platform-specific device detection
  - `detect.rs` — Windows: wmic (primary) + WMI via `wmi` crate (enhancement, wrapped in `catch_unwind`); macOS: `diskutil` plist parsing
- **`test/`** — Core test algorithms, all file-based (write test files to mount point, not raw device)
  - `common.rs` — XorShift64 PRNG, block generation/verification with embedded block numbers
  - `capacity.rs` — Write numbered 1MB blocks → flush → read back to detect fake capacity mapping
  - `speed.rs` — Sequential R/W (32MB chunks) + random 4K IOPS + stability analysis (CV, drop detection)
  - `badblock.rs` — Full write+verify scan
  - `thermal.rs` — Sustained write stress test measuring speed degradation over time
- **`report/`** — Scoring (capacity 35 + speed 25 + stability 15 + badblock 25 = 100) and standalone HTML report generation with embedded ECharts
- **`db.rs`** — SQLite via rusqlite (bundled), stores test reports in `udisk_reports.db`

### React Frontend (`src/`)

- **Routing**: `HashRouter` (required for Tauri file:// protocol, NOT BrowserRouter)
- **Pages**: Home (drive list), Test (config + progress + results), Report (detail view), History (list)
- **Components**: ScoreGauge, SpeedChart, BadBlockMap, CapacityBar, StabilityIndicator, TestProgress — all use ECharts via `echarts-for-react`
- **Hooks**: `useDrives` (auto-refresh every 5s, suppresses loading flash on auto-refresh), `useTest` (Tauri event listener for progress/completion)
- **`lib/tauri.ts`** — Typed wrappers around `invoke()` calls
- **`lib/types.ts`** — Shared TypeScript types matching Rust serde structs + utility functions

## Critical Gotchas

- **Vite `base: "./"` is required** in `vite.config.ts` — Tauri loads from file:// protocol, absolute `/assets/` paths cause 404
- **All disk I/O commands must be async** — use `tokio::task::spawn_blocking` to avoid blocking Tauri's event loop and causing WebView hangs
- **WMI is unreliable** — COM initialization can panic; always wrap in `catch_unwind` and fall back to wmic command-line
- **Windows paths have Chinese characters** — always quote paths in shell commands
- Rust crate name is `udisk_inspector_lib` (not `udisk-inspector`)
