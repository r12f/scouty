# CI and Release

## Overview

GitHub Actions pipelines for continuous integration (build + test on every push/PR) and manual release workflow (version bump, multi-platform build, GitHub Release, crates.io publish).

## Current Status

✅ Implemented

## Design

### CI Pipeline

- **Trigger**: push to main, all PRs
- **Steps**: `cargo build` → `cargo test`
- **Toolchain**: stable Rust
- **P1**: `cargo clippy` lint, `cargo fmt` format check, multi-platform matrix (Linux + macOS + Windows)
- **P2**: Cargo registry/target caching, branch protection rules requiring CI pass

### Release Pipeline

- **Trigger**: `workflow_dispatch` (manual)
- **Flow**:
  1. Bump minor version in `Cargo.toml` (e.g., 0.1.0 → 0.2.0)
  2. Commit version change + push to main
  3. Create git tag (e.g., `v0.2.0`)
  4. Release build + full test suite
  5. Multi-platform build matrix:
     - Linux x86_64
     - macOS x86_64 + aarch64 (Apple Silicon)
     - Windows x86_64
  6. Create GitHub Release with platform binaries + auto-generated changelog
  7. `cargo publish` to crates.io

- **P1**: Configurable bump type (patch/minor/major), dry run mode, structured changelog from conventional commits
- **P2**: Binary compression (tar.gz/zip) + SHA256 checksums, release notification (Discord webhook)

### Prerequisites

- `CARGO_REGISTRY_TOKEN` in GitHub Actions secrets
- `Cargo.toml` metadata: description, license, repository

### Safety

- Same version cannot be published twice (idempotent)
- If publish fails, GitHub Release kept but marked as pre-release
- Token managed via GitHub Secrets, never exposed in logs

## Change Log

| Date | Change |
|------|--------|
| 2026-02-18 | CI pipeline spec (build + test) |
| 2026-02-19 | Release pipeline spec (multi-platform, crates.io) |
