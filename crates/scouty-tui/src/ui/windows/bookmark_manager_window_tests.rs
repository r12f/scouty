#[cfg(test)]
mod tests {
    use crate::ui::windows::bookmark_manager_window::BookmarkManagerWindow;
    use crate::ui::{dispatch_key, ComponentResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_esc_closes() {
        let mut w = BookmarkManagerWindow {
            cursor: 0,
            entries: vec![],
            action: None,
            deleted_ids: vec![],
        };
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_navigation() {
        use crate::ui::windows::bookmark_manager_window::BookmarkEntry;
        let mut w = BookmarkManagerWindow {
            cursor: 0,
            entries: vec![
                BookmarkEntry {
                    filtered_idx: 0,
                    record_id: 1,
                    message: "msg1".into(),
                },
                BookmarkEntry {
                    filtered_idx: 5,
                    record_id: 2,
                    message: "msg2".into(),
                },
            ],
            action: None,
            deleted_ids: vec![],
        };
        dispatch_key(&mut w, key(KeyCode::Char('j')));
        assert_eq!(w.cursor, 1);
        dispatch_key(&mut w, key(KeyCode::Char('k')));
        assert_eq!(w.cursor, 0);
    }

    #[test]
    fn test_delete() {
        use crate::ui::windows::bookmark_manager_window::BookmarkEntry;
        let mut w = BookmarkManagerWindow {
            cursor: 0,
            entries: vec![BookmarkEntry {
                filtered_idx: 0,
                record_id: 42,
                message: "msg".into(),
            }],
            action: None,
            deleted_ids: vec![],
        };
        dispatch_key(&mut w, key(KeyCode::Char('d')));
        assert_eq!(w.entries.len(), 0);
        assert_eq!(w.deleted_ids, vec![42]);
    }

    #[test]
    fn test_enter_jumps() {
        use crate::ui::windows::bookmark_manager_window::{BookmarkAction, BookmarkEntry};
        let mut w = BookmarkManagerWindow {
            cursor: 0,
            entries: vec![BookmarkEntry {
                filtered_idx: 10,
                record_id: 1,
                message: "msg".into(),
            }],
            action: None,
            deleted_ids: vec![],
        };
        let result = dispatch_key(&mut w, key(KeyCode::Enter));
        assert_eq!(result, ComponentResult::Close);
        assert!(matches!(w.action, Some(BookmarkAction::Jump(10))));
    }
}
