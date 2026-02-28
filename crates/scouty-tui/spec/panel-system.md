# Panel System

## Background & Goals

The detail panel and region UI (region manager + density chart) currently have inconsistent UI patterns. This spec unifies them into a **collapsible panel system**: panels sit collapsed at the bottom of the log table, expand on shortcut, switch between panels with Ctrl+arrow keys, and share common operations while each panel has its own unique features.

**Value:** A unified panel model means users learn one interaction pattern for all panels. Future panels (stats, bookmarks, etc.) can reuse the framework directly.

## Problem Statement

1. Detail panel and region UI have inconsistent interaction models (detail is a fixed area, region manager was a floating window)
2. No unified panel switching mechanism
3. Cannot see collapsed state of multiple panels at a glance

## User Stories

- As a log analyst, I want to expand/collapse bottom panels with shortcuts so I can view details/region info without leaving the log table
- As a log analyst, I want unified shortcuts to switch between panels so I can quickly compare different dimensions of information
- As a log analyst, I want collapsed panels to take no screen space so the log table viewing area is maximized

## Requirements

### P0 — Must Have

- [ ] **Panel Framework** — unified panel trait/interface defining render, keybinding, collapse/expand behavior (dependency: none)
- [ ] **Detail Panel Migration** — migrate existing detail panel to a panel system instance (dependency: Panel Framework)
- [ ] **Region Panel** — combine region manager + region density chart into a single region panel (dependency: Panel Framework)
- [ ] **Panel Switching** — `Tab`/`Shift+Tab` to cycle focus between log table and panels (dependency: Panel Framework)
- [ ] **Panel Maximize** — `z` to toggle maximize/restore; maximized panel fills log table + panel area (dependency: Panel Framework)

### P1 — Should Have

(none currently)

### P2 — Nice to Have

- [ ] **Multiple Panels Open** — horizontally split bottom area to show multiple panels (dependency: Panel Framework)

## Functional Requirements

### Panel System Architecture

#### Layout

```
┌───────────────────────────────────────────────────────┐
│                                                       │
│                    Log Table                           │
│                                                       │
├─── [Detail] ── [Region] ── [Stats] ── [Category] ───────┤  ← Panel Tab Bar (only this line when collapsed)
│                                                       │
│              Active Panel Content                     │
│                                                       │
└───────────────────────────────────────────────────────┘
│ ▁▂▃▅▇█▇▅▃▂▁ │ 1,234/5,678             ← Line 1      │
│ [VIEW] /: Search │ f: Filter            ← Line 2      │
└───────────────────────────────────────────────────────┘
```

- Panel area sits between the log table and the status bar
- **Collapsed:** only the panel tab bar is visible (one line height), showing tab names like `[Detail] [Region] [Stats] [Category]` with the selected tab highlighted
- **Expanded:** tab bar + panel content area; each panel has its own default height (see individual panel definitions)
- Only one panel is expanded at a time

#### Panel Tab Bar

```
Collapsed:  ▸ Detail │ Region
Expanded:   ▾ Detail │ Region
```

- `▸` indicates collapsed, `▾` indicates expanded
- Selected panel name is highlighted (e.g., reverse video or bold)
- Inactive panel names displayed normally
- Tab bar is always visible (regardless of collapse/expand state)
- **Focus indicator:** When the panel has focus, the active tab title uses `theme.panel_tab.focused` style (accent bg, bold). When focus is on the log table, it uses `theme.panel_tab.unfocused` style (muted/gray). See [theme.md](theme.md) for per-theme color definitions.

#### Focus Model

Three focus layers:
1. **Log Table** — default focus, standard log table operations
2. **Panel Tab Bar** — switch/select panels
3. **Panel Content** — panel-internal operations

Focus switching:

| Shortcut | Behavior |
|----------|----------|
| `Tab` | Cycle focus forward: Log Table → Detail → Region → Stats → Category → Log Table → … (expands panel if collapsed) |
| `Shift+Tab` | Cycle focus backward (reverse of Tab) |
| `Esc` | Panel Content → Log Table (focus returns to log table) |
| Original shortcut | Directly open/close corresponding panel without changing focus (e.g., `Enter` toggles Detail, `r` toggles Region, `S` toggles Stats, `C` toggles Category). Focus stays on the current widget (typically log table). Use `Tab` to move focus into the panel. |

#### Shared Panel Operations (all panels)

| Shortcut | Behavior |
|----------|----------|
| `j`/`k` | Navigate up/down within panel |
| `Esc` | Return focus to log table |
| `Esc` | Return focus to log table |
| `Tab`/`Shift+Tab` | Cycle to next/previous panel or back to log table |

**Maximize:**

| Shortcut | Behavior |
|----------|----------|
| `z` | Toggle maximize/restore. When maximized, panel fills log table + panel area, keeping only tab bar + status bar. Press `z` again to restore default height. |

#### Panel Registration

```rust
trait Panel: UiComponent {
    fn name(&self) -> &str;           // Tab display name
    fn shortcut(&self) -> char;        // Quick-open shortcut key
    fn default_height(&self) -> PanelHeight;  // Default height strategy
    fn is_available(&self) -> bool;    // Whether panel has content to show
    fn on_log_cursor_changed(&mut self, index: usize);  // Notified when log table cursor moves
}

enum PanelHeight {
    FitContent,              // Adapt to content (e.g., Detail panel)
    Percentage(u16),         // Percentage of terminal height (e.g., Region panel 40%)
}
```

Panels are registered via trait. Adding a new panel only requires implementing this trait.

### Detail Panel (Migration)

**Tab name:** `Detail`
**Shortcut:** `Enter` (from log table)
**Default height:** `FitContent` — adapts to content (same behavior as current detail panel)
**Content:** identical to existing detail panel — left side message/expansion tree, right side fields

Panel-specific operations:

| Shortcut | Behavior |
|----------|----------|
| `Tab` | Switch focus between left and right areas |
| `←`/`→` | Collapse/expand tree node (left side expansion tree) |
| `H`/`L` | Collapse all / expand all |
| `f` | Create filter from current field |

**Follows cursor:** automatically updates content when log table cursor moves.

### Region Panel (New)

**Tab name:** `Region`
**Shortcut:** `r` (from log table)
**Default height:** `Percentage(40)` — 40% of terminal height
**Content:** left-right split layout

#### Layout

```
Region                                                                          
┌─ Region List (~70%) ──────────────────────────────────┬─ Timeline (~30%) ─────┐
│                                                       │  port_startup         │
│  Port Startup Ethernet0   10:30:45→10:30:47  2.1s     │  ──██──████──░░──     │
│  Port Startup Ethernet4   10:30:45→10:30:48  3.0s     │                       │
│▸ Port Startup Ethernet20  10:30:45→?         >30s ⏱   │  http_request         │
│  SAI Create ROUTE_ENTRY   10:30:46→10:30:46  12ms     │  █─█──██──████─       │
│  HTTP GET /api/status     10:31:02→10:31:02  45ms     │                       │
│  HTTP POST /api/login     10:31:05→10:31:06  1.2s     │  sai_create           │
│                                                       │  ─██─█──██──          │
│  Total: 6 regions (3 types) │ 5 completed │ 1 timeout │                       │
└───────────────────────────────────────────────────────┴───────────────────────┘
```

**Left — Region List (~70% width):**
- One row per region: name, start→end time, duration, description
- Sorted by start time; ties broken by end time (ascending)
- Timed-out regions marked with `⏱`, duration shows `>timeout`
- `j`/`k` navigation, selected row highlighted
- Focus defaults to left side

**Right — Timeline (~30% width, minimum 40 characters):**
- One row per region type, showing type name + mini timeline bar for all regions of that type
- `████` represents region duration, `░░` represents timed-out regions
- Time axis auto-scales to cover all regions' time range
- When left cursor moves, the corresponding type row on the right highlights, and the selected region's bar is marked with a distinct color
- When terminal width is insufficient (right side < 40 chars), right side hides and left side fills full width

**Left-right linkage:**
- Selecting a region on the left → corresponding type row highlights on the right, selected region's bar shown in accent color
- Right side is view-only, not independently operable (focus always stays on the left)

#### Panel-Specific Operations

| Shortcut | Behavior |
|----------|----------|
| `j`/`k` | Navigate region list up/down |
| `Enter` | Jump to selected region's start record (focus returns to log table) |
| `f` | Filter log table to selected region's record range |
| `t` | Filter list by region type (toggle; press again to show all) |
| `s` | Toggle sort order (start time / duration) |

**Follows cursor:** when log table cursor moves into a region's record range, the corresponding region in the list auto-highlights.

### Region Markers (Preserved)

Gutter markers in the log table (▶/│/◀) are preserved unchanged. They are part of the log table, not the panel system.

## Non-Functional Requirements

- **Performance:** No render computation when panel is collapsed; only render visible area when expanded
- **Terminal compatibility:** Minimum terminal width 60 columns; panel auto-collapses when height is insufficient
- **Responsiveness:** Panel expand/collapse/switch must be instant (no animation, no delay)

## Acceptance Criteria

- [ ] `Enter` opens Detail panel, then `Tab` switches to Region panel
- [ ] `r` directly opens Region panel
- [ ] `Tab`/`Shift+Tab` cycle focus between log table and panels
- [ ] Tab bar takes only one line when collapsed
- [ ] Log table height auto-shrinks when panel expands
- [ ] Region panel left-right split: left region list, right timeline (~30% width, min 40 chars)
- [ ] Region list sorted by start time → end time
- [ ] Selecting a region on left highlights corresponding type on right with accent-colored bar
- [ ] Right timeline auto-hides when terminal width insufficient
- [ ] Detail panel follows log table cursor updates
- [ ] Region panel list highlights corresponding region when cursor is within a region
- [ ] `Esc` returns from panel to log table
- [ ] `z` maximizes panel (log table hidden), press again to restore

## Out of Scope

- User-defined panels (plugin panels) — future consideration
- Panel drag-to-reorder — fixed order is sufficient
- Left/right side panels — bottom only
- Floating/detached panel mode — embedded only

## Open Questions

(All resolved)
