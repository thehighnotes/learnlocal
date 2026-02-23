use std::path::PathBuf;
use crate::error::{LearnLocalError, Result};
use super::types::Progress;

pub struct ProgressStore {
    path: PathBuf,
    pub data: Progress,
}

impl ProgressStore {
    /// Load progress from the default location, or create a new one if it doesn't exist.
    pub fn load() -> Result<Self> {
        let path = progress_file_path()?;
        Self::load_from(path)
    }

    /// Load progress from a specific path.
    pub fn load_from(path: PathBuf) -> Result<Self> {
        let data = if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            serde_json::from_str(&contents).map_err(|e| {
                LearnLocalError::Progress(format!("Failed to parse progress file: {}", e))
            })?
        } else {
            Progress::new()
        };

        Ok(Self { path, data })
    }

    /// Save progress to disk atomically (write tmp + rename).
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.data)?;

        // Atomic write: write to temp file, then rename
        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, &self.path).map_err(|e| {
            LearnLocalError::Progress(format!("Failed to save progress: {}", e))
        })?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Create an empty ProgressStore (for tests and initialization).
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            path: PathBuf::new(),
            data: Progress::new(),
        }
    }
}

fn progress_file_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| LearnLocalError::Progress("Could not determine data directory".to_string()))?;
    Ok(data_dir.join("learnlocal").join("progress.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nonexistent_creates_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("progress.json");
        let store = ProgressStore::load_from(path).unwrap();
        assert_eq!(store.data.version, 2);
        assert!(store.data.courses.is_empty());
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("progress.json");

        let mut store = ProgressStore::load_from(path.clone()).unwrap();
        store.data.courses.insert(
            "test@1".to_string(),
            super::super::types::CourseProgress {
                course_version: "1.0.0".to_string(),
                started_at: "2026-02-07T10:00:00Z".to_string(),
                last_activity: "2026-02-07T11:00:00Z".to_string(),
                lessons: std::collections::HashMap::new(),
            },
        );
        store.save().unwrap();

        let reloaded = ProgressStore::load_from(path).unwrap();
        assert!(reloaded.data.courses.contains_key("test@1"));
    }
}
