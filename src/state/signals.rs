use crate::course::types::ExerciseFile;
use crate::exec::runner::ExecutionResult;
use std::collections::HashMap;
use std::time::Instant;

/// In-memory session state, not persisted.
pub struct SessionState {
    /// Current code the user is editing
    pub current_code: Vec<ExerciseFile>,

    /// Full attempt history for this exercise
    pub attempt_history: Vec<FullAttempt>,

    /// When this exercise was first shown
    pub exercise_started_at: Instant,

    /// How many hints have been revealed (for non-staged or current stage)
    pub hints_revealed: usize,

    /// Last execution result
    pub last_execution: Option<ExecutionResult>,

    /// Current stage index for staged exercises (None for non-staged)
    pub current_stage_idx: Option<usize>,

    /// Per-stage hint tracking: stage_index → hints revealed count
    pub stage_hints_revealed: HashMap<usize, usize>,
}

impl SessionState {
    pub fn new(starter_files: Vec<ExerciseFile>) -> Self {
        Self {
            current_code: starter_files,
            attempt_history: Vec::new(),
            exercise_started_at: Instant::now(),
            hints_revealed: 0,
            last_execution: None,
            current_stage_idx: None,
            stage_hints_revealed: HashMap::new(),
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
        self.current_stage_idx = None;
        self.stage_hints_revealed.clear();
    }

    /// Advance to the next stage. Preserves current_code (the defining mechanic).
    /// Resets hints for the new stage, increments stage index.
    pub fn advance_stage(&mut self) {
        // Save current stage's hint count
        if let Some(idx) = self.current_stage_idx {
            self.stage_hints_revealed.insert(idx, self.hints_revealed);
        }

        let next_idx = self.current_stage_idx.map_or(1, |idx| idx + 1);
        self.current_stage_idx = Some(next_idx);

        // Restore hint count for this stage if we've been here before, else reset
        self.hints_revealed = self
            .stage_hints_revealed
            .get(&next_idx)
            .copied()
            .unwrap_or(0);
        self.last_execution = None;
        // NOTE: current_code is NOT reset — code carries forward
    }

    /// Initialize session for a staged exercise starting at a specific stage.
    pub fn init_staged(&mut self, stage_idx: usize) {
        self.current_stage_idx = Some(stage_idx);
        self.hints_revealed = self
            .stage_hints_revealed
            .get(&stage_idx)
            .copied()
            .unwrap_or(0);
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
