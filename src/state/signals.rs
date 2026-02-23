use crate::course::types::ExerciseFile;
use crate::exec::runner::ExecutionResult;
use std::time::Instant;

/// In-memory session state, not persisted.
pub struct SessionState {
    /// Current code the user is editing
    pub current_code: Vec<ExerciseFile>,

    /// Full attempt history for this exercise
    pub attempt_history: Vec<FullAttempt>,

    /// When this exercise was first shown
    pub exercise_started_at: Instant,

    /// How many hints have been revealed
    pub hints_revealed: usize,

    /// Last execution result
    pub last_execution: Option<ExecutionResult>,
}

impl SessionState {
    pub fn new(starter_files: Vec<ExerciseFile>) -> Self {
        Self {
            current_code: starter_files,
            attempt_history: Vec::new(),
            exercise_started_at: Instant::now(),
            hints_revealed: 0,
            last_execution: None,
        }
    }

    pub fn time_spent_seconds(&self) -> u64 {
        self.exercise_started_at.elapsed().as_secs()
    }

    pub fn reset_for_exercise(&mut self, starter_files: Vec<ExerciseFile>) {
        self.current_code = starter_files;
        self.attempt_history.clear();
        self.exercise_started_at = Instant::now();
        self.hints_revealed = 0;
        self.last_execution = None;
    }
}

#[allow(dead_code)]
pub struct FullAttempt {
    pub code: Vec<ExerciseFile>,
    pub execution_result: ExecutionResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub time_spent_seconds: u64,
    pub hints_revealed: usize,
}
