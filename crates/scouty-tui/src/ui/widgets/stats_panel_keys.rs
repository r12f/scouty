//! Stats panel shortcut hints.
//!
//! The stats panel has no interactive keys — it's read-only.
//! Common panel keys (Tab/S-Tab to switch, z to maximize, Esc to close)
//! are handled by MainWindow and included in its hints.

/// Shortcut hints for the stats panel (empty — no panel-specific keys).
pub fn shortcut_hints() -> Vec<(&'static str, &'static str)> {
    vec![]
}
