#[cfg(test)]
mod tests {
    use crate::config::Theme;
    use crate::ui::windows::help_window::HelpWindow;
    use crate::ui::{dispatch_key, ComponentResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_esc_closes() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_q_closes() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('q'))),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_enter_closes() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_jk_scrolls() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        assert_eq!(w.scroll, 0);
        dispatch_key(&mut w, key(KeyCode::Char('j')));
        assert_eq!(w.scroll, 1);
        dispatch_key(&mut w, key(KeyCode::Char('j')));
        assert_eq!(w.scroll, 2);
        dispatch_key(&mut w, key(KeyCode::Char('k')));
        assert_eq!(w.scroll, 1);
    }

    #[test]
    fn test_scroll_doesnt_go_below_zero() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        dispatch_key(&mut w, key(KeyCode::Char('k')));
        assert_eq!(w.scroll, 0);
    }

    #[test]
    fn test_arrow_scrolls() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        dispatch_key(&mut w, key(KeyCode::Down));
        assert_eq!(w.scroll, 1);
        dispatch_key(&mut w, key(KeyCode::Up));
        assert_eq!(w.scroll, 0);
    }

    #[test]
    fn test_page_down_scrolls_by_visible_height() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        w.visible_height = 10;
        assert_eq!(w.scroll, 0);
        let result = dispatch_key(&mut w, key(KeyCode::PageDown));
        assert_eq!(result, ComponentResult::Consumed);
        assert_eq!(w.scroll, 10);
    }

    #[test]
    fn test_page_up_scrolls_by_visible_height() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        w.visible_height = 10;
        w.scroll = 15;
        let result = dispatch_key(&mut w, key(KeyCode::PageUp));
        assert_eq!(result, ComponentResult::Consumed);
        assert_eq!(w.scroll, 5);
    }

    #[test]
    fn test_page_up_clamps_to_zero() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        w.visible_height = 10;
        w.scroll = 3;
        dispatch_key(&mut w, key(KeyCode::PageUp));
        assert_eq!(w.scroll, 0);
    }

    #[test]
    fn test_page_down_clamps_to_max() {
        let theme = Theme::default();
        let mut w = HelpWindow::new(&theme);
        w.visible_height = 10;
        let max_scroll = w.total_lines().saturating_sub(w.visible_height);
        w.scroll = max_scroll.saturating_sub(3);
        dispatch_key(&mut w, key(KeyCode::PageDown));
        assert_eq!(w.scroll, max_scroll);
    }
}
