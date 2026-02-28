#[cfg(test)]
mod tests {
    use crate::ui::framework::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::layout::Rect;
    use ratatui::Frame;

    // ── Test helpers ─────────────────────────────────────────────

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    /// A simple leaf widget for testing.
    struct TestWidget {
        name: &'static str,
        focusable: bool,
        /// Keys this widget handles (returns Handled for these).
        handled_keys: Vec<KeyCode>,
        children: Vec<Box<dyn Widget>>,
    }

    impl TestWidget {
        fn leaf(name: &'static str, focusable: bool, handled_keys: Vec<KeyCode>) -> Self {
            Self {
                name,
                focusable,
                handled_keys,
                children: Vec::new(),
            }
        }

        fn with_children(
            name: &'static str,
            focusable: bool,
            handled_keys: Vec<KeyCode>,
            children: Vec<Box<dyn Widget>>,
        ) -> Self {
            Self {
                name,
                focusable,
                handled_keys,
                children,
            }
        }
    }

    impl Widget for TestWidget {
        fn children(&self) -> &[Box<dyn Widget>] {
            &self.children
        }
        fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
            &mut self.children
        }
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
        fn handle_key(&mut self, event: KeyEvent) -> KeyAction {
            if self.handled_keys.contains(&event.code) {
                KeyAction::Handled
            } else {
                KeyAction::Unhandled
            }
        }
        fn is_focusable(&self) -> bool {
            self.focusable
        }
        fn name(&self) -> &str {
            self.name
        }
    }

    /// A simple window for testing.
    struct TestWindow {
        name: &'static str,
        action_on_esc: bool,
    }

    impl TestWindow {
        fn new(name: &'static str, action_on_esc: bool) -> Self {
            Self {
                name,
                action_on_esc,
            }
        }
    }

    impl Window for TestWindow {
        fn name(&self) -> &str {
            self.name
        }
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
        fn handle_key(&mut self, event: KeyEvent) -> WindowAction {
            if self.action_on_esc && event.code == KeyCode::Esc {
                WindowAction::Close
            } else {
                WindowAction::Unhandled
            }
        }
    }

    // ── WindowStack tests ────────────────────────────────────────

    #[test]
    fn window_stack_base_window() {
        let base = Box::new(TestWindow::new("Main", false));
        let stack = WindowStack::new(base);
        assert_eq!(stack.len(), 1);
        assert!(stack.is_base_only());
        assert_eq!(stack.top().name(), "Main");
    }

    #[test]
    fn window_stack_push_pop() {
        let base = Box::new(TestWindow::new("Main", false));
        let mut stack = WindowStack::new(base);

        stack.push(Box::new(TestWindow::new("Help", true)));
        assert_eq!(stack.len(), 2);
        assert!(!stack.is_base_only());
        assert_eq!(stack.top().name(), "Help");

        let popped = stack.pop();
        assert!(popped.is_some());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.top().name(), "Main");
    }

    #[test]
    fn window_stack_cannot_pop_base() {
        let base = Box::new(TestWindow::new("Main", false));
        let mut stack = WindowStack::new(base);

        let result = stack.pop();
        assert!(result.is_none());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn window_stack_handle_key_close() {
        let base = Box::new(TestWindow::new("Main", false));
        let mut stack = WindowStack::new(base);
        stack.push(Box::new(TestWindow::new("Help", true)));

        assert_eq!(stack.len(), 2);

        // Esc should close the Help window
        let result = stack.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.top().name(), "Main");
    }

    /// A window that returns Open on '?' key to push a Help window.
    struct OpenerWindow;
    impl Window for OpenerWindow {
        fn name(&self) -> &str {
            "Opener"
        }
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
        fn handle_key(&mut self, event: KeyEvent) -> WindowAction {
            if event.code == KeyCode::Char('?') {
                WindowAction::Open(Box::new(TestWindow::new("Help", true)))
            } else {
                WindowAction::Handled
            }
        }
    }

    #[test]
    fn window_stack_handle_key_open() {
        let base = Box::new(OpenerWindow);
        let mut stack = WindowStack::new(base);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.top().name(), "Opener");

        // '?' should open Help window on top
        let result = stack.handle_key(key(KeyCode::Char('?')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.top().name(), "Help");

        // Esc closes Help, back to Opener
        let result = stack.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.top().name(), "Opener");
    }

    #[test]
    fn window_stack_input_only_goes_to_top() {
        let base = Box::new(TestWindow::new("Main", false));
        let mut stack = WindowStack::new(base);
        stack.push(Box::new(TestWindow::new("Overlay", true)));

        // Input goes to Overlay, not Main
        assert_eq!(stack.top().name(), "Overlay");
    }

    #[test]
    fn window_stack_multiple_overlays() {
        let base = Box::new(TestWindow::new("Main", false));
        let mut stack = WindowStack::new(base);
        stack.push(Box::new(TestWindow::new("Help", true)));
        stack.push(Box::new(TestWindow::new("Confirm", true)));

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.top().name(), "Confirm");

        stack.handle_key(key(KeyCode::Esc)); // close Confirm
        assert_eq!(stack.top().name(), "Help");

        stack.handle_key(key(KeyCode::Esc)); // close Help
        assert_eq!(stack.top().name(), "Main");
    }

    // ── FocusManager tests ───────────────────────────────────────

    #[test]
    fn focus_manager_tab_cycles_focusable_children() {
        let root = TestWidget::with_children(
            "root",
            false,
            vec![],
            vec![
                Box::new(TestWidget::leaf("A", true, vec![])),
                Box::new(TestWidget::leaf("B", false, vec![])), // not focusable
                Box::new(TestWidget::leaf("C", true, vec![])),
            ],
        );

        let mut fm = FocusManager::with_path(vec![0]); // focused on A
        assert_eq!(fm.path(), &[0]);

        // Tab → skip B (not focusable) → C
        fm.next(&root);
        assert_eq!(fm.path(), &[2]);

        // Tab → wrap to A
        fm.next(&root);
        assert_eq!(fm.path(), &[0]);
    }

    #[test]
    fn focus_manager_shift_tab_reverses() {
        let root = TestWidget::with_children(
            "root",
            false,
            vec![],
            vec![
                Box::new(TestWidget::leaf("A", true, vec![])),
                Box::new(TestWidget::leaf("B", true, vec![])),
                Box::new(TestWidget::leaf("C", true, vec![])),
            ],
        );

        let mut fm = FocusManager::with_path(vec![0]); // focused on A

        // Shift+Tab → wrap to C (reverse)
        fm.prev(&root);
        assert_eq!(fm.path(), &[2]);

        // Shift+Tab → B
        fm.prev(&root);
        assert_eq!(fm.path(), &[1]);

        // Shift+Tab → A
        fm.prev(&root);
        assert_eq!(fm.path(), &[0]);
    }

    #[test]
    fn focus_manager_no_focusable_widgets() {
        let root = TestWidget::with_children(
            "root",
            false,
            vec![],
            vec![Box::new(TestWidget::leaf("A", false, vec![]))],
        );

        let mut fm = FocusManager::new();
        assert!(!fm.next(&root));
        assert!(!fm.prev(&root));
    }

    #[test]
    fn focus_manager_nested_focusable() {
        let root = TestWidget::with_children(
            "root",
            false,
            vec![],
            vec![
                Box::new(TestWidget::leaf("A", true, vec![])),
                Box::new(TestWidget::with_children(
                    "Panel",
                    false,
                    vec![],
                    vec![
                        Box::new(TestWidget::leaf("B", true, vec![])),
                        Box::new(TestWidget::leaf("C", true, vec![])),
                    ],
                )),
            ],
        );

        let mut fm = FocusManager::with_path(vec![0]); // A

        // Tab → B (nested)
        fm.next(&root);
        assert_eq!(fm.path(), &[1, 0]);

        // Tab → C (nested)
        fm.next(&root);
        assert_eq!(fm.path(), &[1, 1]);

        // Tab → wrap to A
        fm.next(&root);
        assert_eq!(fm.path(), &[0]);
    }

    // ── Event bubbling tests ─────────────────────────────────────

    #[test]
    fn event_bubbling_child_handles() {
        let mut root = TestWidget::with_children(
            "root",
            false,
            vec![KeyCode::Char('q')],
            vec![Box::new(TestWidget::leaf(
                "Child",
                true,
                vec![KeyCode::Char('j')],
            ))],
        );

        let fm = FocusManager::with_path(vec![0]); // focused on Child

        // 'j' → Child handles it
        let result = fm.dispatch_key(&mut root, key(KeyCode::Char('j')));
        assert_eq!(result, KeyAction::Handled);
    }

    #[test]
    fn event_bubbling_child_unhandled_parent_handles() {
        let mut root = TestWidget::with_children(
            "root",
            false,
            vec![KeyCode::Char('q')],
            vec![Box::new(TestWidget::leaf(
                "Child",
                true,
                vec![KeyCode::Char('j')],
            ))],
        );

        let fm = FocusManager::with_path(vec![0]); // focused on Child

        // 'q' → Child doesn't handle, bubbles to root → root handles
        let result = fm.dispatch_key(&mut root, key(KeyCode::Char('q')));
        assert_eq!(result, KeyAction::Handled);
    }

    #[test]
    fn event_bubbling_nobody_handles() {
        let mut root = TestWidget::with_children(
            "root",
            false,
            vec![KeyCode::Char('q')],
            vec![Box::new(TestWidget::leaf(
                "Child",
                true,
                vec![KeyCode::Char('j')],
            ))],
        );

        let fm = FocusManager::with_path(vec![0]);

        // 'x' → nobody handles
        let result = fm.dispatch_key(&mut root, key(KeyCode::Char('x')));
        assert_eq!(result, KeyAction::Unhandled);
    }

    #[test]
    fn event_bubbling_nested() {
        // root(handles 'q') → Panel(handles 'p') → Leaf(handles 'j')
        let mut root = TestWidget::with_children(
            "root",
            false,
            vec![KeyCode::Char('q')],
            vec![Box::new(TestWidget::with_children(
                "Panel",
                false,
                vec![KeyCode::Char('p')],
                vec![Box::new(TestWidget::leaf(
                    "Leaf",
                    true,
                    vec![KeyCode::Char('j')],
                ))],
            ))],
        );

        let fm = FocusManager::with_path(vec![0, 0]); // focused on Leaf

        // 'j' → Leaf handles
        assert_eq!(
            fm.dispatch_key(&mut root, key(KeyCode::Char('j'))),
            KeyAction::Handled
        );

        // 'p' → Leaf unhandled → Panel handles
        assert_eq!(
            fm.dispatch_key(&mut root, key(KeyCode::Char('p'))),
            KeyAction::Handled
        );

        // 'q' → Leaf unhandled → Panel unhandled → root handles
        assert_eq!(
            fm.dispatch_key(&mut root, key(KeyCode::Char('q'))),
            KeyAction::Handled
        );

        // 'x' → nobody handles
        assert_eq!(
            fm.dispatch_key(&mut root, key(KeyCode::Char('x'))),
            KeyAction::Unhandled
        );
    }

    // ── KeyAction / WindowAction tests ───────────────────────────

    #[test]
    fn key_action_equality() {
        assert_eq!(KeyAction::Handled, KeyAction::Handled);
        assert_ne!(KeyAction::Handled, KeyAction::Unhandled);
    }

    #[test]
    fn window_action_equality() {
        assert_eq!(WindowAction::Handled, WindowAction::Handled);
        assert_eq!(WindowAction::Close, WindowAction::Close);
        assert_ne!(WindowAction::Handled, WindowAction::Close);
    }

    // ── TabbedContainer tests ───────────────────────────────────────

    fn make_tab_entry(name: &str, shortcut: Option<char>, handles: Option<KeyCode>) -> TabEntry {
        TabEntry {
            name: name.to_string(),
            shortcut,
            widget: Box::new(TestWidget {
                name: Box::leak(name.to_string().into_boxed_str()),
                focusable: true,
                handled_keys: handles.into_iter().collect(),
                children: vec![],
            }),
        }
    }

    fn make_test_tabbed() -> TabbedContainer {
        TabbedContainer::new(vec![
            make_tab_entry("Detail", Some('d'), Some(KeyCode::Char('j'))),
            make_tab_entry("Region", Some('r'), Some(KeyCode::Char('k'))),
            make_tab_entry("Stats", Some('S'), None),
        ])
    }

    #[test]
    fn tabbed_container_next_prev() {
        let mut tc = make_test_tabbed();
        assert_eq!(tc.active, 0);
        assert_eq!(tc.active_name(), "Detail");

        tc.next_tab();
        assert_eq!(tc.active, 1);
        assert_eq!(tc.active_name(), "Region");

        tc.next_tab();
        assert_eq!(tc.active, 2);

        tc.next_tab(); // wrap
        assert_eq!(tc.active, 0);

        tc.prev_tab(); // wrap back
        assert_eq!(tc.active, 2);

        tc.prev_tab();
        assert_eq!(tc.active, 1);
    }

    #[test]
    fn tabbed_container_ctrl_arrows_removed() {
        let mut tc = make_test_tabbed();

        let ctrl_right = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);
        let ctrl_left = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);

        // Ctrl+Arrow should no longer be handled
        assert_eq!(tc.handle_key(ctrl_right), KeyAction::Unhandled);
        assert_eq!(tc.handle_key(ctrl_left), KeyAction::Unhandled);
    }

    #[test]
    fn tabbed_container_shortcut_keys() {
        let mut tc = make_test_tabbed();

        // 'r' activates Region
        let r_key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(tc.handle_key(r_key), KeyAction::Handled);
        assert_eq!(tc.active, 1);

        // 'd' activates Detail
        let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        assert_eq!(tc.handle_key(d_key), KeyAction::Handled);
        assert_eq!(tc.active, 0);

        // 'S' activates Stats (shift char)
        let s_key = KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT);
        assert_eq!(tc.handle_key(s_key), KeyAction::Handled);
        assert_eq!(tc.active, 2);
    }

    #[test]
    fn tabbed_container_delegates_to_active() {
        let mut tc = make_test_tabbed();
        // Active = Detail (handles 'j')
        let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(tc.handle_key(j_key), KeyAction::Handled);

        // 'k' is not handled by Detail
        let k_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(tc.handle_key(k_key), KeyAction::Unhandled);

        // Switch to Region, now 'k' handled
        tc.next_tab();
        assert_eq!(tc.handle_key(k_key), KeyAction::Handled);
    }

    #[test]
    fn tabbed_container_is_focusable() {
        let tc = make_test_tabbed();
        assert!(tc.is_focusable());
    }

    #[test]
    fn tabbed_container_tab_count() {
        let tc = make_test_tabbed();
        assert_eq!(tc.tab_count(), 3);
    }

    // ── Shortcut hints collection tests ─────────────────────────────

    /// A widget that returns specific shortcut hints.
    struct HintWidget {
        name: &'static str,
        hints: Vec<(&'static str, &'static str)>,
        children: Vec<Box<dyn Widget>>,
    }

    impl Widget for HintWidget {
        fn children(&self) -> &[Box<dyn Widget>] {
            &self.children
        }
        fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
            &mut self.children
        }
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
        fn handle_key(&mut self, _event: KeyEvent) -> KeyAction {
            KeyAction::Unhandled
        }
        fn is_focusable(&self) -> bool {
            true
        }
        fn name(&self) -> &str {
            self.name
        }
        fn shortcut_hints(&self) -> Vec<(&str, &str)> {
            self.hints.clone()
        }
    }

    #[test]
    fn collect_hints_from_focus_path() {
        let root = HintWidget {
            name: "Root",
            hints: vec![("Esc", "Close"), ("?", "Help")],
            children: vec![Box::new(HintWidget {
                name: "Panel",
                hints: vec![("Tab", "Switch")],
                children: vec![Box::new(HintWidget {
                    name: "Leaf",
                    hints: vec![("j/k", "↑↓"), ("Enter", "Select")],
                    children: vec![],
                })],
            })],
        };

        // Focus path: root → child[0] → child[0] (Leaf)
        let fm = FocusManager::with_path(vec![0, 0]);
        let hints = fm.collect_hints(&root);

        // Order: Leaf first, then Panel, then Root
        assert_eq!(hints.len(), 5);
        assert_eq!(hints[0], ("j/k".to_string(), "↑↓".to_string()));
        assert_eq!(hints[1], ("Enter".to_string(), "Select".to_string()));
        assert_eq!(hints[2], ("Tab".to_string(), "Switch".to_string()));
        assert_eq!(hints[3], ("Esc".to_string(), "Close".to_string()));
        assert_eq!(hints[4], ("?".to_string(), "Help".to_string()));
    }

    #[test]
    fn collect_hints_empty_focus_returns_root_only() {
        let root = HintWidget {
            name: "Root",
            hints: vec![("?", "Help")],
            children: vec![],
        };
        let fm = FocusManager::new();
        let hints = fm.collect_hints(&root);
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0], ("?".to_string(), "Help".to_string()));
    }

    #[test]
    fn tabbed_container_shortcut_hints() {
        let tc = make_test_tabbed();
        let hints = tc.shortcut_hints();
        assert_eq!(hints, vec![("Tab/S-Tab", "Switch")]);
    }
}
