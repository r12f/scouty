# 🔍 Scouty

**A fast, extensible CLI TUI log viewer built in Rust.**

Scouty helps developers and SREs browse, parse, filter, and analyze logs from multiple sources — all within the terminal.

## ✨ Features

### Multi-Source Log Loading
- **Local files** — Plain text log files
- **Archives** — gz, zip, 7z compressed logs
- **Live syslog** — Real-time syslog stream
- **OpenTelemetry (OTLP)** — Receive logs via gRPC and HTTP

Each source gets its own loader, and a single session can combine multiple loaders for unified viewing.

### Flexible Parsing
- **Regex-based parsers** — Fully customizable via YAML configuration
- **Parser groups** — Multiple parsers per source with automatic fallback
- **Auto-detection** — Parser factory selects the right parser group based on source type and log content
- **Multi-line merging** — Handles stack traces and multi-line logs (configurable per loader)
- **Thread-pooled** — Parsing runs in parallel for maximum throughput

### Powerful Filtering
- **Expression-based** — `level = "Error" AND component contains "auth"`
- **Operators** — `=`, `!=`, `>`, `>=`, `<`, `<=`, `contains`, `starts_with`, `ends_with`, `regex`
- **Logic** — `AND`, `OR`, `NOT` with parentheses for grouping
- **Include/Exclude** — Exclude-first, then include. No include filters = include all
- **Any field** — Filter on timestamp, level, pid, component, message, or custom metadata

### Log Processing Pipeline

```
Load → Parse → Store → Process → Filter → View
```

1. **Load** — Multiple loaders read from different sources in parallel
2. **Parse** — Thread pool parses raw text into structured `LogRecord`s (immutable after parse)
3. **Store** — Records stored in timestamp-sorted order, supporting live inserts
4. **Process** — Extensible post-processing stage (plugin-ready)
5. **Filter** — Filter engine applies exclude/include rules
6. **View** — Filtered results available for TUI display or programmatic access

## 📦 Crates

| Crate | Description |
|-------|-------------|
| `scouty` | Core library — log records, loaders, parsers, store, filters, session management |
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

## ⚙️ Parser Configuration

Parsers are configured via YAML files. A single file can define multiple parser groups:

```yaml
parser_groups:
  - name: syslog
    multiline: true
    patterns:
      - name: rfc3164
        regex: '^(?P<timestamp>\w{3}\s+\d+\s+\d+:\d+:\d+)\s+(?P<source>\S+)\s+(?P<process_name>\S+?)(\[(?P<pid>\d+)\])?\s*:\s*(?P<message>.*)'
      - name: rfc5424
        regex: '^<\d+>\d+\s+(?P<timestamp>\S+)\s+(?P<source>\S+)\s+(?P<process_name>\S+)\s+(?P<pid>\S+)\s+\S+\s+(?P<message>.*)'

  - name: generic
    multiline: false
    patterns:
      - name: timestamp_level
        regex: '^\[(?P<timestamp>[^\]]+)\]\s*\[(?P<level>\w+)\]\s*(?P<message>.*)'
```

Each parser group tries its patterns in order — if the first fails, it falls back to the next.

## 🔎 Filter Expressions

```
# Simple field comparison
level = "Error"

# Range query
timestamp >= "2024-01-01T00:00:00Z" AND timestamp < "2024-01-02T00:00:00Z"

# Compound with parentheses
(level = "Error" OR level = "Fatal") AND component contains "database"

# String matching
message regex "timeout.*retry"
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
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │  Processors    │ (extensible)          │
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │ Filter Engine  │                      │
│              └───────┬────────┘                      │
│                      ▼                               │
│              ┌────────────────┐                      │
│              │ Filtered View  │                      │
│              └────────────────┘                      │
└─────────────────────────────────────────────────────┘
```

## 📄 License

MIT
