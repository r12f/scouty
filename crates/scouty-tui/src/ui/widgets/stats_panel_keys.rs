//! Stats panel key handling and shortcut hints.
//!
//! The stats panel is read-only — no panel-specific keys.
//! Common panel keys (Tab/S-Tab to switch, z to maximize, Esc to close)
//! are handled by MainWindow and included in its hints.

use crate::app::App;
use crate::ui::framework::KeyAction;
use crossterm::event::KeyEvent;

/// Handle a key event when the stats panel has focus.
/// Stats is read-only, so all keys are unhandled (bubble up).
pub fn handle_key(_app: &mut App, _key: KeyEvent) -> KeyAction {
    KeyAction::Unhandled
}

/// Shortcut hints for the stats panel (empty — no panel-specific keys).
pub fn shortcut_hints() -> Vec<(&'static str, &'static str)> {
    vec![]
}
