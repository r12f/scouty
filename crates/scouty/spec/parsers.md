# Parsers

## Overview

Scouty parsers transform raw log lines into `LogRecord` structs. The system supports multiple parser types: a unified hand-written syslog parser (zero-regex), SONiC SWSS parser, sairedis parser, JSON log parser, and user-defined regex parsers.

All parsers may optionally populate the `expanded` field on `LogRecord` to provide structured expansion of log content for the detail panel (see log-record spec for `ExpandedField` structure).


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
| `0-9{4}-` + ` ` at pos 10 | High-precision syslog | `2025-11-24 17:56:03.073872-08:00 hostname process[pid]: msg` |

#### High-Precision Syslog Format (RSYSLOG_FileFormat)

rsyslog's `RSYSLOG_FileFormat` template (and systemd-journald's default file output) uses `YYYY-MM-DD HH:MM:SS.ffffff+ZZ:ZZ` — an ISO 8601-like timestamp but with a space instead of `T` between date and time. This is the default format on many modern Linux distributions.

**Detection:** `0-9{4}` at pos 0-3, `-` at pos 4, digit at pos 5, `-` at pos 7, digit at pos 8, **space** at pos 10 (distinguishes from ISO 8601 which has `T`).

**Variants:**

| Variant | Example |
|---------|---------|
| With timezone | `2026-03-03 06:54:06.123456-08:00 hostname process[pid]: msg` |
| Without fractional seconds | `2026-03-03 06:54:06-08:00 hostname process[pid]: msg` |
| UTC (no offset) | `2026-03-03 06:54:06.123456 hostname process[pid]: msg` |

**Parsing:** Same as ISO 8601 syslog path but accept space at pos 10 in addition to `T`. After parsing the timestamp, continue with the shared `parse_hostname` -> `parse_process_part` -> `parse_message` pipeline.

**Dual-timestamp lines:** Some rsyslog configurations prepend a second timestamp (e.g., `2026-03-03 06:54:06 2026-03-01T00:00:39.241739-08:00 hostname ...`). When two consecutive timestamps are detected at the start of a line, use the **second** (more precise) timestamp as the event time and discard the first (reception time).

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
- Distinguishes from SWSS by checking `.` vs `T`/` ` at position 10 in the timestamp
- High-precision format (space at pos 10) must also be distinguished from `iso-level-msg` common parser (check if token after timestamp is a log level keyword)

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

**Structured expansion:** Populates `expanded` with Operation, Table, Key, and Attributes (KV pairs as ordered key-value tree).

**Parsing logic:** Split by `|`, locate SET/DEL position, determine TABLE:Key vs TABLE|SubKey format. Key may contain `:` (e.g., IPv6), so only split at first `:`.

### SONiC Sairedis Log Parser

Parses SAI Redis operation logs: `YYYY-MM-DD.HH:MM:SS.ffffff|<op>|<detail...>`

**15 op codes:** `c` (Create), `r` (Remove), `s` (Set), `g` (Get), `G` (GetResponse), `a` (NotifySyncd), `A` (NotifySyncdResponse), `p` (CounterPoll), `C`/`R`/`S`/`B` (Bulk ops), `q` (Query), `Q` (QueryResponse), `n` (Notification)

**Key decisions:**
- **Stateful parsing** for `G`/`Q` responses: parser maintains `last_sync_op_context` from preceding `g`/`q`
- Bulk operations stored as single record (not split into multiple)
- Auto-detection: second `|`-segment is single char op code (vs SWSS multi-char TABLE_NAME)
- Unknown op codes gracefully fall back (op as function, detail as message)

**Structured expansion:** Populates `expanded` with Operation (human-readable name), Object Type, OID, and Attributes. For stateful `G`/`Q` responses, expansion includes the correlated request context.

**Response status extraction:** For response operations (`G`, `A`, `Q`), the first attribute in the detail is a SAI status code (e.g., `SAI_STATUS_SUCCESS`). The parser must extract this as a dedicated **Status** field in the expansion (separate from the Attributes list). Remaining attributes follow as normal KV pairs. This ensures the error/success status is prominently visible in the Detail panel rather than buried inside the attributes tree.

### JSON Log Parser

Parses log lines that are complete JSON objects (one object per line, i.e., NDJSON/JSON Lines).

**Auto-detection:** Line starts with `{` and is valid JSON.

**Field mapping (well-known keys):**

| JSON key (case-insensitive) | LogRecord field |
|----|-----|
| `timestamp`, `time`, `ts`, `@timestamp` | `timestamp` |
| `level`, `severity`, `loglevel` | `level` |
| `message`, `msg`, `log` | `message` |
| `hostname`, `host` | `hostname` |
| `service`, `component`, `logger`, `name` | `component_name` |
| `pid` | `pid` |
| `tid`, `thread` | `tid` |

All other keys go to `metadata`.

**Structured expansion:** Populates `expanded` with a "Payload" `KeyValue` tree, recursively expanding nested objects and arrays. Well-known fields already mapped to LogRecord top-level fields are excluded from expansion to avoid duplication. Nested objects become nested `KeyValue`, arrays become `List`.

**Key decisions:**
- Well-known field names are case-insensitive for broad compatibility
- Numeric/boolean JSON values converted to string for `metadata`
- Nested JSON preserved in structured expansion (not flattened)
- Very deep nesting (>10 levels): truncated with `...` marker
- Invalid JSON lines: fall through to next parser in the group

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
| JSON Parser | ≥ 500K rec/sec | `serde_json` + field mapping |
| Regex Parser (syslog) | ≥ 1M rec/sec | Optimized regex |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-19 | Regex parser benchmark framework and optimization |
| 2026-02-20 | Extended syslog format support, hostname/container fields |
| 2026-02-21 | UnifiedSyslogParser consolidating 4 parsers |
| 2026-02-21 | SONiC SWSS log parser with context/function fields |
| 2026-02-22 | SONiC sairedis log parser with stateful G/Q context |
| 2026-02-24 | JSON log parser with well-known field mapping |
| 2026-02-24 | Structured expansion (expanded field) for SWSS, sairedis, and JSON parsers |
| 2026-03-03 | Added high-precision syslog format (RSYSLOG_FileFormat) with YYYY-MM-DD HH:MM:SS detection |
| 2026-03-03 | Added dual-timestamp handling — use second (precise) timestamp when two are present |
