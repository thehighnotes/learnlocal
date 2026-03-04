use std::cell::Cell;

/// Result of processing a key event in the inline editor.
#[derive(Debug, PartialEq)]
pub enum EditorAction {
    /// Keep editing
    Continue,
    /// Save and close editor (Esc)
    SaveAndClose,
    /// Save without closing (Ctrl+S)
    Save,
}

pub struct InlineEditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub file_idx: usize,
    pub dirty: bool,
    /// Set by render() so we know how many lines are visible
    pub visible_height: Cell<usize>,
    /// When true, renders `$ ` prefix instead of line numbers, and Tab is ignored
    pub command_mode: bool,
}

impl InlineEditorState {
    pub fn new(content: &str, file_idx: usize) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        // Ensure at least one line
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            lines,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            file_idx,
            dirty: false,
            visible_height: Cell::new(20),
            command_mode: false,
        }
    }

    pub fn new_command_mode(content: &str, file_idx: usize) -> Self {
        let mut state = Self::new(content, file_idx);
        state.command_mode = true;
        state
    }

    pub fn content(&self) -> String {
        self.lines.join("\n") + "\n"
    }

    /// Process a key event. Returns an EditorAction.
    pub fn handle_key(
        &mut self,
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> EditorAction {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Ctrl+S = save
        if code == KeyCode::Char('s') && modifiers.contains(KeyModifiers::CONTROL) {
            return EditorAction::Save;
        }

        // Esc = save and close
        if code == KeyCode::Esc {
            return EditorAction::SaveAndClose;
        }

        match code {
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Tab => {
                if !self.command_mode {
                    self.insert_tab();
                }
            }
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => {
                self.cursor_col = self.lines[self.cursor_line].len();
            }
            _ => {}
        }

        EditorAction::Continue
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_col > line.len() {
            self.cursor_col = line.len();
        }
        line.insert(self.cursor_col, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_col > line.len() {
            self.cursor_col = line.len();
        }
        let rest = line[self.cursor_col..].to_string();
        line.truncate(self.cursor_col);
        self.lines.insert(self.cursor_line + 1, rest);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.dirty = true;
        self.ensure_visible();
    }

    fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(' ');
        }
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col > line.len() {
                self.cursor_col = line.len();
            }
            if self.cursor_col > 0 {
                line.remove(self.cursor_col - 1);
                self.cursor_col -= 1;
                self.dirty = true;
            }
        } else if self.cursor_line > 0 {
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
            self.dirty = true;
            self.ensure_visible();
        }
    }

    fn delete(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_line].remove(self.cursor_col);
            self.dirty = true;
        } else if self.cursor_line + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next_line);
            self.dirty = true;
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.ensure_visible();
        }
    }

    fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.ensure_visible();
        }
    }

    fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor_col();
            self.ensure_visible();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.clamp_cursor_col();
            self.ensure_visible();
        }
    }

    fn clamp_cursor_col(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    fn ensure_visible(&mut self) {
        let visible = self.visible_height.get();
        if visible == 0 {
            return;
        }
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + visible {
            self.scroll_offset = self.cursor_line - visible + 1;
        }
    }
}

/// Split a line at the cursor position, returning (before, cursor_char, after).
/// If cursor is past end of line, cursor_char is a space.
pub fn split_at_cursor(line: &str, col: usize) -> (String, String, String) {
    if col >= line.len() {
        (line.to_string(), " ".to_string(), String::new())
    } else {
        let before = line[..col].to_string();
        let cursor_char = line[col..col + 1].to_string();
        let after = line[col + 1..].to_string();
        (before, cursor_char, after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_new_editor_state() {
        let editor = InlineEditorState::new("hello\nworld", 0);
        assert_eq!(editor.lines, vec!["hello", "world"]);
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 0);
        assert!(!editor.dirty);
    }

    #[test]
    fn test_new_editor_empty() {
        let editor = InlineEditorState::new("", 0);
        assert_eq!(editor.lines, vec![""]);
    }

    #[test]
    fn test_insert_char() {
        let mut editor = InlineEditorState::new("ab", 0);
        editor.cursor_col = 1;
        editor.handle_key(KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(editor.lines[0], "aXb");
        assert_eq!(editor.cursor_col, 2);
        assert!(editor.dirty);
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = InlineEditorState::new("hello world", 0);
        editor.cursor_col = 5;
        editor.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(editor.lines, vec!["hello", " world"]);
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_backspace_within_line() {
        let mut editor = InlineEditorState::new("abc", 0);
        editor.cursor_col = 2;
        editor.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(editor.lines[0], "ac");
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_backspace_merge_lines() {
        let mut editor = InlineEditorState::new("ab\ncd", 0);
        editor.cursor_line = 1;
        editor.cursor_col = 0;
        editor.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(editor.lines, vec!["abcd"]);
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 2);
    }

    #[test]
    fn test_delete_within_line() {
        let mut editor = InlineEditorState::new("abc", 0);
        editor.cursor_col = 1;
        editor.handle_key(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(editor.lines[0], "ac");
    }

    #[test]
    fn test_delete_merge_lines() {
        let mut editor = InlineEditorState::new("ab\ncd", 0);
        editor.cursor_col = 2; // end of first line
        editor.handle_key(KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(editor.lines, vec!["abcd"]);
    }

    #[test]
    fn test_arrow_navigation() {
        let mut editor = InlineEditorState::new("ab\ncd", 0);
        editor.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(editor.cursor_col, 1);
        editor.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 1);
        editor.handle_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(editor.cursor_col, 0);
        editor.handle_key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(editor.cursor_line, 0);
    }

    #[test]
    fn test_home_end() {
        let mut editor = InlineEditorState::new("hello", 0);
        editor.cursor_col = 3;
        editor.handle_key(KeyCode::Home, KeyModifiers::NONE);
        assert_eq!(editor.cursor_col, 0);
        editor.handle_key(KeyCode::End, KeyModifiers::NONE);
        assert_eq!(editor.cursor_col, 5);
    }

    #[test]
    fn test_tab_inserts_spaces() {
        let mut editor = InlineEditorState::new("", 0);
        editor.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(editor.lines[0], "    ");
        assert_eq!(editor.cursor_col, 4);
    }

    #[test]
    fn test_esc_returns_save_and_close() {
        let mut editor = InlineEditorState::new("", 0);
        let action = editor.handle_key(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(action, EditorAction::SaveAndClose);
    }

    #[test]
    fn test_ctrl_s_returns_save() {
        let mut editor = InlineEditorState::new("", 0);
        let action = editor.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(action, EditorAction::Save);
    }

    #[test]
    fn test_content_roundtrip() {
        let original = "line1\nline2\nline3\n";
        let editor = InlineEditorState::new(original, 0);
        assert_eq!(editor.content(), original);
    }

    #[test]
    fn test_split_at_cursor() {
        let (before, cursor, after) = split_at_cursor("hello", 2);
        assert_eq!(before, "he");
        assert_eq!(cursor, "l");
        assert_eq!(after, "lo");
    }

    #[test]
    fn test_split_at_cursor_end() {
        let (before, cursor, after) = split_at_cursor("hi", 2);
        assert_eq!(before, "hi");
        assert_eq!(cursor, " ");
        assert_eq!(after, "");
    }

    #[test]
    fn test_wrap_right_to_next_line() {
        let mut editor = InlineEditorState::new("ab\ncd", 0);
        editor.cursor_col = 2; // end of first line
        editor.handle_key(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(editor.cursor_line, 1);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_command_mode_tab_ignored() {
        let mut editor = InlineEditorState::new_command_mode("", 0);
        editor.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(editor.lines[0], "");
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_command_mode_flag() {
        let editor = InlineEditorState::new_command_mode("ls -la", 0);
        assert!(editor.command_mode);
        assert_eq!(editor.lines, vec!["ls -la"]);

        let editor = InlineEditorState::new("ls -la", 0);
        assert!(!editor.command_mode);
    }

    #[test]
    fn test_wrap_left_to_prev_line() {
        let mut editor = InlineEditorState::new("ab\ncd", 0);
        editor.cursor_line = 1;
        editor.cursor_col = 0;
        editor.handle_key(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 2);
    }
}
