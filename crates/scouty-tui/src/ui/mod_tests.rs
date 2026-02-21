mod tests {
    use crate::ui::*;
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
        let result = dispatch_key(&mut r, key(KeyCode::Up));
        assert_eq!(r.last_call, "up");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::Down));
        assert_eq!(r.last_call, "down");
        assert_eq!(result, ComponentResult::Consumed);
    }

    #[test]
    fn test_jk_dispatch_when_enabled() {
        let mut r = Recorder::new(true);
        let result = dispatch_key(&mut r, key(KeyCode::Char('j')));
        assert_eq!(r.last_call, "down");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::Char('k')));
        assert_eq!(r.last_call, "up");
        assert_eq!(result, ComponentResult::Consumed);
    }

    #[test]
    fn test_jk_dispatch_to_char_when_disabled() {
        let mut r = Recorder::new(false);
        let result = dispatch_key(&mut r, key(KeyCode::Char('j')));
        assert_eq!(r.last_call, "char");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::Char('k')));
        assert_eq!(r.last_call, "char");
        assert_eq!(result, ComponentResult::Consumed);
    }

    #[test]
    fn test_space_toggle_enter_confirm_esc_cancel() {
        let mut r = Recorder::new(true);
        let result = dispatch_key(&mut r, key(KeyCode::Char(' ')));
        assert_eq!(r.last_call, "toggle");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::Enter));
        assert_eq!(r.last_call, "confirm");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::Esc));
        assert_eq!(r.last_call, "cancel");
        assert_eq!(result, ComponentResult::Close);
    }

    #[test]
    fn test_page_up_down() {
        let mut r = Recorder::new(true);
        let result = dispatch_key(&mut r, key(KeyCode::PageUp));
        assert_eq!(r.last_call, "page_up");
        assert_eq!(result, ComponentResult::Consumed);
        let result = dispatch_key(&mut r, key(KeyCode::PageDown));
        assert_eq!(r.last_call, "page_down");
        assert_eq!(result, ComponentResult::Consumed);
    }

    #[test]
    fn test_unknown_key_falls_to_on_key() {
        let mut r = Recorder::new(true);
        let result = dispatch_key(&mut r, key(KeyCode::F(1)));
        assert_eq!(r.last_call, "key");
        assert_eq!(result, ComponentResult::Ignored);
    }

    #[test]
    fn test_regular_char_goes_to_on_char() {
        let mut r = Recorder::new(true);
        let result = dispatch_key(&mut r, key(KeyCode::Char('x')));
        assert_eq!(r.last_call, "char");
        assert_eq!(result, ComponentResult::Consumed);
    }
}
