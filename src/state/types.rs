use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Progress {
    pub version: u32,
    #[serde(default)]
    pub courses: HashMap<String, CourseProgress>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            version: 2,
            courses: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseProgress {
    pub course_version: String,
    pub started_at: String,
    pub last_activity: String,
    #[serde(default)]
    pub lessons: HashMap<String, LessonProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonProgress {
    pub status: ProgressStatus,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub exercises: HashMap<String, ExerciseProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseProgress {
    pub status: ProgressStatus,
    #[serde(default)]
    pub attempts: Vec<AttemptRecord>,
    /// Current stage index (0-based) for staged exercises, None for non-staged
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_stage: Option<usize>,
    /// Stage IDs that have been completed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_stages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptRecord {
    pub timestamp: String,
    pub time_spent_seconds: u64,
    pub compile_success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_matched: Option<bool>,
    pub hints_revealed: usize,
    /// Which stage this attempt was for (None for non-staged exercises)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
}

/// Key used for progress storage: {course_id}@{major_version}
pub fn progress_key(course_id: &str, version: &str) -> String {
    let major = semver::Version::parse(version)
        .map(|v| v.major)
        .unwrap_or(0);
    format!("{}@{}", course_id, major)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_key() {
        assert_eq!(
            progress_key("cpp-fundamentals", "1.2.3"),
            "cpp-fundamentals@1"
        );
        assert_eq!(progress_key("rust-basics", "2.0.0"), "rust-basics@2");
    }

    #[test]
    fn test_progress_key_invalid_version() {
        assert_eq!(progress_key("test", "bad"), "test@0");
    }

    #[test]
    fn test_progress_serde_roundtrip() {
        let mut progress = Progress::new();
        progress.courses.insert(
            "test@1".to_string(),
            CourseProgress {
                course_version: "1.0.0".to_string(),
                started_at: "2026-02-07T10:00:00Z".to_string(),
                last_activity: "2026-02-07T11:00:00Z".to_string(),
                lessons: HashMap::new(),
            },
        );

        let json = serde_json::to_string(&progress).unwrap();
        let loaded: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.version, 2);
        assert!(loaded.courses.contains_key("test@1"));
    }

    #[test]
    fn test_attempt_record_serde() {
        let record = AttemptRecord {
            timestamp: "2026-02-07T10:00:00Z".to_string(),
            time_spent_seconds: 45,
            compile_success: true,
            run_exit_code: Some(0),
            output_matched: Some(true),
            hints_revealed: 1,
            stage_id: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        let loaded: AttemptRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.time_spent_seconds, 45);
        assert_eq!(loaded.hints_revealed, 1);
        assert!(loaded.stage_id.is_none());
    }

    #[test]
    fn test_attempt_record_with_stage() {
        let record = AttemptRecord {
            timestamp: "2026-03-21T10:00:00Z".to_string(),
            time_spent_seconds: 30,
            compile_success: true,
            run_exit_code: Some(0),
            output_matched: Some(true),
            hints_revealed: 0,
            stage_id: Some("basic".to_string()),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("basic"));
        let loaded: AttemptRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.stage_id, Some("basic".to_string()));
    }

    #[test]
    fn test_exercise_progress_backward_compat() {
        // Old progress.json without stage fields should deserialize fine
        let json = r#"{"status":"in_progress","attempts":[]}"#;
        let ep: ExerciseProgress = serde_json::from_str(json).unwrap();
        assert_eq!(ep.status, ProgressStatus::InProgress);
        assert!(ep.current_stage.is_none());
        assert!(ep.completed_stages.is_empty());
    }

    #[test]
    fn test_exercise_progress_with_stages() {
        let ep = ExerciseProgress {
            status: ProgressStatus::InProgress,
            attempts: vec![],
            current_stage: Some(1),
            completed_stages: vec!["basic".to_string()],
        };
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains("current_stage"));
        assert!(json.contains("basic"));
        let loaded: ExerciseProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.current_stage, Some(1));
        assert_eq!(loaded.completed_stages, vec!["basic".to_string()]);
    }

    #[test]
    fn test_exercise_progress_stage_fields_omitted_when_empty() {
        let ep = ExerciseProgress {
            status: ProgressStatus::Completed,
            attempts: vec![],
            current_stage: None,
            completed_stages: vec![],
        };
        let json = serde_json::to_string(&ep).unwrap();
        // Stage fields should be omitted via skip_serializing_if
        assert!(!json.contains("current_stage"));
        assert!(!json.contains("completed_stages"));
    }

    #[test]
    fn test_old_attempt_record_without_stage_id() {
        // Old attempt records without stage_id should deserialize fine
        let json = r#"{
            "timestamp": "2026-02-07T10:00:00Z",
            "time_spent_seconds": 45,
            "compile_success": true,
            "hints_revealed": 1
        }"#;
        let record: AttemptRecord = serde_json::from_str(json).unwrap();
        assert!(record.stage_id.is_none());
        assert_eq!(record.time_spent_seconds, 45);
    }
}
