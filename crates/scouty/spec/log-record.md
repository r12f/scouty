# Log Record

## Overview

`LogRecord` is the core data structure representing a single parsed log entry. It is the universal exchange format between all Scouty components — parsers produce it, the store holds it, filters evaluate it, and the TUI renders it.


## Design

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `DateTime` | Log timestamp |
| `level` | `LogLevel` | Severity level |
| `source` | `Arc<str>` | Log source identifier (shared across records from same loader) |
| `hostname` | `Option<String>` | Hostname (first-class field) |
| `container` | `Option<String>` | Container name (first-class field) |
| `context` | `Option<String>` | Context info (e.g., SWSS table key, SAI OID) |
| `function` | `Option<String>` | Function/operation (e.g., SWSS SET/DEL, SAI op) |
| `pid` | `Option<u32>` | Process ID |
| `tid` | `Option<u32>` | Thread ID |
| `component_name` | `Option<String>` | Component name |
| `process_name` | `Option<String>` | Process name |
| `message` | `String` | Log message body |
| `raw` | `String` | Original raw log text |
| `metadata` | `Option<HashMap<String, String>>` | Extensible key-value metadata (None when empty to avoid allocation) |
| `loader_id` | `Arc<str>` | Source loader identifier (shared) |

### LogLevel Enum

Includes: `Trace`, `Debug`, `Info`, `Notice`, `Warn`, `Error`, `Fatal`

Key decisions:
- **NOTICE is a distinct level** — not mapped to INFO, preserved as independent level for syslog compatibility.
- **Immutability** — once parsed, a LogRecord is immutable.
- **Shared strings** — `source` and `loader_id` use `Arc<str>` to avoid per-record allocation since they are identical within a batch.
- **Optional metadata** — when no extra KV pairs exist, `metadata` is `None` (zero HashMap allocation).

### Filter Expression Support

All fields (including `hostname`, `container`, `context`, `function`, and metadata keys) are addressable in filter expressions.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-18 | Initial LogRecord design with core fields |
| 2026-02-20 | Added `hostname` and `container` as first-class fields |
| 2026-02-21 | Added `context` and `function` for SWSS/sairedis support |
| 2026-02-19 | Optimized `source`/`loader_id` to `Arc<str>`, metadata to `Option<HashMap>` |
