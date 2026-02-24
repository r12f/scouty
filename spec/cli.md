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
| `--filter <expr>` | Apply filter expression (pipe mode, can repeat) |
| `--filter-preset <name>` | Load a saved filter preset from `~/.scouty/filters/<name>.yaml` |
| `--level <level>` | Minimum log level filter (pipe mode): trace/debug/info/notice/warn/error/fatal |
| `--format <fmt>` | Output format (pipe mode): `raw` (default), `json`, `yaml`, `csv` |
| `--fields <list>` | Comma-separated fields to include in structured output (pipe mode) |
| `--no-tui` | Force pipe mode even when stdout is a TTY |

### Pipe Output Mode (Non-interactive)

When stdout is not a TTY (piped to another command or file), or when `--no-tui` is specified, scouty runs in **pipe mode**: no TUI, just parse → filter → output to stdout. This exposes scouty's parser and filter engine for scripting and automation.

**Auto-detection:** `!isatty(stdout)` → pipe mode. Use `--no-tui` to force pipe mode even when stdout is a TTY.

> Note: `!isatty(stdin)` controls **input** (read from stdin pipe). `!isatty(stdout)` controls **output** (pipe mode). They are independent — you can pipe in AND out, or just one direction.

**Examples:**

```bash
# Filter errors from a log file, output as JSON
scouty-tui --filter 'level == "Error"' /var/log/syslog | jq .

# Chain with grep/awk — scouty parses, downstream processes
scouty-tui --level warn --format json app.log | jq '.message'

# Multiple filters (AND logic)
scouty-tui --filter 'component == "orchagent"' --filter 'level == "Error"' swss.log

# Specific fields only
scouty-tui --format csv --fields timestamp,level,message /var/log/syslog > filtered.csv

# Pipe in from another command
journalctl -u myservice | scouty-tui --level error --format json

# Force pipe mode to terminal (for quick inspection without TUI)
scouty-tui --no-tui --level error app.log

# Use a saved filter preset
scouty-tui --filter-preset my-error-filters --format json app.log

# Preset + additional filter (AND logic)
scouty-tui --filter-preset production-noise --filter 'component == "orchagent"' app.log

# SSH remote + pipe mode
scouty-tui --format json ssh://prod:/var/log/app.log | jq '.message'
```

**Output formats:**

| Format | Description |
|--------|-------------|
| `raw` | Original log line as-is (default) |
| `json` | One JSON object per line (NDJSON), all parsed fields |
| `yaml` | YAML document per record (separated by `---`) |
| `csv` | CSV with header row, all fields or `--fields` subset |

**`--fields` option** (for `json`, `yaml`, `csv`):
- Comma-separated list of field names: `timestamp,level,message,component,hostname,...`
- Special value `all` (default): include all non-empty fields
- Metadata keys accessible by name (e.g., `--fields timestamp,level,message,request_id`)
- `raw` format ignores `--fields` (always outputs the original line)

**`--filter-preset` option:**
- Loads all filters from `~/.scouty/filters/<name>.yaml` (same format as TUI filter presets)
- Includes both filter expressions and level filter defined in the preset
- Can be combined with `--filter` and `--level` (all combined with AND logic)
- `--level` on CLI overrides the preset's `level_filter` if both are specified
- Error if preset file not found: `Filter preset not found: ~/.scouty/filters/<name>.yaml`
- Also works in TUI mode: launches TUI with the preset's filters pre-applied

**`--filter` option:**
- Uses the same filter expression syntax as the TUI `f` key (see filter spec)
- Multiple `--filter` flags are combined with AND logic
- Applied after parsing, before output

**`--level` option:**
- Shorthand for level filtering: `--level warn` is equivalent to `--filter 'level >= "Warn"'`
- Values: `trace`, `debug`, `info`, `notice`, `warn`, `error`, `fatal` (case-insensitive)

**Behavior:**
- Records are output as they are parsed (streaming, not buffered to completion)
- Exit code 0 on success, non-zero on parse/IO error
- Stderr used for progress/error messages (e.g., `Parsed 10,000 records from 3 files`)
- Follow mode works in pipe mode: `cat /var/log/syslog | scouty-tui --format json` streams continuously
- Parser auto-detection works the same as TUI mode
- Config files (`~/.scouty/config.yaml`, `./scouty.yaml`) are still loaded for parser settings, but TUI-specific settings (theme, keybindings) are ignored

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
| 2026-02-24 | Pipe output mode: --filter, --level, --format, --fields, --no-tui for non-interactive use |
| 2026-02-24 | Added --filter-preset to load saved filter presets (works in both pipe and TUI mode) |
