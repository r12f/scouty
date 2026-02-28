# Log Categorization

## Changelog

| Date | Change |
|------|--------|
| 2026-02-28 | Initial spec — categorization processor + Category panel |

## Background & Goals

Users often want to classify log records into meaningful groups — errors by subsystem, request types, lifecycle phases, etc. Currently this requires manual filtering. The categorization processor automatically classifies every log record at load time, maintains per-category statistics, and presents results in a dedicated panel.

## Problem Statement

- No way to define reusable log classification rules
- No aggregate view showing distribution across custom categories
- Understanding log composition requires repeated manual filtering

## User Stories

- As a log analyst, I want to define categories (e.g., "SAI errors", "port events", "routing updates") so logs are automatically classified on load
- As a log analyst, I want to see a summary panel showing each category's record count and time distribution so I can quickly identify hotspots
- As a log analyst, I want categories to update in real-time as new log records are loaded (streaming/tailing)

## Design

### Concepts

- **Category Definition** — a named classification rule with a filter expression and optional actions
- **Category Group** — a collection of category definitions (one YAML file = one group)
- **Categorization Processor** — a log processor that evaluates each incoming record against all category definitions; when a filter matches, the record is counted in that category's stats
- **Category Panel** — a TUI panel tab showing per-category statistics and density charts

### Processing Flow

```
For each incoming log record:
  1. Evaluate against all category definitions (in config order)
  2. For each matching category:
     a. Increment category record count
     b. Update category density data (time-bucketed histogram)
  3. A record can match multiple categories (non-exclusive)
```

Note: categorization is **non-exclusive** — a single record may belong to multiple categories. Categories are independent classifications, not partitions.

### Configuration

Category configs are YAML files stored in:
- `/etc/scouty/categories/*.yaml` — system-wide
- `~/.scouty/categories/*.yaml` — user-level
- `./scouty-categories/*.yaml` — project-level

Loading order follows config precedence (system → user → project → CLI). Each file defines a category group with one or more categories.

### Config File Format

```yaml
# ~/.scouty/categories/network-events.yaml

categories:
  - name: "BGP Updates"
    filter: 'component == "bgp" && level >= info'

  - name: "Port Flaps"
    filter: 'message contains "link state changed"'

  - name: "SAI Errors"
    filter: 'component == "sairedis" && level >= error'

  - name: "Config Changes"
    filter: 'message contains "config" && message contains "changed"'
```

### Category Definition Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name shown in the Category panel |
| `filter` | Yes | Filter expression (same syntax as interactive filter: `field == "value"`, `contains`, `>=`, `&&`, `\|\|`) |

### Data Model

```rust
struct CategoryDefinition {
    name: String,
    filter: FilterExpression,
}

struct CategoryStats {
    definition: CategoryDefinition,
    count: usize,                    // Total matching records
    density: Vec<u64>,               // Time-bucketed histogram (same bucketing as density chart)
}

struct CategoryStore {
    categories: Vec<CategoryStats>,  // Ordered as defined in config
}
```

The `CategoryStore` is updated by the categorization processor and read by the Category panel for rendering.

## UI — Category Panel

The Category panel is a new tab in the TabbedContainer: `[Detail] [Region] [Stats] [Category]`

**Shortcut:** `C` (from log table, toggles Category panel open/close without switching focus)

### Layout

```
┌─ Category Panel ───────────────────────────────────────────────────────┐
│ Name              Count   Density                                      │
│─────────────────────────────────────────────────────────────────────── │
│ BGP Updates         1,234  ▁▂▅▇█▇▅▃▂▁▁▁▂▃▅▇▇▅▃▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁ │
│ Port Flaps            87  ▁▁▁▁▁▁█▁▁▁▁▁▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁ │
│ SAI Errors           456  ▁▁▃▅▇▇▅▃▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁ │
│ Config Changes        23  ▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁ │
└────────────────────────────────────────────────────────────────────────┘
```

### Columns

| Column | Description | Alignment |
|--------|-------------|-----------|
| Name | Category display name | Left |
| Count | Total matching records, comma-formatted | Right |
| Density | Sparkline density chart (same time-bucketing as status bar density) | Left, fills remaining width |

### Keybindings (Category panel widget)

| Key | Action |
|-----|--------|
| `j`/`k` | Move cursor up/down through categories |
| `Enter` | Apply this category's filter as the active filter (jump to log table with filter applied) |
| `Esc` | Return focus to log table |

### Rendering Rules

- Categories listed in config definition order
- Selected row highlighted with accent color
- Density sparkline uses same block characters as status bar density chart: `▁▂▃▄▅▆▇█`
- Density chart time range matches the full log time range
- Count updates in real-time as new records stream in

## Requirements Breakdown

### P0 — Must Have
- [ ] CategoryStore data model (dependency: none)
- [ ] Categorization processor — evaluate filters, update counts + density (dependency: CategoryStore)
- [ ] Category config loading from YAML files (dependency: CategoryStore)
- [ ] Category panel — list view with name, count, density sparkline (dependency: CategoryStore + panel system)
- [ ] `C` shortcut to toggle Category panel (dependency: Category panel)

### P1 — Should Have
- [ ] `Enter` on category applies its filter to the log table (dependency: Category panel + filter system)
- [ ] Real-time update when tailing/streaming logs (dependency: Categorization processor)

### P2 — Nice to Have
- [ ] Category sorting by count (descending) as alternative to config order
- [ ] Percentage column showing `count / total_records`

## Non-Functional Requirements

- **Performance:** Categorization must not add perceptible lag to log loading; filter evaluation should be O(1) per category per record
- **Memory:** Category density data uses same bucketing as existing density chart — fixed-size histogram, not per-record storage
- **Config errors:** Invalid filter expressions in category config are reported as warnings on startup; the invalid category is skipped, others still load

## Acceptance Criteria

- [ ] Category YAML configs load from `/etc/scouty/categories/`, `~/.scouty/categories/`, `./scouty-categories/`
- [ ] Each incoming log record is evaluated against all categories; matching records increment count
- [ ] Category panel shows all defined categories with name, count, and density sparkline
- [ ] `C` toggles Category panel without switching focus
- [ ] `Tab`/`Shift+Tab` cycles through panels including Category
- [ ] Density sparkline covers full log time range
- [ ] A record can match multiple categories
- [ ] Invalid category filter shows warning, does not crash

## Out of Scope

- Category-based coloring/highlighting of log records in the log table
- Nested/hierarchical categories
- Category editing from TUI (config files only)
- Exporting category statistics

## Open Questions

(None)
