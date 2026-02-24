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
| `expanded` | `Option<Vec<ExpandedField>>` | Parser-provided structured expansion of the log content (None when not applicable) |

### ExpandedField

A recursive tree structure for representing parsed/expanded log content:

```rust
enum ExpandedValue {
    Text(String),                          // Simple text value
    KeyValue(Vec<(String, ExpandedValue)>),  // Ordered key-value pairs (preserves order)
    List(Vec<ExpandedValue>),              // Ordered list of values
}

struct ExpandedField {
    label: String,           // Display label (e.g., "TABLE", "Attributes", "Payload")
    value: ExpandedValue,    // The structured content
}
```

**Design rationale:**
- Each parser knows its own log format best — parsers provide the expansion, not the UI
- `ExpandedValue` is recursive, supporting nested JSON objects, SWSS table→key→attrs, sairedis op→oid→attrs
- `Option<Vec>` avoids allocation for simple log lines that don't need expansion
- Ordering is preserved (important for readability — parsers control display order)

**Examples by parser:**

SWSS log `2025-01-15.10:30:45.123456|ROUTE_TABLE|Vrf1:10.0.0.0/24|SET|nexthop:10.1.1.1|ifname:Ethernet0`:
```
expanded:
  - label: "Operation"
    value: Text("SET")
  - label: "Table"
    value: Text("ROUTE_TABLE")
  - label: "Key"
    value: Text("Vrf1:10.0.0.0/24")
  - label: "Attributes"
    value: KeyValue([("nexthop", Text("10.1.1.1")), ("ifname", Text("Ethernet0"))])
```

Sairedis log `2025-01-15.10:30:45.123456|c|SAI_OBJECT_TYPE_ROUTE_ENTRY:...`:
```
expanded:
  - label: "Operation"
    value: Text("Create")
  - label: "Object Type"
    value: Text("SAI_OBJECT_TYPE_ROUTE_ENTRY")
  - label: "OID"
    value: Text("oid:0x5000000000612")
  - label: "Attributes"
    value: KeyValue([("SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID", Text("oid:0x4000...")), ...])
```

JSON log `{"timestamp":"...","level":"ERROR","service":"auth","msg":"login failed","details":{"user":"alice","ip":"10.0.0.1"}}`:
```
expanded:
  - label: "Payload"
    value: KeyValue([
      ("service", Text("auth")),
      ("msg", Text("login failed")),
      ("details", KeyValue([("user", Text("alice")), ("ip", Text("10.0.0.1"))]))
    ])
```

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
| 2026-02-24 | Added `expanded` field with ExpandedField/ExpandedValue tree for parser-provided structured expansion |
