# 🔍 Scouty

**A fast, extensible CLI TUI log viewer built in Rust.**

Scouty helps developers and SREs browse, parse, filter, and analyze logs from multiple sources — all within the terminal.

![Apache-2.0 License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)

## ✨ Features

### 📂 Multi-Source Log Loading
- **Local files** — Plain text log files
- **Archives** — gz, zip, 7z compressed logs
- **Stdin/pipe input** — `cat log | scouty-tui` with auto follow mode
- **Multi-file loading** — Multiple files merged by timestamp (merge-sort)
- **Default syslog** — Opens `/var/log/syslog` on Linux when no file specified
- **Configurable default paths** — `default_paths` in `~/.scouty/config.yaml` with glob support
- **Live syslog** — Real-time syslog stream
- **OpenTelemetry (OTLP)** — Receive logs via gRPC and HTTP

Each source gets its own loader, and a single session can combine multiple loaders for unified viewing.

### 🔤 Smart Parsing with Auto-Detection
- **Regex-based parsers** — Fully customizable via YAML configuration
- **Parser groups** — Multiple parsers per source with automatic fallback
- **Auto-detection** — Parser factory selects the right parser based on log content
- **Unified Syslog Parser** — Hand-written zero-regex parser supporting 3 syslog formats:
  - **BSD syslog** — `Nov 24 17:56:03 hostname process[pid]: message`
  - **Extended syslog** — `2025 Nov 24 17:56:03.073872 hostname LEVEL container#process[pid]: message`
  - **ISO 8601 syslog** — `2025-11-24T17:56:03.073872-08:00 hostname process[pid]: message`
- **SONiC SWSS Parser** — `2025-11-13.22:19:35.512358|TABLE:Key|SET|key:value|...`
- **Sairedis Parser** — `2025-05-18.06:38:35.610696|c|SAI_OBJECT_TYPE_SWITCH|...` (4.9M rec/sec)
- **Multi-line merging** — Handles stack traces and multi-line logs (configurable per loader)
- **Parallel parsing** — Rayon-based thread pool for maximum throughput

### 🔎 Powerful Filtering
- **Expression-based** — `level = "Error" AND component contains "auth"`
- **Operators** — `=`, `!=`, `>`, `>=`, `<`, `<=`, `contains`, `starts_with`, `ends_with`, `regex`
- **Logic** — `AND`, `OR`, `NOT` with parentheses for grouping
- **Include/Exclude** — Exclude-first, then include. No include filters = include all
- **Any field** — Filter on timestamp, level, pid, tid, component, hostname, container, context, function, message, or custom metadata

### 🖥️ Interactive TUI
- **Table view** with configurable columns and auto-width (default: Time + Log; toggle more via `c`)
- **Column separators** — vertical lines between columns for clarity
- **Level colors** — FATAL red bold / ERROR red / WARN yellow / INFO green / DEBUG gray / TRACE dark gray / NOTICE cyan
- **Detail panel** — Split-pane view: left 70% log content, right 30% structured fields table
- **2-line status bar** — Line 1: density chart + position; Line 2: mode + shortcuts/input/status
- **Regex search** with match highlighting and navigation
- **Density graph** — Braille-character time distribution in status bar with `[█=Xs]` time-per-column label
- **Filter dialogs** — Quick exclude/include, field-based multi-select, filter manager, cursor preserved after filter
- **Custom highlight** — `h` to add highlight rule, `H` for highlight manager, full-row background coloring, auto color rotation
- **Time jump** — `]` jump forward / `[` jump backward by relative time (5m, 30s, 2h)
- **Stats overlay** — `S` shows log level distribution, top components
- **Save/export** — `Ctrl+s` quick export filtered logs to auto-named file; `:w <filename>` for custom filename
- **Pipe input** — `cat log | scouty-tui` with auto follow mode
- **Copy to clipboard** — Raw, JSON, or YAML format via OSC 52
- **Component architecture** — Unified `UiComponent` trait with standardized keyboard dispatch

### 🚀 High Performance
- **10M+ records/sec** parsing with hand-written zero-regex syslog parser
- **Segmented sorted array** log store with O(N) merge-sort insertion
- **Zero-copy filtering** via `Arc<LogRecord>` shared between store and background filter threads
- **Async background filtering** with dual-buffer swap for non-blocking UI
- **Parallel parsing** — Rayon-based thread pool for maximum throughput

## 📦 Crates

| Crate | Description |
|-------|-------------|
| `scouty` | Core library — log records, loaders, parsers, store, filters, views, session management |
| `scouty-tui` | Terminal UI — interactive log browsing powered by `scouty` |

## 🚀 Quick Start

### Install from source

```bash
git clone https://github.com/r12f/scouty.git
cd scouty
cargo build --release
```

### View a log file

```bash
./target/release/scouty-tui /path/to/your.log
```

### View compressed logs

```bash
./target/release/scouty-tui /path/to/logs.gz
```

### View multiple files

```bash
./target/release/scouty-tui /var/log/syslog /var/log/auth.log
```

### Install globally (optional)

```bash
cargo install --path crates/scouty-tui
# Then use directly:
scouty-tui /path/to/your.log
```

### Pipe input

```bash
cat /var/log/syslog | scouty-tui
journalctl -f | scouty-tui
```

## ⌨️ Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down one line |
| `k` / `↑` | Move up one line |
| `PageDown` / `Ctrl+j` / `Ctrl+↓` | Page down |
| `PageUp` / `Ctrl+k` / `Ctrl+↑` | Page up |
| `g` | Jump to first line |
| `G` | Jump to last line |
| `Ctrl+G` | Go to line number |
| `]` | Time jump forward (e.g. 5m, 30s, 2h) |
| `[` | Time jump backward |

### Search & Filter

| Key | Action |
|-----|--------|
| `/` | Search (regex supported) |
| `n` | Next search match |
| `N` | Previous search match |
| `f` | Filter expression input |
| `-` | Quick exclude (text input) |
| `=` | Quick include (text input) |
| `_` / `Ctrl+-` | Exclude field dialog (multi-select from current row) |
| `+` / `Ctrl+=` | Include field dialog (multi-select from current row) |
| `F` | Filter manager |

### Display

| Key | Action |
|-----|--------|
| `Enter` | Toggle detail panel |
| `c` | Column selector (toggle columns) |
| `S` | Stats overlay (level distribution, top components) |

### Highlight

| Key | Action |
|-----|--------|
| `h` | Add highlight rule |
| `H` | Highlight manager |

### Copy & Export

| Key | Action |
|-----|--------|
| `y` | Copy selected row (raw text) |
| `Y` | Copy with format dialog (Raw/JSON/YAML) |
| `Ctrl+s` | Save/export current view to file |

### General

| Key | Action |
|-----|--------|
| `Ctrl+]` | Toggle follow mode |
| `?` | Help |
| `q` | Quit |
| `Esc` | Close current dialog/panel |

### Dialog Navigation (universal)

All dialogs and windows share these standard controls:

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Move selection |
| `PageUp` / `PageDown` | Page through options |
| `Space` | Toggle selection |
| `Enter` | Confirm |
| `Esc` | Cancel |

## 📊 Log Record Fields

Scouty parses logs into structured records with these first-class fields:

| Field | Struct Name | Filter Alias | Description | Example |
|-------|-------------|--------------|-------------|---------|
| Timestamp | `timestamp` | `timestamp` | Log timestamp | `2025-11-24T17:56:03.073872` |
| Level | `level` | `level` | Severity level | `INFO`, `ERROR`, `NOTICE` |
| Hostname | `hostname` | `hostname` | Source host | `BSL-0101-0101-01LT0` |
| Process | `process_name` | `process` | Process name | `dockerd`, `root` |
| PID | `pid` | `pid` | Process ID | `871` |
| TID | `tid` | `tid` | Thread ID | `12345` |
| Component | `component_name` | `component` | Component/module | `SWITCH_TABLE`, `squid` |
| Container | `container` | `container` | Container name | `restapi`, `pmon` |
| Context | `context` | `context` | Contextual key | `Ethernet248`, `fd00::/80` |
| Function | `function` | `function` | Operation/function | `SET`, `DEL` |
| Message | `message` | `message` | Log message body | *(the log text)* |
| Source | `source` | `source` | Source identifier (file path, syslog source, etc.) | `/var/log/syslog` |
| Raw | `raw` | `raw` | Original raw log line | *(full unparsed line)* |
| Loader ID | `loader_id` | — | Identifier of the loader that produced this record | `file:///var/log/syslog` |
| Metadata | `metadata` | *(by key name)* | Additional key-value pairs | `hostname=myhost` |

## ⚙️ Parser Configuration

Parsers are configured via YAML files. A single file can define multiple parser groups:

```yaml
parser_groups:
  - name: syslog
    multiline: true
    patterns:
      - name: rfc3164
        regex: '^(?P<timestamp>\w{3}\s+\d+\s+\d+:\d+:\d+)\s+(?P<source>\S+)\s+(?P<process_name>\S+?)(\[(?P<pid>\d+)\])?\s*:\s*(?P<message>.*)'

  - name: generic
    multiline: false
    patterns:
      - name: timestamp_level
        regex: '^\[(?P<timestamp>[^\]]+)\]\s*\[(?P<level>\w+)\]\s*(?P<message>.*)'
```

Each parser group tries its patterns in order — if the first fails, it falls back to the next.

## 🔎 Filter Expressions

```bash
# Simple field comparison
level = "Error"

# Range query
timestamp >= "2024-01-01T00:00:00Z" AND timestamp < "2024-01-02T00:00:00Z"

# Compound with parentheses
(level = "Error" OR level = "Fatal") AND component contains "database"

# String matching
message regex "timeout.*retry"

# Filter by hostname or container
hostname = "BSL-0101-0101-01LT0" AND container = "restapi"
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Log Session                        │
│                                                     │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐        │
│  │ Loader 1 │   │ Loader 2 │   │ Loader N │        │
│  │(text file)│   │  (gz)    │   │  (otlp)  │        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘        │
│       │               │               │              │
│  ┌────▼─────┐   ┌────▼─────┐   ┌────▼─────┐        │
│  │ Parser   │   │ Parser   │   │ Parser   │        │
│  │ Group 1  │   │ Group 2  │   │ Group N  │        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘        │
│       │               │               │              │
│       └───────────────┼───────────────┘              │
│                       ▼                              │
│              ┌────────────────┐                      │
│              │   Log Store    │ (timestamp-sorted)    │
│              │  Segmented     │ (~64K-128K/segment)   │
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │  Processors    │ (extensible pipeline) │
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │ Filter Engine  │ (exclude → include)   │
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │ LogStore View  │ (async dual-buffer    │
│              │                │  background filtering)│
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │  TUI / Output  │                      │
│              └────────────────┘                      │
└─────────────────────────────────────────────────────┘
```

### Supported Log Formats (Auto-Detected)

| Format | Example |
|--------|---------|
| BSD Syslog | `Nov 24 17:56:03 myhost sshd[1234]: Accepted publickey` |
| Extended Syslog | `2025 Nov 24 17:56:03.073872 myhost INFO docker#nginx[42]: GET /` |
| ISO 8601 Syslog | `2025-11-24T17:56:03.073872-08:00 myhost cron[99]: running job` |
| SONiC SWSS | `2025-11-13.22:19:35.512358\|PORT_TABLE:Ethernet248\|SET\|admin_status:up` |
| Sairedis | `2025-05-18.06:38:35.610696\|c\|SAI_OBJECT_TYPE_SWITCH\|...` |
| Custom Regex | *(user-defined via YAML)* |

## 📄 License

Apache-2.0
