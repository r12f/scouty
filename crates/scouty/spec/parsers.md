# Parsers

## Overview

Scouty parsers transform raw log lines into `LogRecord` structs. The system supports multiple parser types: a unified hand-written syslog parser (zero-regex), SONiC SWSS parser, sairedis parser, and user-defined regex parsers.


## Design

### Parser Architecture

- **`LogParser` trait** — common interface for all parsers
- **Parser Group** — ordered list of parsers with fallback: first parser fails → try second → all fail → record goes to `failing_parsing_logs`
- **Parser Factory** — auto-detects format from loader info and first few lines, creates appropriate Parser Group
- **Thread pool** parallel parsing; parsed LogRecords are immutable
- **Parser config** stored in local YAML files (one config can define multiple Parser Groups)

### UnifiedSyslogParser (Hand-written, Zero-regex)

Replaces 4 legacy parsers (BSD hand-written, BSD regex, Extended hand-written, ISO regex) with one unified parser.

**Auto-detection by first byte:**

| First bytes | Format | Example |
|---|---|---|
| `A-Z` (3-letter month) | BSD syslog | `Nov 24 17:56:03 hostname process[pid]: msg` |
| `0-9{4} A-Z` (year+month) | Extended syslog | `2025 Nov 24 17:56:03.073872 hostname LEVEL container#process[pid]: msg` |
| `0-9{4}-` + `T` at pos 10 | ISO 8601 syslog | `2025-11-24T17:56:03.073872-08:00 hostname process[pid]: msg` |

**Shared parsing modules:**
- `parse_hostname(bytes, offset)` → hostname + new offset
- `parse_process_part(bytes, offset)` → container (optional) + process + pid (optional)
- `parse_message(bytes, offset)` → message after `: `

**PROCESS_PART syntax:** `container#process_name[pid]:`
- `#` separates container and process
- `[N]` extracts pid
- Either part may be absent

**Key decisions:**
- BSD syslog year inference uses file modification time
- Distinguishes from SWSS by checking `.` vs `T` at position 10 in the timestamp

### SONiC SWSS Log Parser

Parses `|`-delimited SWSS logs: `YYYY-MM-DD.HH:MM:SS.ffffff|<content...>`

**Field mapping:**

| Parsed | LogRecord field |
|--------|----------------|
| Timestamp | `timestamp` |
| TABLE name | `component` |
| Key | `context` |
| OP (SET/DEL) | `function` |
| KV pairs | `message` |

**Parsing logic:** Split by `|`, locate SET/DEL position, determine TABLE:Key vs TABLE|SubKey format. Key may contain `:` (e.g., IPv6), so only split at first `:`.

### SONiC Sairedis Log Parser

Parses SAI Redis operation logs: `YYYY-MM-DD.HH:MM:SS.ffffff|<op>|<detail...>`

**13 op codes:** `c` (Create), `r` (Remove), `s` (Set), `g` (Get), `G` (GetResponse), `p` (CounterPoll), `C`/`R`/`S`/`B` (Bulk ops), `q` (Query), `Q` (QueryResponse), `n` (Notification)

**Key decisions:**
- **Stateful parsing** for `G`/`Q` responses: parser maintains `last_sync_op_context` from preceding `g`/`q`
- Bulk operations stored as single record (not split into multiple)
- Auto-detection: second `|`-segment is single char op code (vs SWSS multi-char TABLE_NAME)
- Unknown op codes gracefully fall back (op as function, detail as message)

### Regex Parser Optimization

- Benchmark framework using `criterion` in `crates/scouty/benches/`
- `source`/`loader_id` → `Arc<str>` (shared, not per-record allocation)
- `metadata` → `Option<HashMap>` or `SmallVec` for few KV pairs
- Timestamp parsing: direct format parsing without fallback when format known; hand-written fast path for common syslog timestamp formats
- `LogLevel::from_str_loose` — zero-allocation via `eq_ignore_ascii_case`

## Performance Benchmarks

| Parser | Target | Notes |
|--------|--------|-------|
| UnifiedSyslogParser (all 3 formats) | ≥ 10M rec/sec | Zero-regex, byte-level |
| SWSS Parser | ≥ 1M rec/sec | Hand-written |
| Sairedis Parser | ≥ 1M rec/sec | Hand-written |
| Regex Parser (syslog) | ≥ 1M rec/sec | Optimized regex |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-19 | Regex parser benchmark framework and optimization |
| 2026-02-20 | Extended syslog format support, hostname/container fields |
| 2026-02-21 | UnifiedSyslogParser consolidating 4 parsers |
| 2026-02-21 | SONiC SWSS log parser with context/function fields |
| 2026-02-22 | SONiC sairedis log parser with stateful G/Q context |
