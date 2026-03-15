use std::collections::HashMap;
use std::path::PathBuf;

use crate::exec::sandbox::Sandbox;

/// A single command + its output in the shell history.
pub struct ShellHistoryEntry {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

/// Persistent state for shell mode (command exercises).
/// Owns the sandbox and background services for the duration of the exercise.
pub struct ShellState {
    pub sandbox: Sandbox,
    pub env_vars: Option<HashMap<String, String>>,
    pub cwd_override: Option<PathBuf>,
    pub needs_loopback: bool,
    pub history: Vec<ShellHistoryEntry>,
    /// Current input line being typed
    pub input: String,
    /// Cursor column within input
    pub cursor_col: usize,
    /// Index into history for Up/Down navigation (None = typing fresh input)
    pub history_nav_idx: Option<usize>,
    /// Saved input when navigating history (restored on Down past end)
    pub saved_input: String,
    /// Scroll offset for the terminal view
    pub scroll_offset: u16,
    /// Total content line count (set by render)
    pub content_line_count: u16,
    /// Background service child processes (name, child)
    pub service_children: Vec<(String, std::process::Child)>,
    /// Drain thread handles for service pipe readers
    pub drain_handles: Vec<std::thread::JoinHandle<()>>,
    /// Whether the help overlay is showing
    pub show_help: bool,
}

impl Drop for ShellState {
    fn drop(&mut self) {
        // Kill any remaining service children
        for (_, child) in &mut self.service_children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl ShellState {
    pub fn new(sandbox: Sandbox) -> Self {
        Self {
            sandbox,
            env_vars: None,
            cwd_override: None,
            needs_loopback: false,
            history: Vec::new(),
            input: String::new(),
            cursor_col: 0,
            history_nav_idx: None,
            saved_input: String::new(),
            scroll_offset: 0,
            content_line_count: 0,
            service_children: Vec::new(),
            drain_handles: Vec::new(),
            show_help: false,
        }
    }

    /// Navigate to the previous command in history (Up arrow).
    pub fn history_prev(&mut self) {
        let cmd_count = self.history.len();
        if cmd_count == 0 {
            return;
        }
        match self.history_nav_idx {
            None => {
                // First Up press: save current input, go to last command
                self.saved_input = self.input.clone();
                let idx = cmd_count - 1;
                self.history_nav_idx = Some(idx);
                self.input = self.history[idx].command.clone();
                self.cursor_col = self.input.chars().count();
            }
            Some(idx) if idx > 0 => {
                let new_idx = idx - 1;
                self.history_nav_idx = Some(new_idx);
                self.input = self.history[new_idx].command.clone();
                self.cursor_col = self.input.chars().count();
            }
            _ => {} // Already at oldest
        }
    }

    /// Navigate to the next command in history (Down arrow).
    pub fn history_next(&mut self) {
        if let Some(idx) = self.history_nav_idx {
            if idx + 1 < self.history.len() {
                let new_idx = idx + 1;
                self.history_nav_idx = Some(new_idx);
                self.input = self.history[new_idx].command.clone();
                self.cursor_col = self.input.chars().count();
            } else {
                // Past end: restore saved input
                self.history_nav_idx = None;
                self.input = self.saved_input.clone();
                self.cursor_col = self.input.chars().count();
            }
        }
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        let char_count = self.input.chars().count();
        let char_idx = self.cursor_col.min(char_count);
        let byte_offset = self.input.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(self.input.len());
        self.input.insert(byte_offset, c);
        self.cursor_col = char_idx + 1;
        // Any edit breaks history navigation
        self.history_nav_idx = None;
    }

    /// Delete the character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let char_idx = self.cursor_col - 1;
            let byte_offset = self.input.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(0);
            self.input.remove(byte_offset);
            self.cursor_col = char_idx;
        }
    }

    /// Delete the character at the cursor (Delete key).
    pub fn delete_char(&mut self) {
        let char_count = self.input.chars().count();
        if self.cursor_col < char_count {
            let byte_offset = self.input.char_indices().nth(self.cursor_col).map(|(i, _)| i).unwrap_or(self.input.len());
            self.input.remove(byte_offset);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        let char_count = self.input.chars().count();
        if self.cursor_col < char_count {
            self.cursor_col += 1;
        }
    }

    /// Clear the current input line (Ctrl+C).
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_col = 0;
        self.history_nav_idx = None;
    }

    /// Take the current input, push it to history as the command text,
    /// and reset input state. Returns the command string (may be empty).
    pub fn take_input(&mut self) -> String {
        let cmd = self.input.clone();
        self.input.clear();
        self.cursor_col = 0;
        self.history_nav_idx = None;
        self.saved_input.clear();
        cmd
    }

    /// Auto-scroll to bottom of content.
    pub fn scroll_to_bottom(&mut self) {
        // Will be clamped in render
        self.scroll_offset = self.content_line_count;
    }
}
