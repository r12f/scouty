# CLI

## Overview

Command-line interface for `scouty-tui`, handling file arguments, default syslog paths, and stdin pipe detection.


## Design

### Usage

```
scouty-tui [OPTIONS] [file1] [file2] ...
```

### Options

| Flag | Description |
|------|-------------|
| `--theme <name>` | Override theme selection |
| `--config <path>` | Load additional config file (overrides system and user configs) |
| `--generate-config` | Generate default config file to stdout |
| `--generate-theme <name>` | Generate a built-in theme file to stdout |

### Config Generation

**`--generate-config`** — prints the full default config to stdout with comments explaining each field.

```bash
# Generate and save as user config
scouty-tui --generate-config > ~/.scouty/config.yaml

# Generate to current directory as project config
scouty-tui --generate-config > ./scouty.yaml
```

Output includes all configurable fields with their default values and brief comments:

```yaml
# Scouty configuration file
# Place at ~/.scouty/config.yaml (user) or ./scouty.yaml (project)
# See: https://github.com/r12f/scouty

# Theme selection (built-in: default, dark, light, solarized, landmine)
theme: default

# Default log paths (glob supported)
default_paths: []

# Keybindings (uncomment to override)
# keybindings:
#   quit: "q"
#   search: "/"
#   filter: "f"
#   ...

# SSH settings
# ssh:
#   connect_timeout: 10
#   keepalive_interval: 30
```

**`--generate-theme <name>`** — prints the specified built-in theme's full YAML definition to stdout.

```bash
# Export the landmine theme as a starting point for customization
scouty-tui --generate-theme landmine > ~/.scouty/themes/my-theme.yaml

# List available built-in themes
scouty-tui --generate-theme list
```

When `name` is `list`, prints all available built-in theme names (one per line).

**Behavior:**
- Both commands print to stdout and exit immediately (TUI does not launch)
- Output is valid YAML that can be directly used as config/theme file
- All fields included with defaults, commented-out optional sections for discoverability
- Exit code 0 on success, non-zero if theme name is unknown

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
| 2026-02-23 | Added --theme and --config CLI flags |
| 2026-02-24 | Added --generate-config and --generate-theme for default config generation |
