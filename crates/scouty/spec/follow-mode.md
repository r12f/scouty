# Follow Mode

## Changelog

| Date | Change |
|------|--------|
| 2026-02-28 | Initial spec — follow mode for file tailing |

## Background & Goals

When analyzing live systems, users need to watch log files as new lines are appended — similar to `tail -f`. Follow mode keeps the log view updated with the latest records, auto-scrolling to the bottom as new data arrives.

## Problem Statement

- Currently scouty loads a file once and shows a static snapshot
- No way to monitor a live log file for new entries
- Users must manually re-open the file to see new records

## User Stories

- As a log analyst, I want to follow a log file so I can monitor live system behavior without leaving scouty
- As a log analyst, I want to pause following (stop auto-scroll) so I can investigate a specific section without being interrupted by new lines
- As a log analyst, I want to see a clear indicator when follow mode is active so I know the view is live

## Design

### Activation

Follow mode is **enabled only via CLI** — it cannot be turned on from within the TUI:

```bash
scouty --follow <file>
scouty -f <file>
```

**Rationale:** Enabling follow mode requires restarting the file reader with a file watcher. This is equivalent to re-opening the program, so it is a startup-only option.

### Behavior When Active

1. **File watching:** Monitor the target file for new data (appended bytes)
2. **Incremental loading:** Parse and append new log records to the LogStore as they arrive
3. **Auto-scroll:** When the cursor is at the last record, the view automatically scrolls to show new records as they arrive
4. **No auto-scroll when browsing:** If the user has scrolled up (cursor is not at the last record), new records are still loaded but the view does NOT auto-scroll. A "new records" indicator is shown in the status bar.

### Pausing / Disabling Follow

Follow mode can be **dynamically disabled** from within the TUI but **cannot be re-enabled**:

| Action | Effect |
|--------|--------|
| `F` key (toggle) | Disable follow mode — stop watching file, stop loading new records. View becomes static. |
| Re-enable | Not possible from TUI. User must restart scouty with `--follow` to re-enable. |

When follow is disabled:
- File watcher is stopped
- No more records are loaded
- Status bar indicator changes to show follow is off
- All loaded records remain in the LogStore

### Status Bar Indicator

Follow mode state is shown in the status bar (line 1):

| State | Indicator | Description |
|-------|-----------|-------------|
| Following (at bottom) | `[FOLLOW]` | Actively following, auto-scrolling |
| Following (scrolled up) | `[FOLLOW ↓123]` | Following but user browsed up; 123 new records below |
| Follow disabled | (no indicator) | Static mode, no file watching |

The `[FOLLOW]` indicator appears before the density chart on status bar line 1.

### Interaction with Other Features

| Feature | Behavior |
|---------|----------|
| **Filters** | Applied to new records as they arrive; filtered records update counts |
| **Regions** | Region processor runs on new records; open regions may get new end points |
| **Categories** | Category processor evaluates new records; counts and density update |
| **Statistics** | Stats panel updates with new record data |
| **Search** | Search results include new records that match |
| **`G` (go to end)** | Jumps to the latest record and re-enables auto-scroll |

### File Handling

- **File truncation:** If the file is truncated (size decreases), reload from the beginning
- **File rotation:** If the file is replaced (inode changes), switch to the new file
- **File deletion:** If the file is deleted, show warning in status bar, keep existing records
- **Read interval:** Poll for new data at a reasonable interval (e.g., 100ms) or use OS file watching (inotify/kqueue/ReadDirectoryChanges)
- **Stdin follow:** When reading from stdin (pipe), follow mode is **implicit** — stdin is always streamed until EOF

## Requirements Breakdown

### P0 — Must Have
- [ ] `--follow` / `-f` CLI flag (dependency: none)
- [ ] File watcher — detect new data appended to file (dependency: none)
- [ ] Incremental loading — parse new bytes, append records to LogStore (dependency: file watcher)
- [ ] Auto-scroll when cursor is at last record (dependency: incremental loading)
- [ ] `[FOLLOW]` status bar indicator (dependency: follow mode state)
- [ ] `F` key to disable follow mode (dependency: follow mode state)

### P1 — Should Have
- [ ] `[FOLLOW ↓N]` indicator showing count of new records below when scrolled up (dependency: follow mode + status bar)
- [ ] `G` re-enables auto-scroll (dependency: auto-scroll logic)
- [ ] File truncation detection — reload from beginning (dependency: file watcher)
- [ ] File rotation detection — switch to new file (dependency: file watcher)

### P2 — Nice to Have
- [ ] File deletion warning in status bar
- [ ] OS-native file watching (inotify/kqueue) instead of polling

## Non-Functional Requirements

- **Latency:** New records should appear within 200ms of being written to the file
- **CPU:** Idle CPU usage in follow mode should be negligible when no new data arrives
- **Memory:** Incremental loading must not re-parse existing records; only new bytes are processed

## Acceptance Criteria

- [ ] `scouty --follow <file>` opens file and watches for new data
- [ ] `scouty -f <file>` is equivalent shorthand
- [ ] New lines appended to the file appear in the log table automatically
- [ ] Auto-scroll works when cursor is at the last record
- [ ] Scrolling up pauses auto-scroll; `G` resumes it
- [ ] `[FOLLOW]` indicator shown in status bar when active
- [ ] `F` disables follow mode; indicator disappears; no more records loaded
- [ ] `F` when follow is already disabled is a no-op (does not re-enable)
- [ ] Filters, regions, categories, stats all process new records correctly
- [ ] Stdin input is implicitly followed until EOF

## Out of Scope

- Following multiple files simultaneously (future consideration)
- Network/remote file following
- Re-enabling follow mode from within TUI
- Follow mode for directory watching (watching for new files)

## Open Questions

(None)
