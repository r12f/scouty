# Region Parsing

## Overview

Region parsing identifies logical spans ("regions") in log streams by matching configurable start and end points. A region groups consecutive log records that belong to a single logical operation (e.g., a request lifecycle, a SAI bulk operation, a port startup sequence).

Regions are defined via YAML config files and processed by a log processor that runs after parsing, attaching region metadata to matched log records.


## Design

### Concepts

- **Region Definition** — a named configuration describing how to detect a region's start/end boundaries
- **Start Point** — a filter expression that identifies a potential region start; on match, regex extracts metadata
- **End Point** — a filter expression that identifies a potential region end; on match, regex extracts metadata
- **Region** — a created span from a matched start to a matched end, where extracted metadata fields correlate the two
- **Region Processor** — a log processor that evaluates records against region definitions and creates regions

### Region Detection Flow

```
For each incoming log record:
  1. Check against all active region definitions
  2. For each definition:
     a. Try each END POINT filter:
        - If matched → extract metadata via regex
        - Search backwards for the nearest unmatched START POINT
          whose extracted metadata matches on the specified correlation fields
        - If correlation succeeds → CREATE REGION (start..end)
        - Construct region name/description from template
     b. Try each START POINT filter:
        - If matched → extract metadata via regex
        - Store as pending start point (awaiting a matching end)
```

### Configuration

Region configs are YAML files stored in:
- `/etc/scouty/regions/*.yaml` — system-wide
- `~/.scouty/regions/*.yaml` — user-level
- `./scouty-regions/*.yaml` — project-level

Loading order follows config precedence (system → user → project). Each file can define multiple region definitions.

### Config File Format

```yaml
# ~/.scouty/regions/sonic-operations.yaml

regions:
  - name: "sai_bulk_create"
    description: "SAI bulk object creation operation"

    start_points:
      - filter: 'function == "c" AND component == "sairedis"'
        regex: 'SAI_OBJECT_TYPE_(?P<obj_type>\w+).*oid:(?P<oid>0x[0-9a-f]+)'
        reason: "single create"
      - filter: 'function == "C" AND component == "sairedis"'
        regex: 'SAI_OBJECT_TYPE_(?P<obj_type>\w+).*count:(?P<count>\d+)'
        reason: "bulk create ({count} objects)"

    end_points:
      - filter: 'function == "G" AND component == "sairedis"'
        regex: 'SAI_STATUS_(?P<status>\w+)'
        reason: "got response: {status}"
      - filter: 'function == "s" AND message =~ "SAI_STATUS"'
        regex: 'SAI_STATUS_(?P<status>\w+)'
        reason: "status callback: {status}"

    # Fields that must match between start and end to correlate them
    correlate:
      - "obj_type"    # extracted metadata field

    # Template for constructing region name and description
    # {start_reason} and {end_reason} reference the matched point's reason
    template:
      name: "SAI Create {obj_type}"
      description: "{start_reason} → {end_reason}"

  - name: "port_startup"
    description: "Port initialization to oper up"

    start_points:
      - filter: 'message =~ "addPort" AND component == "orchagent"'
        regex: '(?:addPort|initPort).*?(?P<port>Ethernet\d+)'
        reason: "port add requested"

    end_points:
      - filter: 'message =~ "oper_status.*up" AND component == "orchagent"'
        regex: '(?P<port>Ethernet\d+).*oper_status.*(?P<oper_status>up|down)'
        reason: "oper {oper_status}"
      - filter: 'message =~ "Port init failed"'
        regex: '(?P<port>Ethernet\d+).*(?P<error>.+)'
        reason: "init failed: {error}"

    correlate:
      - "port"        # same port name links start to end

    template:
      name: "Port Startup {port}"
      description: "{start_reason} → {end_reason}"

    # Optional: max time window between start and end (default: unlimited)
    timeout: "30s"

  - name: "http_request"
    description: "HTTP request lifecycle"

    start_points:
      - filter: 'message =~ "request started"'
        regex: 'request_id=(?P<req_id>[a-f0-9-]+).*method=(?P<method>\w+).*path=(?P<path>\S+)'
        reason: "request started"

    end_points:
      - filter: 'message =~ "request completed"'
        regex: 'request_id=(?P<req_id>[a-f0-9-]+).*status=(?P<status>\d+).*duration=(?P<duration>\S+)'
        reason: "completed {status} ({duration})"

    correlate:
      - "req_id"

    template:
      name: "{method} {path}"
      description: "{start_reason} → {end_reason}"

    timeout: "60s"
```

### Config Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `regions[].name` | string | yes | Unique identifier for this region type |
| `regions[].description` | string | no | Human-readable description |
| `regions[].start_points` | list | yes | One or more start point matchers |
| `regions[].start_points[].filter` | string | yes | Filter expression (same syntax as TUI filter) |
| `regions[].start_points[].regex` | string | no | Regex with named groups for metadata extraction (applied to `message` field). If omitted, no metadata extracted from this point. |
| `regions[].start_points[].reason` | string | no | Reason template for this start point. Supports `{field}` substitution from regex groups. Available as `{start_reason}` in region template. |
| `regions[].end_points` | list | yes | One or more end point matchers |
| `regions[].end_points[].filter` | string | yes | Filter expression |
| `regions[].end_points[].regex` | string | no | Regex with named groups for metadata extraction |
| `regions[].end_points[].reason` | string | no | Reason template for this end point. Supports `{field}` substitution from regex groups. Available as `{end_reason}` in region template. |
| `regions[].correlate` | list | yes | Metadata field names that must match between start and end |
| `regions[].template.name` | string | yes | Template string for region name (`{field}` substitution) |
| `regions[].template.description` | string | no | Template string for region description |
| `regions[].timeout` | string | no | Max duration between start and end (`30s`, `5m`, `1h`). Stale pending starts are discarded. Default: no timeout. |

### Correlation Logic

When an end point matches:

1. Extract metadata from the end record using regex
2. Walk backwards through pending (unmatched) start points for this region definition
3. For each pending start, check if ALL `correlate` fields have equal values between start and end metadata
4. First match wins → region is created from that start record to the current end record
5. The matched start is consumed (removed from pending list)
6. All log records between start and end are tagged with the region

If no correlation fields are specified or all are empty, the nearest pending start is used (LIFO).

### Region Data Structure

```rust
struct Region {
    definition_name: String,        // e.g., "port_startup"
    name: String,                   // e.g., "Port Startup Ethernet0" (from template)
    description: Option<String>,    // e.g., "port add requested → oper up" (from template)
    start_reason: Option<String>,   // e.g., "port add requested" (rendered from start point reason)
    end_reason: Option<String>,     // e.g., "oper up" (rendered from end point reason)
    start_index: usize,             // LogStore index of start record
    end_index: usize,               // LogStore index of end record
    metadata: HashMap<String, String>,  // merged metadata from start + end
}
```

### LogRecord Integration

Log records that belong to a region get tagged:

| Field | Type | Description |
|-------|------|-------------|
| `metadata["_region"]` | String | Region name (from template) |
| `metadata["_region_type"]` | String | Region definition name |
| `metadata["_region_pos"]` | String | Position within region: `start`, `middle`, `end`, or `start+end` (single-record region) |

This allows filtering by region: `_region == "Port Startup Ethernet0"` or `_region_type == "port_startup"`.

### TUI Integration

#### Region Markers in Log Table

- Start records: `▶` marker in a dedicated gutter column (left of table)
- End records: `◀` marker
- Middle records: `│` marker (within a region)
- Markers colored by region type (using highlight palette rotation)

#### Region Navigation

| Key | Function |
|-----|----------|
| `r` | Region manager — list all detected regions |
| `R` | Jump to next region start |

**Region Manager (`r`):**

```
┌─ Regions ───────────────────────────────────────────┐
│                                                     │
│  Port Startup Ethernet0          10:30:45 → 10:30:47│
│  Port Startup Ethernet4          10:30:45 → 10:30:48│
│  SAI Create ROUTE_ENTRY          10:30:46 → 10:30:46│
│  HTTP GET /api/status            10:31:02 → 10:31:02│
│                                                     │
│  Total: 4 regions (2 types)                         │
│                                                     │
│  [Enter] Jump  [f] Filter  [Esc] Close              │
└─────────────────────────────────────────────────────┘
```

- `Enter` — jump to region start record
- `f` — filter to show only records in selected region
- `j`/`k` navigation

#### Region Density Chart (Floating Window)

Region density chart is a **standalone floating window** (not part of the log density bar). It visualizes region distribution over time using a Gantt-style timeline.

**Open:** `r` → region manager → `d` on a region type, or directly via `Shift+D` from log table.

**Layout:** Floating window, 95% of log table width, 70% of log table height, centered.

```
┌─ Region Density: port_startup ─────────────────────────────────────────────────┐
│                                                                                │
│  Time     10:30:00    10:30:15    10:30:30    10:30:45    10:31:00             │
│           ┊           ┊           ┊           ┊           ┊                    │
│  Eth0     ████████████████████                                                 │
│  Eth4     ████████████████████████████                                         │
│  Eth8              ███████████                                                 │
│  Eth12                    █████████████████████████████                         │
│  Eth16                              ██████████                                 │
│  Eth20    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ (timeout)                     │
│                                                                                │
│  Total: 6 regions │ 5 completed │ 1 timed out                                  │
│                                                                                │
│  [j/k] Navigate  [Enter] Jump  [f] Filter  [t] Type  [Esc] Close              │
└────────────────────────────────────────────────────────────────────────────────┘
```

**Features:**
- Each row = one region instance, labeled by the primary correlate field value (e.g., port name)
- `████` bars show region duration; bar length proportional to time span
- Color: completed regions use region type color; timed-out regions use `░` (dimmed)
- Time axis auto-scales to fit all visible regions
- `j`/`k` to navigate rows
- `Enter` — jump to selected region's start record in log table
- `f` — filter log table to selected region
- `t` — switch between region types (if multiple types defined)
- Sorted by start time (default), `s` to toggle sort by duration
- `Esc` closes the floating window

**Behavior:**
- Does NOT replace or modify the existing log density chart in the status bar
- The log density chart (`d`/`D` keys) continues to work as before (level/highlight modes only)
- Region density is its own independent visualization

### CLI Integration (Pipe Mode)

```bash
# Filter by region type
scouty-tui --filter '_region_type == "port_startup"' --format json app.log

# Show only region start/end records
scouty-tui --filter '_region_pos == "start" OR _region_pos == "end"' app.log
```

### Performance Considerations

- Region processor runs as a post-parse step, after records are in LogStore
- Filter expressions compiled once at config load time
- Regex compiled once at config load time
- Pending start points stored in memory; `timeout` prevents unbounded growth
- Large files: region detection is incremental (processes new records as they arrive)


## Change Log

| Date | Change |
|------|--------|
| 2026-02-24 | Initial region parsing spec — configurable start/end matching, correlation, templates |
| 2026-02-24 | Region density chart as floating window (95%×70%), Gantt-style timeline, separate from log density bar |
| 2026-02-24 | Start/end point reason field — each point specifies its own reason, available as {start_reason}/{end_reason} in templates |
