use notify::{Config as NotifyConfig, PollWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Child;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::exec::runner::RunOutput;

pub struct WatchState {
    /// Persists for watch duration — exercise files live here
    #[allow(dead_code)]
    pub sandbox_dir: TempDir,
    /// (filename, full_path) of editable files being watched
    pub watched_files: Vec<(String, PathBuf)>,
    /// Non-blocking editor process (if launched)
    pub editor_process: Option<Child>,
    /// Receives file change events from notify
    pub change_rx: mpsc::Receiver<notify::Result<notify::Event>>,
    /// Kept alive to maintain the watch
    _watcher: PollWatcher,
    /// For debounce: don't re-run within 300ms of last trigger
    pub last_run_trigger: Instant,
    /// Last run result for display
    pub last_watch_output: Option<RunOutput>,
    /// If true, grade on save (not just run)
    pub auto_test: bool,
}

impl WatchState {
    pub fn new(
        sandbox_dir: TempDir,
        watched_files: Vec<(String, PathBuf)>,
        editor_process: Option<Child>,
        auto_test: bool,
    ) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        let config = NotifyConfig::default()
            .with_poll_interval(Duration::from_millis(500));

        let mut watcher = PollWatcher::new(tx, config)?;

        // Watch the sandbox directory for changes
        watcher.watch(sandbox_dir.path(), RecursiveMode::Recursive)?;

        Ok(Self {
            sandbox_dir,
            watched_files,
            editor_process,
            change_rx: rx,
            _watcher: watcher,
            last_run_trigger: Instant::now() - Duration::from_secs(1), // allow immediate first run
            last_watch_output: None,
            auto_test,
        })
    }

    /// Drain the receiver, debounce, and return true if we should re-run.
    pub fn check_changes(&mut self) -> bool {
        let mut any_change = false;

        // Drain all pending events
        while let Ok(event) = self.change_rx.try_recv() {
            if let Ok(ev) = event {
                // Only care about modify/create events
                if matches!(
                    ev.kind,
                    notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                ) {
                    any_change = true;
                }
            }
        }

        if !any_change {
            return false;
        }

        // Debounce: at least 300ms since last trigger
        let now = Instant::now();
        if now.duration_since(self.last_run_trigger) < Duration::from_millis(300) {
            return false;
        }

        self.last_run_trigger = now;
        true
    }

    /// Read current file contents from disk.
    pub fn read_files_back(&self) -> Vec<(String, String)> {
        self.watched_files
            .iter()
            .filter_map(|(name, path)| {
                std::fs::read_to_string(path)
                    .ok()
                    .map(|content| (name.clone(), content))
            })
            .collect()
    }

    /// Kill editor process if still alive.
    pub fn cleanup(&mut self) {
        if let Some(ref mut child) = self.editor_process {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.editor_process = None;
    }
}

impl Drop for WatchState {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_state_creation() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test.cpp");
        std::fs::write(&test_file, "int main() {}").unwrap();

        let watched = vec![("test.cpp".to_string(), test_file.clone())];

        let ws = WatchState::new(dir, watched, None, false);
        assert!(ws.is_ok());

        let ws = ws.unwrap();
        assert!(!ws.auto_test);
        assert!(ws.last_watch_output.is_none());
    }

    #[test]
    fn test_read_files_back() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("main.cpp");
        std::fs::write(&test_file, "hello world").unwrap();

        let watched = vec![("main.cpp".to_string(), test_file)];
        let ws = WatchState::new(dir, watched, None, false).unwrap();

        let files = ws.read_files_back();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "main.cpp");
        assert_eq!(files[0].1, "hello world");
    }

    #[test]
    fn test_check_changes_no_events() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test.cpp");
        std::fs::write(&test_file, "").unwrap();

        let watched = vec![("test.cpp".to_string(), test_file)];
        let mut ws = WatchState::new(dir, watched, None, false).unwrap();

        // No changes yet
        assert!(!ws.check_changes());
    }
}
