use crate::error::{LearnLocalError, Result};
use std::path::{Path, PathBuf};

/// Returns the persistent sandbox directory for a given course/lesson.
/// `~/.local/share/learnlocal/sandboxes/{course_id}@{major}/{lesson_id}/`
pub fn sandbox_dir(course_id: &str, version: &str, lesson_id: &str) -> Result<PathBuf> {
    let data_dir = dirs::data_dir().ok_or_else(|| {
        LearnLocalError::Progress("Could not determine data directory".to_string())
    })?;
    let major = crate::state::types::progress_key(course_id, version);
    Ok(data_dir
        .join("learnlocal")
        .join("sandboxes")
        .join(major)
        .join(lesson_id))
}

/// Save sandbox files to the persistent directory.
pub fn save_sandbox_files(dir: &Path, files: &[(String, String)]) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, content) in files {
        let path = dir.join(name);
        std::fs::write(&path, content)?;
    }
    Ok(())
}

/// Load all files from a sandbox directory.
pub fn load_sandbox_files(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = std::fs::read_to_string(&path)?;
            files.push((name, content));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

// --- Exercise draft persistence ---

/// Returns the persistent draft directory for a given course/lesson/exercise.
/// `~/.local/share/learnlocal/drafts/{course_id}@{major}/{lesson_id}/{exercise_id}/`
pub fn draft_dir(
    course_id: &str,
    version: &str,
    lesson_id: &str,
    exercise_id: &str,
) -> Result<PathBuf> {
    let data_dir = dirs::data_dir().ok_or_else(|| {
        LearnLocalError::Progress("Could not determine data directory".to_string())
    })?;
    let major = crate::state::types::progress_key(course_id, version);
    Ok(data_dir
        .join("learnlocal")
        .join("drafts")
        .join(major)
        .join(lesson_id)
        .join(exercise_id))
}

/// Save exercise draft files to the persistent directory.
pub fn save_draft_files(dir: &Path, files: &[(String, String)]) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, content) in files {
        let path = dir.join(name);
        std::fs::write(&path, content)?;
    }
    Ok(())
}

/// Load all files from a draft directory.
pub fn load_draft_files(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let content = std::fs::read_to_string(&path)?;
            files.push((name, content));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

/// Remove all draft files for a given exercise.
pub fn clear_draft_files(dir: &Path) -> Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// Quick check: does a sandbox directory exist with files in it?
pub fn has_sandbox_files(course_id: &str, version: &str, lesson_id: &str) -> bool {
    sandbox_dir(course_id, version, lesson_id)
        .ok()
        .map(|dir| {
            dir.exists()
                && std::fs::read_dir(&dir)
                    .ok()
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("sandbox");

        let files = vec![
            ("main.py".to_string(), "print('hello')".to_string()),
            ("helper.py".to_string(), "def foo(): pass".to_string()),
        ];

        save_sandbox_files(&dir, &files).unwrap();
        let loaded = load_sandbox_files(&dir).unwrap();

        assert_eq!(loaded.len(), 2);
        assert!(loaded
            .iter()
            .any(|(n, c)| n == "main.py" && c == "print('hello')"));
        assert!(loaded
            .iter()
            .any(|(n, c)| n == "helper.py" && c == "def foo(): pass"));
    }

    #[test]
    fn test_load_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("nonexistent");
        let loaded = load_sandbox_files(&dir).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_has_sandbox_files_false_when_missing() {
        // Uses a course_id that won't exist on disk
        assert!(!has_sandbox_files(
            "nonexistent-course",
            "1.0.0",
            "lesson-1"
        ));
    }

    #[test]
    fn test_sandbox_dir_format() {
        let dir = sandbox_dir("cpp-fundamentals", "2.0.0", "variables").unwrap();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("sandboxes"));
        assert!(dir_str.contains("cpp-fundamentals@2"));
        assert!(dir_str.contains("variables"));
    }

    #[test]
    fn test_save_overwrite() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("sandbox");

        let files1 = vec![("main.py".to_string(), "v1".to_string())];
        save_sandbox_files(&dir, &files1).unwrap();

        let files2 = vec![("main.py".to_string(), "v2".to_string())];
        save_sandbox_files(&dir, &files2).unwrap();

        let loaded = load_sandbox_files(&dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1, "v2");
    }

    #[test]
    fn test_draft_dir_format() {
        let dir = draft_dir("cpp-fundamentals", "2.0.0", "variables", "ex-01").unwrap();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("drafts"));
        assert!(dir_str.contains("cpp-fundamentals@2"));
        assert!(dir_str.contains("variables"));
        assert!(dir_str.contains("ex-01"));
    }

    #[test]
    fn test_draft_save_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("draft");

        let files = vec![("main.cpp".to_string(), "int main() {}".to_string())];

        save_draft_files(&dir, &files).unwrap();
        let loaded = load_draft_files(&dir).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "main.cpp");
        assert_eq!(loaded[0].1, "int main() {}");
    }

    #[test]
    fn test_draft_load_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("nonexistent");
        let loaded = load_draft_files(&dir).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_draft_clear() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("draft");

        let files = vec![("main.py".to_string(), "code".to_string())];
        save_draft_files(&dir, &files).unwrap();
        assert!(dir.exists());

        clear_draft_files(&dir).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn test_draft_clear_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("nonexistent");
        // Should not error when clearing a directory that doesn't exist
        clear_draft_files(&dir).unwrap();
    }
}
