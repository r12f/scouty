# 🔍 Scouty

**A fast, extensible CLI TUI log viewer built in Rust.**

Scouty helps developers and SREs browse, parse, filter, and analyze logs from multiple sources — all within the terminal.

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)

## ✨ Features

### 🚀 High Performance
- **10M+ records/sec** parsing with hand-written zero-regex syslog parser
- **Segmented sorted array** log store with O(N) merge-sort insertion
- **Zero-copy filtering** via `Arc<LogRecord>` shared between store and background filter threads
- **Async background filtering** with dual-buffer swap for non-blocking UI
- **Memory-mapped I/O** for fast file loading

### 📂 Multi-Source Log Loading
- **Local files** — Plain text log files
- **Archives** — gz, zip, 7z compressed logs
- **Live syslog** — Real-time syslog stream
- **OpenTelemetry (OTLP)** — Receive logs via gRPC and HTTP

Each source gets its own loader, and a single session can combine multiple loaders for unified viewing.

### 🔤 Smart Parsing with Auto-Detection
- **Unified Syslog Parser** — Hand-written zero-regex parser supporting 3 syslog formats with automatic detection:
  - **BSD syslog** — `Nov 24 17:56:03 hostname process[pid]: message`
  - **Extended syslog** — `2025 Nov 24 17:56:03.073872 hostname LEVEL container#process[pid]: message`
  - **ISO 8601 syslog** — `2025-11-24T17:56:03.073872-08:00 hostname process[pid]: message`
- **SONiC SWSS Parser** — `2025-11-13.22:19:35.512358|TABLE:Key|SET|key:value|...`
- **Regex-based parsers** — Fully customizable via YAML configuration
- **Parser groups** — Multiple parsers per source with automatic fallback
- **Multi-line merging** — Handles stack traces and multi-line logs (configurable per loader)

### 🔎 Powerful Filtering
- **Expression-based** — `level = "Error" AND component contains "auth"`
- **Operators** — `=`, `!=`, `>`, `>=`, `<`, `<=`, `contains`, `starts_with`, `ends_with`, `regex`
- **Logic** — `AND`, `OR`, `NOT` with parentheses for grouping
- **Include/Exclude** — Exclude-first, then include. No include filters = include all
- **Any field** — Filter on timestamp, level, pid, tid, component, hostname, container, context, function, message, or custom metadata

### 🖥️ Interactive TUI
- **Table view** with configurable columns and auto-width
- **Level colors** — FATAL red bold / ERROR red / WARN yellow / INFO green / DEBUG gray / TRACE dark gray / NOTICE cyan
- **Detail panel** — Persistent bottom panel showing all structured fields
- **Regex search** with match highlighting and navigation
- **Density graph** — Braille-character time distribution in status bar
- **Filter dialogs** — Quick exclude/include, field-based multi-select, filter manager
- **Copy to clipboard** — Raw, JSON, or YAML format via OSC 52
- **Component architecture** — Unified `UiComponent` trait with standardized keyboard dispatch

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
scouty-tui /path/to/your.log
```

### View compressed logs

```bash
scouty-tui /path/to/logs.gz
```

### View multiple files

```bash
scouty-tui /var/log/syslog /var/log/auth.log
```

## ⌨️ Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down one line |
| `k` / `↑` | Move up one line |
| `Ctrl+j` / `Ctrl+↓` | Page down |
| `Ctrl+k` / `Ctrl+↑` | Page up |
| `g` | Jump to first line |
| `G` | Jump to last line |
| `Ctrl+G` | Go to line number |
| `Enter` | Toggle detail panel |
| `Ctrl+]` | Toggle follow mode |

### Search & Filter

| Key | Action |
|-----|--------|
| `/` | Search (regex supported) |
| `n` | Next search match |
| `N` | Previous search match |
| `f` | Filter expression input |
| `-` | Quick exclude (text input) |
| `=` | Quick include (text input) |
| `Ctrl+-` | Exclude field dialog (multi-select from current row) |
| `Ctrl+=` | Include field dialog (multi-select from current row) |
| `F` | Filter manager (view/add/delete filters) |

### Display & Copy

| Key | Action |
|-----|--------|
| `c` | Column selector (toggle columns) |
| `y` | Copy selected row (raw text) |
| `Y` | Copy with format dialog (Raw/JSON/YAML) |

### General

| Key | Action |
|-----|--------|
| `Esc` | Close current dialog/panel |
| `q` | Quit |
| `?` | Help |

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

| Field | Description | Example |
|-------|-------------|---------|
| `timestamp` | Log timestamp | `2025-11-24T17:56:03.073872` |
| `level` | Severity level | `INFO`, `ERROR`, `NOTICE` |
| `hostname` | Source host | `BSL-0101-0101-01LT0` |
| `process_name` | Process name | `dockerd`, `root` |
| `pid` | Process ID | `871` |
| `tid` | Thread ID | `12345` |
| `component` | Component/module | `SWITCH_TABLE`, `squid` |
| `container` | Container name | `restapi`, `pmon` |
| `context` | Contextual key | `Ethernet248`, `fd00::/80` |
| `function` | Operation/function | `SET`, `DEL` |
| `message` | Log message body | *(the log text)* |
| `source` | Source file path | `/var/log/syslog` |

Additional fields are stored as metadata key-value pairs and are accessible in filters and the detail panel.

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
| Custom Regex | *(user-defined via YAML)* |

## 📄 License

MIT
