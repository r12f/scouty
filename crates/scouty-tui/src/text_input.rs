//! Shared text input component with cursor navigation.

#[cfg(test)]
#[path = "text_input_tests.rs"]
mod text_input_tests;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Reusable text input with cursor support.
#[derive(Debug, Clone)]
pub struct TextInput {
    pub text: String,
    /// Cursor position in characters (not bytes).
    pub cursor: usize,
}

impl std::fmt::Display for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn with_text(text: &str) -> Self {
        let cursor = text.chars().count();
        Self {
            text: text.to_string(),
            cursor,
        }
    }

    /// Insert a character at cursor position.
    pub fn insert(&mut self, c: char) {
        let byte_pos = self.byte_pos();
        self.text.insert(byte_pos, c);
        self.cursor += 1;
    }

    /// Delete character before cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let byte_pos = self.byte_pos();
            let ch = self.text[byte_pos..].chars().next().unwrap();
            self.text.drain(byte_pos..byte_pos + ch.len_utf8());
        }
    }

    /// Delete character at cursor (delete key).
    pub fn delete(&mut self) {
        let byte_pos = self.byte_pos();
        if byte_pos < self.text.len() {
            let ch = self.text[byte_pos..].chars().next().unwrap();
            self.text.drain(byte_pos..byte_pos + ch.len_utf8());
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.cursor < self.char_count() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start.
    pub fn home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end.
    pub fn end(&mut self) {
        self.cursor = self.char_count();
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Get the text value.
    pub fn value(&self) -> &str {
        &self.text
    }

    /// Check if input is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get trimmed text.
    pub fn trim(&self) -> &str {
        self.text.trim()
    }

    /// Set the text, moving cursor to end.
    pub fn set(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.char_count();
    }

    /// Push a character at the end (legacy compat).
    pub fn push(&mut self, c: char) {
        self.text.push(c);
        self.cursor = self.char_count();
    }

    /// Pop a character from the end (legacy compat).
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.text.pop();
        self.cursor = self.char_count();
        ch
    }

    /// Get cursor position in characters (for rendering).
    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    /// Delete from cursor to end of line (Ctrl+k).
    pub fn kill_to_end(&mut self) {
        let byte_pos = self.byte_pos();
        self.text.truncate(byte_pos);
    }

    /// Delete from start of line to cursor (Ctrl+u).
    pub fn kill_to_start(&mut self) {
        let byte_pos = self.byte_pos();
        self.text.drain(..byte_pos);
        self.cursor = 0;
    }

    /// Handle a key event. Returns true if the key was consumed.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Normalize Ctrl+H to Backspace (some terminals send ^H)
        if ctrl && key.code == KeyCode::Char('h') {
            self.backspace();
            return true;
        }

        // Readline/emacs shortcuts
        if ctrl {
            match key.code {
                KeyCode::Char('a') => self.home(),
                KeyCode::Char('e') => self.end(),
                KeyCode::Char('k') => self.kill_to_end(),
                KeyCode::Char('u') => self.kill_to_start(),
                _ => return false,
            }
            return true;
        }

        match key.code {
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Home => self.home(),
            KeyCode::End => self.end(),
            KeyCode::Char(c) if !ctrl => self.insert(c),
            _ => return false,
        }
        true
    }

    /// Convert char cursor position to byte position.
    fn byte_pos(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Count characters in text.
    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Split text at cursor position for rendering.
    /// Returns (text_before_cursor, char_at_cursor_or_block, text_after_cursor).
    pub fn render_parts(&self) -> (&str, String, &str) {
        let byte_pos = self.byte_pos();
        let before = &self.text[..byte_pos];
        if byte_pos < self.text.len() {
            let ch = self.text[byte_pos..].chars().next().unwrap();
            let after_pos = byte_pos + ch.len_utf8();
            (before, ch.to_string(), &self.text[after_pos..])
        } else {
            (before, "█".to_string(), "")
        }
    }
}
