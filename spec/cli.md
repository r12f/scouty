# CLI

## Overview

Command-line interface for `scouty-tui`, handling file arguments, default syslog paths, and stdin pipe detection.


## Design

### Usage

```
scouty-tui [file1] [file2] ...
```

### Argument Handling

| Arguments | Behavior |
|-----------|----------|
| None (Linux) | Open `/var/log/syslog`, fallback to `/var/log/messages` |
| None (no syslog found) | Friendly error + usage hint |
| None (macOS) | Usage hint (no default, macOS syslog differs) |
| One or more files | Load all files, each with independent loader + parser group |
| Pipe detected (`!isatty(stdin)`) | Read from stdin via StdinLoader |
| Pipe + files | Both stdin and files load into same LogStore |

### Multi-file Loading

- Each file gets independent FileLoader + ParserGroup (format auto-detected)
- All records merge into one LogSession / LogStore
- LogStore `insert_batch` handles cross-file timestamp merge-sort
- Different formats (e.g., syslog + sairedis) can coexist
- Loading screen shows all file names

### Stdin Detection

- `!isatty(stdin)` → pipe mode
- StdinLoader runs in background thread
- Auto-enables Follow mode
- Source field: `<stdin>`
- EOF → status bar shows `[EOF]`

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Multi-file support, Linux default syslog, stdin pipe input |
