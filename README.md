# UDisk Inspector

**USB flash drive quality testing CLI tool — detect fake capacity drives, benchmark speed, scan for bad blocks.**

**U盘质量检测命令行工具 — 检测扩容假盘、测速、坏块扫描、热稳定性测试。**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-blue)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()

---

## Why / 为什么需要这个工具

Fake USB drives are everywhere. A drive labeled "64GB" might only have 8GB of real storage — the firmware is hacked to report a false capacity. Files stored beyond the real limit get silently corrupted. **You won't know until it's too late.**

市面上充斥着扩容假U盘。一个标称 64GB 的盘可能实际只有 8GB，固件被篡改显示虚假容量。超出真实容量的文件会被静默损坏，**你打开文件之前根本不会发现**。

UDisk Inspector writes numbered data blocks, reads them back, and verifies every byte — the only reliable way to detect fake capacity. It also benchmarks sequential/random speed and scans for bad blocks.

UDisk Inspector 通过写入编号数据块、回读验证每个字节来检测真实容量，同时测试顺序/随机读写速度和坏块扫描。

## Features / 功能

- **Fake capacity detection / 扩容检测** — Write-verify every block to find the real usable capacity
- **Speed benchmark / 速度测试** — Sequential read/write (32MB chunks) + random 4K IOPS + stability analysis
- **Bad block scan / 坏块扫描** — Full write-read-verify pass across all free space
- **Thermal stress test / 热稳定性** — Sustained write to measure speed degradation over time
- **Scoring system / 评分系统** — 100-point quality score (capacity 35 + speed 25 + stability 15 + badblock 25)
- **HTML reports / HTML 报告** — Standalone report with embedded ECharts graphs
- **JSON output / JSON 输出** — Machine-readable output for AI tools and scripts (`--json`)
- **Cross-platform / 跨平台** — Windows + macOS

## Install / 安装

### Build from source / 从源码编译

```bash
git clone https://github.com/dongsheng123132/udisk-inspector.git
cd udisk-inspector
cargo build --release
# Binary at: target/release/udisk-inspector (4.8MB)
```

Requires Rust 1.70+ and a C compiler (for bundled SQLite).

## Quick Start / 快速上手

```bash
# List USB drives / 列出U盘
udisk-inspector list

# JSON output (for AI tools / scripts)
udisk-inspector list --json

# Run all tests / 运行所有测试
udisk-inspector test E: --all

# Speed test only (256MB) / 仅速度测试
udisk-inspector test E: --speed --test-size-mb 256

# Capacity verification only / 仅容量验证
udisk-inspector test E: --capacity

# Export HTML report / 导出HTML报告
udisk-inspector test E: --all --export-html report.html

# View history / 查看历史报告
udisk-inspector report list
udisk-inspector report show <ID>
```

## Usage / 详细用法

```
udisk-inspector [OPTIONS] <COMMAND>

Commands:
  list                    List USB devices / 列出USB设备
  info <DRIVE>            Drive details / 设备详情
  test <MOUNT> [OPTIONS]  Run tests / 运行测试
  report <ACTION>         Report management / 报告管理

Global Options:
  --json          JSON output to stdout (progress → stderr)
  --db <PATH>     Database path [default: udisk_reports.db]
  -v, --verbose   Verbose logging
  -h, --help      Print help
  -V, --version   Print version

Test Options:
  --capacity              Capacity verification / 容量验证
  --speed                 Speed benchmark / 速度测试
  --badblock              Bad block scan / 坏块扫描
  --thermal <SECS>        Thermal stress test / 热稳定性测试
  --all                   Run all tests / 运行所有测试
  --test-size-mb <MB>     Speed test size [default: 1024]
  --no-save               Don't save to database / 不保存到数据库
  --export-html <PATH>    Export HTML report / 导出HTML报告

Report Actions:
  report list             List reports / 列出报告
  report show <ID>        Show report / 查看报告
  report export <ID> --html <PATH>  Export HTML / 导出HTML
  report delete <ID>      Delete report / 删除报告
```

## How It Works / 工作原理

### Fake Capacity Detection / 扩容检测

```
1. Write: Generate 1MB blocks with embedded block number + pseudo-random data
          写入：生成包含块编号 + 伪随机数据的 1MB 块

2. Sync:  Flush each block to disk (bypass write cache)
          同步：每个块强制刷盘

3. Verify: Read back and check block number + data integrity
           验证：回读检查块编号 + 数据完整性

If block N reads back as block M → fake address remapping detected
如果块 N 读回来变成了块 M → 检测到虚假地址映射
```

### Speed Test / 速度测试

| Test | Method | Metric |
|------|--------|--------|
| Sequential Write | 32MB chunks, sync after each | MB/s |
| Sequential Read | 32MB chunks | MB/s |
| Random Write | 1000x 4K blocks at random offsets | IOPS |
| Random Read | 1000x 4K blocks at random offsets | IOPS |
| Stability | Coefficient of variation + drop detection | 0-100% |

### Scoring / 评分标准

| Category | Points | Criteria |
|----------|--------|----------|
| Capacity | 35 | Real/claimed ratio (>95% = full score) |
| Speed | 25 | Seq read (12pts, vs 100MB/s) + Seq write (13pts, vs 50MB/s) |
| Stability | 15 | Speed consistency (10pts) + drop count (5pts) |
| Bad Blocks | 25 | Bad block ratio (0% = full score) |

## Real-World Test Results / 实测结果

Tested 6 USB drives simultaneously — **found one fake drive**:

实测 6 个 U 盘 — **发现一个扩容假盘**：

| Drive | Capacity | Seq Write | Seq Read | Stability | Verdict |
|-------|----------|-----------|----------|-----------|---------|
| Drive A | 64 GB | 22.3 MB/s | 4131 MB/s* | Good | Best write speed |
| Drive B | 32 GB | 20.3 MB/s | 3633 MB/s* | 89% | Balanced |
| Drive C | 16 GB | 18.8 MB/s | 3907 MB/s* | **91%** | Most reliable |
| Drive D | 64 GB | 8.5 MB/s | 4168 MB/s* | 85% | Slow write |
| Drive E | 64 GB | 4.1 MB/s | 1645 MB/s* | 81% | Slowest |
| **Drive F** | **~~32 GB~~ → 8.5 GB** | **8.2 MB/s** | 2881 MB/s* | **53%** | **FAKE!** |

*\* Read speeds reflect OS cache hits, not actual media speed.*

*\* 读取速度反映的是系统缓存命中，非真实介质速度。*

## AI Integration / AI 集成

Designed as a local API for AI agents (Claude Code, etc.):

专为 AI 工具设计的本地 API（Claude Code 等）：

```bash
# JSON envelope: {"success": true/false, "data": ..., "error": ...}
# stdout = JSON only, stderr = progress/logs

# Example: AI agent checks USB drives
result=$(udisk-inspector list --json)
echo "$result" | jq '.data[] | {name, path, capacity: (.capacity_bytes/1e9)}'

# Example: AI agent runs test and parses result
udisk-inspector test E: --speed --json 2>/dev/null | jq '.data.seq_write_speed'
```

## Architecture / 架构

```
src/
├── main.rs          # Entry: tokio runtime + Ctrl+C handler
├── lib.rs           # Module declarations + STOP_FLAG
├── types.rs         # Shared types (ReportDetail, TestResult, SpeedSample)
├── cli/
│   ├── mod.rs       # Clap command definitions + subcommand handlers
│   └── output.rs    # JSON envelope + Human/JSON dual-mode output
├── disk/
│   ├── detect.rs    # Windows: wmic+WMI / macOS: diskutil
│   ├── info.rs      # Formatting helpers
│   └── io.rs        # Block-level file I/O
├── test/
│   ├── common.rs    # XorShift64 PRNG, block gen/verify, should_stop()
│   ├── capacity.rs  # Write-verify capacity test
│   ├── speed.rs     # Sequential + random I/O benchmark
│   ├── badblock.rs  # Full write-verify scan
│   └── thermal.rs   # Sustained write stress test
├── report/
│   ├── score.rs     # 100-point quality scoring
│   └── html.rs      # Standalone HTML report with ECharts
└── db.rs            # SQLite (bundled rusqlite)
```

## FAQ

**Q: Will this destroy my data? / 会破坏数据吗？**

Tests write temporary files to a `_udisk_test_` directory on the drive. They are automatically cleaned up. However, the capacity test fills all free space temporarily. **Back up important data first.**

测试会在U盘上创建临时文件夹 `_udisk_test_`，测完自动删除。但容量测试会临时占满所有可用空间。**请先备份重要数据。**

**Q: Why are read speeds so high (2000+ MB/s)? / 为什么读取速度这么高？**

OS file cache. The data was just written, so reads hit memory cache instead of actual USB media. Write speeds are the reliable metric.

操作系统文件缓存。数据刚写入，读取命中内存缓存而非真实USB介质。写入速度才是可靠的指标。

**Q: Can I test multiple drives at once? / 能同时测多个盘吗？**

Yes, run multiple instances in parallel:

可以，并行运行多个实例：

```bash
udisk-inspector test E: --speed --json &
udisk-inspector test F: --speed --json &
wait
```

## License

MIT

## Contributing

Issues and PRs welcome. Built with Rust, uses clap for CLI, rusqlite for storage, indicatif for progress bars.
