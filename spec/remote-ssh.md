# Remote Log Reading (SSH)

## Overview

Support reading log files from remote hosts via SSH, allowing users to view remote logs without manually copying files.

## Design

### Usage

```
scouty-tui ssh://user@host:/path/to/logfile
scouty-tui ssh://user@host:22:/var/log/syslog
scouty-tui ssh://host:/var/log/syslog              # uses current user
scouty-tui local.log ssh://prod:/var/log/app.log   # mixed local + remote
```

### URL Format

```
ssh://[user@]host[:port]:/absolute/path
```

- `user` — optional, defaults to current system user
- `host` — hostname or IP
- `port` — optional, defaults to 22
- `/absolute/path` — path on the remote host (must be absolute)

### Connection

- Use system SSH config (`~/.ssh/config`) for host aliases, identity files, proxy jumps, etc.
- Authentication: relies on SSH agent / key-based auth (no interactive password prompt in TUI)
- Connection failure: show friendly error with details (e.g., `SSH: Connection refused to prod:22`)
- Timeout: configurable in `config.yaml`, default 10s

### Data Flow

1. Establish SSH connection
2. Execute `cat <path>` (or `tail -f <path>` in follow mode) on remote host
3. Stream stdout into the same LogStore pipeline as local files
4. Source field: `ssh://user@host:/path`

### Follow Mode

- When follow mode is active (`Ctrl+]`), use `tail -f` instead of `cat`
- SSH connection stays open, streaming new records in real-time
- Connection drop: show `[SSH DISCONNECTED]` in status bar, auto-retry with backoff

### Multi-source

- Remote sources can be mixed with local files and stdin
- Each source gets independent loader, all merge into shared LogStore
- Loading screen shows all sources (local and remote)

### Configuration

```yaml
# In config.yaml
ssh:
  connect_timeout: 10       # seconds
  keepalive_interval: 30    # seconds, 0 to disable
```

## Acceptance Criteria

- [ ] `ssh://` URL scheme recognized and parsed correctly
- [ ] Remote log loaded via SSH and displayed in log table
- [ ] System SSH config respected (host aliases, keys, proxy)
- [ ] Follow mode works with remote sources (tail -f)
- [ ] Connection errors shown as friendly messages
- [ ] Mixed local + remote sources work in same session
- [ ] Source field shows full SSH URL
- [ ] Glob patterns in `default_paths` do NOT apply to SSH URLs

## Change Log

| Date | Change |
|------|--------|
| 2026-02-24 | Initial remote log reading via SSH |
