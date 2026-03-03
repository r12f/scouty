# Changelog

## [0.3.3] - 2026-03-03

### New Features

- **6 built-in themes**: mizuiro (default), landmine, amai, maid, gyaru, dopamine — removed old default/dark/light/solarized
- **`--theme-list`** CLI command: lists all available themes with descriptions
- **`--theme-dump`** (renamed from `--generate-theme`): exports theme YAML with description field
- **Theme description**: now a serde field in Theme struct, included in YAML export
- **Windows config paths**: System (%PROGRAMDATA%\scouty\), User (%APPDATA%\scouty\)
- **Density chart X-axis tick marks** with per-theme `density_tick` color
- **Time jump millisecond support** (e.g. `500ms`)
- **Status bar bookmark format**: simplified to ★N

### Bug Fixes

- Fix highlight colors becoming identical after deletion (#568)
- Fix density chart time bucket alignment to human-friendly intervals (#533)
- Fix help dialog PageUp/PageDown swap (#528)
- Fix density tick review comments (chart_width formula, boundary tests) (#546)
- Replace remaining ANSI 16 colors with RGB in default theme (#531)

### Theme Changes

- amai: darker background, brighter highlights
- dopamine: lavender header, pure red CRIT, neon pink ERROR

## [0.3.0] - 2026-03-01

### New Features

- **Follow Mode** — Real-time log tailing with `--follow` / `-f` (#468, #469, #470, #478, #479)
  - File watcher with incremental loading (polling-based)
  - Auto-scroll with `[FOLLOW]` / `[FOLLOW ↓N]` status bar indicators
  - File truncation detection and automatic reload
  - File rotation (inode change) detection
  - File deletion handling with graceful warning
  - Stdin implicit follow (`command | scouty` streams until EOF)
  - Full integration with filters, regions, categories, stats, and search
  - `Ctrl+]` to disable follow mode; `G` to resume auto-scroll

- **Log Categorization** — YAML-configured log categories (#455, #456, #457)
  - `~/.scouty/categories.yaml` configuration with reusable filter syntax
  - CategoryProcessor for batch and streaming log classification
  - Category panel (`C` toggle) with sparkline density charts
  - Non-exclusive: one log record can match multiple categories

- **Panel System** — Collapsible bottom panels with tab bar (#377, #382, #383, #384, #430)
  - Tab bar with focus highlighting (Tab/Shift+Tab to cycle)
  - Detail, Region, Category, and Statistics panels
  - Maximize (`z`), collapse, and panel-specific key handlers

- **UI Architecture Rewrite** — Window/Widget traits with overlay stack (#433, #436, #437, #438)
  - Window and Widget traits with event bubbling
  - WindowStack for overlay management (push/pop, input routing)
  - All 11 overlay dialogs migrated to new architecture
  - Dynamic shortcut hints collected from widget tree

- **Region System** — Configurable log span detection (#358, #363, #365, #366, #372)
  - YAML region configuration with correlation logic
  - Region processor with timeout-based region creation
  - Region panel with list + timeline split view
  - Gutter markers, region manager, and navigation

- **Pipe Output Mode** — Non-interactive scripting support (#357)
  - `command | scouty --pipe` for filtered output without TUI

- **Sairedis Parser** — NotifySyncd op codes (#485)
  - `a` (NotifySyncd request) and `A` (NotifySyncdResponse) parsing
  - Op code count: 13 → 15

### Improvements

- Dynamic status bar shortcut hints based on focus context (#448, #450)
- Simplified/compact status bar hint display (#447)
- Runtime tracing with opt-in `--log` CLI flag (#399, #402, #404)
- Table header unfocused style when panel has focus (#423)
- Removed redundant panel title bars (tab bar already shows names) (#466)

### Bug Fixes

- Tab/Shift+Tab panel cycling fixes (6 iterations) (#405, #409, #413, #417, #418, #420, #429)
- Panel focus/highlight/render fixes (#388, #389, #393, #396)
- Detail panel fold keys changed from `h`/`l` to `←`/`→` (#453)
- Ctrl+Arrow keybindings removed entirely (#453)
- Enter/r/S expand panel without switching focus (#444)
- Tab bar highlight clearing when panel loses focus (#473)
- Follow mode keybinding conflict with user config (#480)
- Help dialog keybindings match spec (#376)

## [0.2.0] - 2026-02-27

Initial public release.
