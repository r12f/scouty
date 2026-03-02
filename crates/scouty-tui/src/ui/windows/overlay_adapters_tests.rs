#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::ui::framework::{OverlayStack, OverlayWindow, WindowAction};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::layout::Rect;
    use ratatui::Frame;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn test_app() -> App {
        App::load_stdin(Vec::new()).unwrap()
    }

    /// Minimal overlay for testing.
    struct TestOverlay {
        close_on_esc: bool,
    }

    impl OverlayWindow for TestOverlay {
        fn name(&self) -> &str {
            "TestOverlay"
        }

        fn render(&self, _frame: &mut Frame, _area: Rect, _app: &App) {}

        fn handle_key(&mut self, _app: &mut App, key: KeyEvent) -> WindowAction {
            if self.close_on_esc && key.code == KeyCode::Esc {
                WindowAction::Close
            } else {
                WindowAction::Handled
            }
        }

        fn shortcut_hints(&self) -> Vec<(&str, &str)> {
            vec![("Esc", "Close")]
        }
    }

    #[test]
    fn test_overlay_stack_empty() {
        let stack = OverlayStack::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_overlay_stack_push_pop() {
        let mut stack = OverlayStack::new();
        stack.push(Box::new(TestOverlay { close_on_esc: true }));
        assert!(!stack.is_empty());

        let popped = stack.pop();
        assert!(popped.is_some());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_overlay_stack_dispatch_handles_key() {
        let mut stack = OverlayStack::new();
        let mut app = test_app();
        stack.push(Box::new(TestOverlay { close_on_esc: true }));

        // Non-Esc key → handled, overlay stays
        let handled = stack.handle_key(&mut app, key(KeyCode::Char('j')));
        assert!(handled);
        assert!(!stack.is_empty());
    }

    #[test]
    fn test_overlay_stack_dispatch_close() {
        let mut stack = OverlayStack::new();
        let mut app = test_app();
        stack.push(Box::new(TestOverlay { close_on_esc: true }));

        // Esc → Close → pop
        let handled = stack.handle_key(&mut app, key(KeyCode::Esc));
        assert!(handled);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_overlay_stack_empty_dispatch() {
        let mut stack = OverlayStack::new();
        let mut app = test_app();

        // Empty stack → not handled
        let handled = stack.handle_key(&mut app, key(KeyCode::Char('j')));
        assert!(!handled);
    }

    #[test]
    fn test_overlay_hints() {
        let mut stack = OverlayStack::new();
        stack.push(Box::new(TestOverlay { close_on_esc: true }));
        let hints = stack.top().unwrap().shortcut_hints();
        assert_eq!(hints, vec![("Esc", "Close")]);
    }

    // ── HelpOverlay tests ───────────────────────────────────────────

    use crate::ui::windows::help_window::HelpOverlay;

    #[test]
    fn test_help_overlay_scroll() {
        let app = test_app();
        let mut overlay = HelpOverlay::new(&app);
        let mut app = app;

        // Scroll down
        overlay.handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(overlay.scroll, 1);

        // Scroll back up
        overlay.handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(overlay.scroll, 0);
    }

    #[test]
    fn test_help_overlay_close_on_esc() {
        let app = test_app();
        let mut overlay = HelpOverlay::new(&app);
        let mut app = app;
        app.input_mode = crate::app::InputMode::Help;

        let action = overlay.handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(action, WindowAction::Close);
        assert_eq!(app.input_mode, crate::app::InputMode::Normal);
    }

    #[test]
    fn test_help_overlay_close_on_q() {
        let app = test_app();
        let mut overlay = HelpOverlay::new(&app);
        let mut app = app;
        app.input_mode = crate::app::InputMode::Help;

        let action = overlay.handle_key(&mut app, key(KeyCode::Char('q')));
        assert_eq!(action, WindowAction::Close);
    }

    #[test]
    fn test_help_overlay_hints() {
        let app = test_app();
        let overlay = HelpOverlay::new(&app);
        let hints = overlay.shortcut_hints();
        assert_eq!(hints, vec![("j/k/PgUp/PgDn", "Scroll"), ("Esc/q", "Close")]);
    }
}
