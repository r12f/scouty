//! TUI component architecture — trait, dispatch, and component modules.

pub mod widgets;
pub mod windows;

// Re-export legacy rendering (will be migrated incrementally)
#[path = "../ui_legacy.rs"]
mod ui_legacy;
pub use ui_legacy::render;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;

/// Result from a component's input handling.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ComponentResult {
    /// Input was consumed, no further action needed.
    Consumed,
    /// Component requests to close itself.
    Close,
    /// Component did not handle this input.
    Ignored,
}

/// Common interface for all TUI components (windows and widgets).
///
/// Framework dispatches standard keys to typed callbacks.
/// Components override only the callbacks they care about.
pub trait UiComponent {
    /// Render the component into the given area.
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Whether this component uses j/k as navigation (default: true).
    /// Components that use j/k as shortcut keys should return false.
    fn enable_jk_navigation(&self) -> bool {
        true
    }

    // --- Standard navigation callbacks (default: no-op) ---

    fn on_up(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }
    fn on_down(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }
    fn on_page_up(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }
    fn on_page_down(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }

    // --- Action callbacks ---

    /// Space pressed — toggle selection.
    fn on_toggle(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }
    /// Enter pressed — confirm / submit.
    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Ignored
    }
    /// Esc pressed — cancel / close.
    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    // --- Text input ---

    /// Character typed (non-control).
    fn on_char(&mut self, _c: char) -> ComponentResult {
        ComponentResult::Ignored
    }

    /// Fallback for any key not matched by the framework.
    fn on_key(&mut self, _key: KeyEvent) -> ComponentResult {
        ComponentResult::Ignored
    }
}

/// Unified key dispatch: maps KeyEvent to the appropriate UiComponent callback.
///
/// Call this instead of matching keys inside each component.
pub fn dispatch_key(component: &mut dyn UiComponent, key: KeyEvent) -> ComponentResult {
    let jk_nav = component.enable_jk_navigation();
    match key.code {
        // Navigation — arrow keys always, j/k only if component opts in
        KeyCode::Up => component.on_up(),
        KeyCode::Down => component.on_down(),
        KeyCode::Char('k') if jk_nav && key.modifiers.is_empty() => component.on_up(),
        KeyCode::Char('j') if jk_nav && key.modifiers.is_empty() => component.on_down(),
        KeyCode::PageUp => component.on_page_up(),
        KeyCode::PageDown => component.on_page_down(),

        // Actions
        KeyCode::Char(' ') => component.on_toggle(),
        KeyCode::Enter => component.on_confirm(),
        KeyCode::Esc => component.on_cancel(),

        // Text input
        KeyCode::Char(c) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
            component.on_char(c)
        }

        // Fallback
        _ => component.on_key(key),
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    /// A test component that records which callbacks were invoked.
    struct Recorder {
        jk_nav: bool,
        last_call: &'static str,
    }

    impl Recorder {
        fn new(jk_nav: bool) -> Self {
            Self {
                jk_nav,
                last_call: "",
            }
        }
    }

    impl UiComponent for Recorder {
        fn render(&self, _frame: &mut Frame, _area: Rect) {}

        fn enable_jk_navigation(&self) -> bool {
            self.jk_nav
        }

        fn on_up(&mut self) -> ComponentResult {
            self.last_call = "up";
            ComponentResult::Consumed
        }
        fn on_down(&mut self) -> ComponentResult {
            self.last_call = "down";
            ComponentResult::Consumed
        }
        fn on_page_up(&mut self) -> ComponentResult {
            self.last_call = "page_up";
            ComponentResult::Consumed
        }
        fn on_page_down(&mut self) -> ComponentResult {
            self.last_call = "page_down";
            ComponentResult::Consumed
        }
        fn on_toggle(&mut self) -> ComponentResult {
            self.last_call = "toggle";
            ComponentResult::Consumed
        }
        fn on_confirm(&mut self) -> ComponentResult {
            self.last_call = "confirm";
            ComponentResult::Consumed
        }
        fn on_cancel(&mut self) -> ComponentResult {
            self.last_call = "cancel";
            ComponentResult::Close
        }
        fn on_char(&mut self, _c: char) -> ComponentResult {
            self.last_call = "char";
            ComponentResult::Consumed
        }
        fn on_key(&mut self, _key: KeyEvent) -> ComponentResult {
            self.last_call = "key";
            ComponentResult::Ignored
        }
    }

    #[test]
    fn test_arrow_keys_dispatch_to_navigation() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::Up));
        assert_eq!(r.last_call, "up");
        dispatch_key(&mut r, key(KeyCode::Down));
        assert_eq!(r.last_call, "down");
    }

    #[test]
    fn test_jk_dispatch_when_enabled() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::Char('j')));
        assert_eq!(r.last_call, "down");
        dispatch_key(&mut r, key(KeyCode::Char('k')));
        assert_eq!(r.last_call, "up");
    }

    #[test]
    fn test_jk_dispatch_to_char_when_disabled() {
        let mut r = Recorder::new(false);
        dispatch_key(&mut r, key(KeyCode::Char('j')));
        assert_eq!(r.last_call, "char");
        dispatch_key(&mut r, key(KeyCode::Char('k')));
        assert_eq!(r.last_call, "char");
    }

    #[test]
    fn test_space_toggle_enter_confirm_esc_cancel() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::Char(' ')));
        assert_eq!(r.last_call, "toggle");
        dispatch_key(&mut r, key(KeyCode::Enter));
        assert_eq!(r.last_call, "confirm");
        dispatch_key(&mut r, key(KeyCode::Esc));
        assert_eq!(r.last_call, "cancel");
    }

    #[test]
    fn test_page_up_down() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::PageUp));
        assert_eq!(r.last_call, "page_up");
        dispatch_key(&mut r, key(KeyCode::PageDown));
        assert_eq!(r.last_call, "page_down");
    }

    #[test]
    fn test_unknown_key_falls_to_on_key() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::F(1)));
        assert_eq!(r.last_call, "key");
    }

    #[test]
    fn test_regular_char_goes_to_on_char() {
        let mut r = Recorder::new(true);
        dispatch_key(&mut r, key(KeyCode::Char('x')));
        assert_eq!(r.last_call, "char");
    }
}
