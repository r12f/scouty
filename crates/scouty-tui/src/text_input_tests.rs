#[cfg(test)]
mod tests {
    use crate::text_input::TextInput;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn test_insert_and_value() {
        let mut ti = TextInput::new();
        ti.handle_key(key(KeyCode::Char('a')));
        ti.handle_key(key(KeyCode::Char('b')));
        ti.handle_key(key(KeyCode::Char('c')));
        assert_eq!(ti.value(), "abc");
        assert_eq!(ti.cursor_position(), 3);
    }

    #[test]
    fn test_backspace() {
        let mut ti = TextInput::with_text("abc");
        ti.handle_key(key(KeyCode::Backspace));
        assert_eq!(ti.value(), "ab");
        assert_eq!(ti.cursor_position(), 2);
    }

    #[test]
    fn test_ctrl_h_backspace() {
        let mut ti = TextInput::with_text("abc");
        ti.handle_key(ctrl_key(KeyCode::Char('h')));
        assert_eq!(ti.value(), "ab");
    }

    #[test]
    fn test_delete() {
        let mut ti = TextInput::with_text("abc");
        ti.home();
        ti.handle_key(key(KeyCode::Delete));
        assert_eq!(ti.value(), "bc");
        assert_eq!(ti.cursor_position(), 0);
    }

    #[test]
    fn test_left_right() {
        let mut ti = TextInput::with_text("abc");
        assert_eq!(ti.cursor_position(), 3);
        ti.handle_key(key(KeyCode::Left));
        assert_eq!(ti.cursor_position(), 2);
        ti.handle_key(key(KeyCode::Right));
        assert_eq!(ti.cursor_position(), 3);
    }

    #[test]
    fn test_home_end() {
        let mut ti = TextInput::with_text("abc");
        ti.handle_key(key(KeyCode::Home));
        assert_eq!(ti.cursor_position(), 0);
        ti.handle_key(key(KeyCode::End));
        assert_eq!(ti.cursor_position(), 3);
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut ti = TextInput::with_text("ac");
        ti.handle_key(key(KeyCode::Left)); // cursor at 1
        ti.handle_key(key(KeyCode::Char('b')));
        assert_eq!(ti.value(), "abc");
        assert_eq!(ti.cursor_position(), 2);
    }

    #[test]
    fn test_unicode() {
        let mut ti = TextInput::new();
        ti.handle_key(key(KeyCode::Char('你')));
        ti.handle_key(key(KeyCode::Char('好')));
        assert_eq!(ti.value(), "你好");
        assert_eq!(ti.cursor_position(), 2);
        ti.handle_key(key(KeyCode::Backspace));
        assert_eq!(ti.value(), "你");
    }

    #[test]
    fn test_clear() {
        let mut ti = TextInput::with_text("abc");
        ti.clear();
        assert_eq!(ti.value(), "");
        assert_eq!(ti.cursor_position(), 0);
    }

    #[test]
    fn test_boundary_no_panic() {
        let mut ti = TextInput::new();
        ti.handle_key(key(KeyCode::Backspace)); // no panic
        ti.handle_key(key(KeyCode::Delete)); // no panic
        ti.handle_key(key(KeyCode::Left)); // no panic
        assert_eq!(ti.cursor_position(), 0);
    }

    #[test]
    fn test_ctrl_char_not_inserted() {
        let mut ti = TextInput::new();
        let handled = ti.handle_key(ctrl_key(KeyCode::Char('c')));
        assert!(!handled);
        assert_eq!(ti.value(), "");
    }

    #[test]
    fn render_parts_cursor_at_end() {
        let ti = TextInput::with_text("abc");
        let (before, cursor, after) = ti.render_parts();
        assert_eq!(before, "abc");
        assert_eq!(cursor, " ");
        assert_eq!(after, "");
    }

    #[test]
    fn render_parts_cursor_in_middle() {
        let mut ti = TextInput::with_text("hello");
        ti.move_left();
        ti.move_left();
        let (before, cursor, after) = ti.render_parts();
        assert_eq!(before, "hel");
        assert_eq!(cursor, "l");
        assert_eq!(after, "o");
    }

    #[test]
    fn render_parts_cursor_at_start() {
        let mut ti = TextInput::with_text("ab");
        ti.home();
        let (before, cursor, after) = ti.render_parts();
        assert_eq!(before, "");
        assert_eq!(cursor, "a");
        assert_eq!(after, "b");
    }

    #[test]
    fn insert_at_cursor_middle() {
        let mut ti = TextInput::with_text("ac");
        ti.move_left(); // cursor before 'c'
        ti.insert('b');
        assert_eq!(ti.value(), "abc");
        assert_eq!(ti.cursor_position(), 2);
    }

    #[test]
    fn backspace_at_cursor_middle() {
        let mut ti = TextInput::with_text("abc");
        ti.move_left(); // cursor before 'c'
        ti.backspace(); // delete 'b'
        assert_eq!(ti.value(), "ac");
        assert_eq!(ti.cursor_position(), 1);
    }
}
