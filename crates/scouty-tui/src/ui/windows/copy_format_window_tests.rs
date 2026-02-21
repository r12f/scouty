#[cfg(test)]
mod tests {
    use crate::ui::windows::copy_format_window::CopyFormatWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_esc_closes() {
        let mut w = CopyFormatWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_enter_closes() {
        let mut w = CopyFormatWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_r_j_y_close() {
        for c in ['r', 'j', 'y'] {
            let mut w = CopyFormatWindow;
            assert_eq!(
                dispatch_key(&mut w, key(KeyCode::Char(c))),
                ComponentResult::Close
            );
        }
    }

    #[test]
    fn test_unknown_key_ignored() {
        let mut w = CopyFormatWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('x'))),
            ComponentResult::Ignored
        );
    }

    #[test]
    fn test_navigation_ignored() {
        let mut w = CopyFormatWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Ignored
        );
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Down)),
            ComponentResult::Ignored
        );
    }
}
