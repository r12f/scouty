# SONiC Port Operation Regions - High-Level Spec

## Background & Goals

SONiC port operations (boot init, admin up/down, link flap, etc.) span multiple components (portsyncd, orchagent, syncd/SAI) and produce logs across several seconds to minutes. Currently, users must manually correlate these log entries to understand the full lifecycle of a port operation.

By defining **pre-built region definitions** for common SONiC port operations, scouty can automatically detect, correlate, and visualize these operation lifecycles — giving users instant visibility into port startup times, admin state transitions, and link flap recovery durations.

### Reference

- Port startup flow analysis: [2026-02-24-sonic-port-startup.md](https://github.com/r12f/bunny-house/blob/main/analysis/2026-02-24-sonic-port-startup.md)
- Log pattern research: [2026-03-02-sonic-port-region-log-patterns.md](https://github.com/r12f/bunny-house/blob/main/analysis/2026-03-02-sonic-port-region-log-patterns.md)

## Problem Statement

- Port operations in SONiC span multiple daemons and log messages — hard to manually correlate start/end
- No automated way to measure port boot init time, admin up latency, or link flap recovery duration
- Debugging slow port startup or link instability requires tedious manual log searching

## User Stories

- As a SONiC network engineer, I want to see how long each port takes from initialization to oper up during a cold boot, so I can identify slow ports
- As a SONiC network engineer, I want to see the admin up → oper up latency for each port, so I can detect PHY/SerDes training issues
- As a SONiC network engineer, I want to automatically detect link flaps and see their recovery time, so I can identify unstable links
- As a SONiC network engineer, I want to see the overall system port init duration (PortConfigDone → PortInitDone), so I can benchmark boot performance

## Requirements Breakdown

### P0 — Must Have

- [ ] **Region 1: Port Boot Init** — Track per-port cold boot initialization from `Initialized port` to first oper state change (dependency: none)
  - Start: `Initialized port <port_name>` (orchagent NOTICE)
  - End: `Port <port_name> oper state set from ... to up` OR `to down` (link not connected)
  - Correlate by: port name
  - Timeout: 120s
  - Note: end_points include both `to up` and `to down` because ports without a connected peer will transition to oper down after admin up

- [ ] **Region 2: Port Admin Up** — Track runtime admin up to oper up latency (dependency: none)
  - Start: `Set admin status UP host_tx_ready to true for port <port_name>` (orchagent NOTICE)
  - End: `Port <port_name> oper state set from ... to up`
  - Correlate by: port name
  - Timeout: 60s
  - Note: during cold boot, this region will also trigger (nested within Boot Init) — this is intentional, as it measures a different metric (physical link establishment time vs full init time)

- [ ] **Region 3: Port Admin Down** — Track admin down to oper down transition (dependency: none)
  - Start: `Set admin status DOWN host_tx_ready to false for port <port_name>` (orchagent NOTICE)
  - End: `Port <port_name> oper state set from ... to down`
  - Correlate by: port name
  - Timeout: 10s

### P1 — Should Have

- [ ] **Region 4: Port Link Flap** — Track link down to recovery (dependency: none)
  - Start: `Port <port_name> oper state set from up to down`
  - End: `Port <port_name> oper state set from ... to up`
  - Correlate by: port name
  - Timeout: 300s (5 min)
  - Known limitation: admin-initiated shutdowns will also trigger this region (the oper state change log does not carry the trigger reason). Users can distinguish by looking for a co-occurring Admin Down region on the same port.

- [ ] **Region 5: System Port Init** — Track system-level boot completion (dependency: none)
  - Start: `PortConfigDone` (orchagent INFO, portsyncd NOTICE)
  - End: `PortInitDone` (orchagent INFO, portsyncd NOTICE)
  - Correlate by: none (system-wide, only one instance per boot)
  - Timeout: 300s

### P2 — Nice to Have

- [ ] **Region 6: Port Attribute Change** — Track FEC/attribute change cycle (dependency: none)
  - Start: `Set port <port_name> FEC mode` (orchagent NOTICE)
  - End: `Port <port_name> oper state set from ... to up`
  - Correlate by: port name
  - Timeout: 60s
  - Known limitation: only covers FEC changes. Speed changes do not have a distinct NOTICE-level log message in the current swss codebase. This may be expanded in the future as more attribute change signals are identified.

## Functional Requirements

### Delivery Format

These region definitions are delivered as a **built-in preset YAML configuration file** that ships with scouty. The file is installed to the system config directory and loaded automatically.

- File path: installed as part of scouty's default region configs (e.g., `/etc/scouty/regions/sonic-port-operations.yaml` or bundled in the binary's embedded defaults)
- All 6 region definitions are in a single YAML file
- Users can override or disable individual regions by creating a same-named region in their user/project config (higher precedence)

### Region Definition Format

Each region follows scouty's existing YAML format:
- `start_points`: filter expression + regex with named capture groups
- `end_points`: filter expression + regex with named capture groups (multiple end_points for different outcomes)
- `correlate`: list of metadata field names that must match between start and end
- `template`: name and description templates with `{field}` substitution
- `timeout` + `timeout_reason`: max duration before auto-closing the region

### Log Message Matching

All filter expressions use `message contains "..."` (not exact match) to accommodate:
- syslog format variations (different timestamp formats, hostname, PID)
- potential function name prefixes in the message field (`:- doPortTask:`)
- future log message changes that preserve the key substring

Regex patterns extract the port name using `(?P<port>Ethernet\d+)` to support standard SONiC port naming.

### Overlap Behavior

Multiple regions can be active simultaneously for the same port:
- **Boot Init + Admin Up**: During cold boot, both regions trigger. Boot Init tracks the full init lifecycle; Admin Up tracks physical link establishment. This is correct and intentional.
- **Admin Down + Link Flap**: An admin shutdown triggers both regions. The Admin Down region closes quickly (oper down follows admin down almost immediately). The Link Flap region also starts but will not find a matching `oper up` end — it will either timeout or close when the port is brought back up.

## Non-Functional Requirements

- **Performance**: Region matching adds negligible overhead — each region definition requires one `contains` check per log record (short-circuit on filter mismatch)
- **Compatibility**: Log patterns are based on `sonic-swss` orchagent/portsyncd source code (portsorch.cpp, portsyncd.cpp). Patterns should be compatible with current and recent SONiC releases. Future swss log format changes may require updating the region definitions.
- **No code changes required**: This feature is purely a configuration addition — the existing region system handles all matching, correlation, and display

## Acceptance Criteria

- [ ] A YAML file containing all 6 region definitions loads without errors
- [ ] Port Boot Init region correctly correlates `Initialized port Ethernet0` with `Port Ethernet0 oper state set from down to up` (or `to down`)
- [ ] Port Admin Up region correctly measures time from admin up to oper up
- [ ] Port Admin Down region correctly measures time from admin down to oper down
- [ ] Port Link Flap region detects oper down → oper up transitions
- [ ] System Port Init region detects PortConfigDone → PortInitDone system-wide
- [ ] Port Attribute Change region detects FEC change → oper up
- [ ] Each region's timeout produces a timed-out region with the correct timeout_reason
- [ ] Regions display correctly in the Region panel with proper names (e.g., "Boot Init Ethernet0", "Admin Up Ethernet4")
- [ ] Multiple regions can be active simultaneously for the same port without interference

## Out of Scope

- CMIS/xcvrd pluggable optics module initialization flow (separate daemon, different log patterns)
- Port breakout operations (port delete + recreate, significantly different lifecycle)
- Warm boot port recovery (ports stay oper up during warm boot, no region needed)
- Fast boot / fast reboot port recovery
- Custom user-defined port operation regions (users can already create these via the existing region config system)
- Speed change detection (no distinct NOTICE-level log in current swss)

## Open Questions

None — all questions resolved through research:
- Log patterns confirmed from source code (high confidence)
- Region YAML format confirmed from scouty codebase
- Overlap behavior analyzed and documented as acceptable
