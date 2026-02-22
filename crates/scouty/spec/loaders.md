# Loaders

## Overview

Loaders are responsible for reading raw log data from various sources and feeding lines into their associated Parser Groups. Each source/file gets an independent Loader instance; a session can have multiple Loaders.


## Design

### LogLoader Trait

All loaders implement the `LogLoader` trait, providing a uniform interface for the Log Session pipeline.

### Supported Loader Types

| Loader | Source | Notes |
|--------|--------|-------|
| FileLoader | Local text files | Plain text, line-by-line |
| CompressedLoader | gz, zip, 7z | Decompress + line-by-line |
| StdinLoader | Pipe/stdin input | `cat log \| scouty-tui` |
| SyslogLoader | Live syslog | Network syslog receiver |
| OtlpLoader | OpenTelemetry Logs | gRPC and HTTP OTLP server |

### FileLoader — Multi-file Support

- CLI: `scouty-tui file1 file2 file3 ...` — any number of file arguments
- Each file creates independent FileLoader + ParserGroup (auto-detected by parser factory)
- All records merge into one LogSession / LogStore (timestamp-sorted via `insert_batch`)

### StdinLoader

- Detects pipe: `!isatty(stdin)`
- Background thread reads stdin line-by-line → parser → LogStore
- On EOF: stop reading, status bar shows `[EOF]`
- Auto-enables Follow mode (streaming input)
- Coexists with file arguments: `cat extra.log | scouty-tui file1.log` — stdin and file both load into same LogStore
- Stdin source field: `<stdin>`

### Default Syslog (Linux)

- No arguments on Linux → default to `/var/log/syslog`
- If absent, try `/var/log/messages` (RHEL/CentOS)
- If both absent → friendly error + usage hint
- macOS: no default path (different syslog mechanism), show usage

### Multi-line Log Merging

- Enabled per-loader based on source type
- Parser config defines multi-line rules via regex for line start pattern
- Lines not matching start pattern merge into previous LogRecord
- Merged message retains full multi-line content

## Change Log

| Date | Change |
|------|--------|
| 2026-02-18 | Initial loader design (file, compressed, syslog, OTLP) |
| 2026-02-22 | Multi-file CLI support, Linux default syslog |
| 2026-02-22 | StdinLoader for pipe input |
