use crate::course::types::{Course, Exercise, ExerciseFile, ExerciseType, Lesson};
use crate::exec::runner::ExecutionResult;
use crate::state::progress::ProgressStore;
use crate::state::signals::SessionState;
use crate::state::types::ProgressStatus;

/// Read-only snapshot of all context needed for LLM prompting.
/// Assembled from immutable borrows of Course, SessionState, ProgressStore.
#[derive(Debug, Clone)]
pub struct LlmContext {
    // Course
    pub course_name: String,
    pub language: String,

    // Lesson
    pub lesson_title: String,
    pub lesson_content: String,
    pub lesson_position: String,
    pub concepts_taught: Vec<String>,

    // Exercise
    pub exercise_title: String,
    pub exercise_prompt: String,
    pub exercise_type: String,
    pub starter_code: String,

    // Live session state
    pub current_code: String,
    pub attempt_number: u32,
    pub hints_revealed: Vec<String>,
    pub time_spent_seconds: u64,
    pub last_execution_summary: String,

    // Progress
    pub completed_exercises: Vec<CompletedExerciseSummary>,
    pub lessons_completed: u32,
    pub total_lessons: u32,

    // Sandbox mode
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone)]
pub struct CompletedExerciseSummary {
    pub exercise_id: String,
    pub attempts: u32,
    pub hints_used: usize,
}

impl LlmContext {
    #[allow(clippy::too_many_arguments)]
    pub fn assemble(
        course: &Course,
        lesson: &Lesson,
        exercise: &Exercise,
        session: &SessionState,
        progress_store: &ProgressStore,
        lesson_idx: usize,
        include_lesson_content: bool,
        _max_history_attempts: u32,
    ) -> Self {
        let course_id = course.name.to_lowercase().replace(' ', "-");
        let key = crate::state::types::progress_key(&course_id, &course.version);

        let lesson_content = if include_lesson_content {
            lesson.content_markdown.clone()
        } else {
            String::new()
        };

        let exercise_type_str = match exercise.exercise_type {
            ExerciseType::Write => "write",
            ExerciseType::Fix => "fix",
            ExerciseType::FillBlank => "fill-blank",
            ExerciseType::MultipleChoice => "multiple-choice",
            ExerciseType::Predict => "predict",
            ExerciseType::Command => "command",
        };

        let starter_code = format_files(&exercise.get_starter_files(&course.language.extension));
        let current_code = format_files(&session.current_code);

        let attempt_number = session.attempt_history.len() as u32 + 1;

        let hints_revealed: Vec<String> = exercise
            .hints
            .iter()
            .take(session.hints_revealed)
            .cloned()
            .collect();

        let last_execution_summary = match &session.last_execution {
            Some(result) => format_execution_result(result),
            None => "No execution yet".to_string(),
        };

        // Collect completed exercises from progress
        let mut completed_exercises = Vec::new();
        if let Some(cp) = progress_store.data.courses.get(&key) {
            if let Some(lp) = cp.lessons.get(&lesson.id) {
                for (ex_id, ep) in &lp.exercises {
                    if ep.status == ProgressStatus::Completed {
                        completed_exercises.push(CompletedExerciseSummary {
                            exercise_id: ex_id.clone(),
                            attempts: ep.attempts.len() as u32,
                            hints_used: ep.attempts.last().map(|a| a.hints_revealed).unwrap_or(0),
                        });
                    }
                }
            }
        }

        let lessons_completed = progress_store
            .data
            .courses
            .get(&key)
            .map(|cp| {
                cp.lessons
                    .values()
                    .filter(|lp| lp.status == ProgressStatus::Completed)
                    .count() as u32
            })
            .unwrap_or(0);

        LlmContext {
            course_name: course.name.clone(),
            language: course.language.display_name.clone(),
            lesson_title: lesson.title.clone(),
            lesson_content,
            lesson_position: format!(
                "Lesson {} of {}",
                lesson_idx + 1,
                course.loaded_lessons.len()
            ),
            concepts_taught: lesson.teaches.clone(),
            exercise_title: exercise.title.clone(),
            exercise_prompt: exercise.prompt.clone(),
            exercise_type: exercise_type_str.to_string(),
            starter_code,
            current_code,
            attempt_number,
            hints_revealed,
            time_spent_seconds: session.time_spent_seconds(),
            last_execution_summary,
            completed_exercises,
            lessons_completed,
            total_lessons: course.loaded_lessons.len() as u32,
            sandbox_mode: false,
        }
    }

    pub fn assemble_sandbox(
        course: &Course,
        lesson: &Lesson,
        current_code: &[ExerciseFile],
        last_output: Option<&str>,
        lesson_idx: usize,
    ) -> Self {
        let current_code_str = format_files(current_code);
        let last_execution_summary = last_output.unwrap_or("No execution yet").to_string();

        LlmContext {
            course_name: course.name.clone(),
            language: course.language.display_name.clone(),
            lesson_title: lesson.title.clone(),
            lesson_content: lesson.content_markdown.clone(),
            lesson_position: format!(
                "Lesson {} of {}",
                lesson_idx + 1,
                course.loaded_lessons.len()
            ),
            concepts_taught: lesson.teaches.clone(),
            exercise_title: "Sandbox — free exploration".to_string(),
            exercise_prompt: String::new(),
            exercise_type: "sandbox".to_string(),
            starter_code: String::new(),
            current_code: current_code_str,
            attempt_number: 0,
            hints_revealed: Vec::new(),
            time_spent_seconds: 0,
            last_execution_summary,
            completed_exercises: Vec::new(),
            lessons_completed: 0,
            total_lessons: course.loaded_lessons.len() as u32,
            sandbox_mode: true,
        }
    }

    pub fn to_system_prompt(&self) -> String {
        if self.sandbox_mode {
            return self.to_sandbox_system_prompt();
        }

        if self.exercise_type == "command" {
            return self.to_shell_system_prompt();
        }

        let lesson_section = if self.lesson_content.is_empty() {
            String::new()
        } else {
            format!("\n## Lesson Content\n{}\n", self.lesson_content)
        };

        let concepts_section = if self.concepts_taught.is_empty() {
            String::new()
        } else {
            format!(
                "\n## Concepts Being Taught\n{}\n",
                self.concepts_taught.join(", ")
            )
        };

        let hints_section = if self.hints_revealed.is_empty() {
            "None revealed yet".to_string()
        } else {
            self.hints_revealed
                .iter()
                .enumerate()
                .map(|(i, h)| format!("{}. {}", i + 1, h))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let completed_section = if self.completed_exercises.is_empty() {
            "None yet".to_string()
        } else {
            self.completed_exercises
                .iter()
                .map(|e| {
                    format!(
                        "- {} ({} attempts, {} hints)",
                        e.exercise_id, e.attempts, e.hints_used
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            r#"You are a friendly programming tutor helping a student learn {language}.

## Current Position
- Course: {course_name}
- {lesson_position}
- Lesson: "{lesson_title}"
- Exercise: "{exercise_title}" (attempt #{attempt_number})
- Time spent: {time_spent}s
- Progress: {lessons_completed}/{total_lessons} lessons completed
{lesson_section}{concepts_section}
## Exercise
Type: {exercise_type}
Task: {exercise_prompt}

Starter code:
```
{starter_code}
```

## Student's Current Code
```
{current_code}
```

## Compilation/Run Results
{execution_summary}

## Hints Already Revealed
{hints_summary}

## Previously Completed Exercises (this lesson)
{completed_section}

## Guidelines
- Be encouraging but don't give away the answer
- Reference specific line numbers when relevant
- Connect to concepts from the lesson content
- If they're close, tell them
- If they're stuck in a loop, try a different explanation angle
- Keep responses concise (under 200 words unless explaining a concept)"#,
            language = self.language,
            course_name = self.course_name,
            lesson_position = self.lesson_position,
            lesson_title = self.lesson_title,
            exercise_title = self.exercise_title,
            attempt_number = self.attempt_number,
            time_spent = self.time_spent_seconds,
            lessons_completed = self.lessons_completed,
            total_lessons = self.total_lessons,
            lesson_section = lesson_section,
            concepts_section = concepts_section,
            exercise_type = self.exercise_type,
            exercise_prompt = self.exercise_prompt,
            starter_code = self.starter_code,
            current_code = self.current_code,
            execution_summary = self.last_execution_summary,
            hints_summary = hints_section,
            completed_section = completed_section,
        )
    }

    fn to_sandbox_system_prompt(&self) -> String {
        let concepts_section = if self.concepts_taught.is_empty() {
            String::new()
        } else {
            format!(
                "\n## Concepts Covered\n{}\n",
                self.concepts_taught.join(", ")
            )
        };

        let code_section = if self.current_code.is_empty() {
            "No code yet".to_string()
        } else {
            self.current_code.clone()
        };

        format!(
            r#"You are a friendly programming tutor. The student just completed the exercises for this lesson and is now experimenting freely in a sandbox.

## Current Position
- Course: {course_name}
- {lesson_position}
- Lesson: "{lesson_title}"
- Language: {language}
{concepts_section}
## Lesson Content
{lesson_content}

## Student's Current Sandbox Code
```
{current_code}
```

## Last Run Output
{execution_summary}

## Guidelines
- Help them explore and experiment with the concepts from this lesson
- Suggest small projects or experiments that use what they just learned
- If they're not sure what to try, offer 2-3 concrete ideas
- Explain concepts differently if asked
- Be encouraging and creative — this is their playground
- Keep responses concise (under 200 words unless explaining a concept)"#,
            language = self.language,
            course_name = self.course_name,
            lesson_position = self.lesson_position,
            lesson_title = self.lesson_title,
            concepts_section = concepts_section,
            lesson_content = self.lesson_content,
            current_code = code_section,
            execution_summary = self.last_execution_summary,
        )
    }

    fn to_shell_system_prompt(&self) -> String {
        let lesson_section = if self.lesson_content.is_empty() {
            String::new()
        } else {
            format!("\n## Lesson Content\n{}\n", self.lesson_content)
        };

        let concepts_section = if self.concepts_taught.is_empty() {
            String::new()
        } else {
            format!(
                "\n## Concepts Being Taught\n{}\n",
                self.concepts_taught.join(", ")
            )
        };

        let hints_section = if self.hints_revealed.is_empty() {
            "None revealed yet".to_string()
        } else {
            self.hints_revealed
                .iter()
                .enumerate()
                .map(|(i, h)| format!("{}. {}", i + 1, h))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let transcript_section = if self.current_code.is_empty() {
            "No commands run yet".to_string()
        } else {
            self.current_code.clone()
        };

        format!(
            r#"You are a friendly programming tutor helping a student learn {language} in an interactive shell.

## Current Position
- Course: {course_name}
- {lesson_position}
- Lesson: "{lesson_title}"
- Exercise: "{exercise_title}" (attempt #{attempt_number})
- Time spent: {time_spent}s
- Progress: {lessons_completed}/{total_lessons} lessons completed
{lesson_section}{concepts_section}
## Exercise
Type: command (interactive shell)
Task: {exercise_prompt}

## Shell Transcript (commands the student has run)
```
{transcript}
```

## Hints Already Revealed
{hints_summary}

## Guidelines
- The student is working in an interactive shell, running one command at a time
- Help them build toward the solution step by step
- Suggest specific commands they can try next
- If a command failed, explain why and suggest a fix
- Don't give away the complete solution — guide them toward it
- For multi-step tasks, help them understand the sequence
- Keep responses concise (under 200 words unless explaining a concept)"#,
            language = self.language,
            course_name = self.course_name,
            lesson_position = self.lesson_position,
            lesson_title = self.lesson_title,
            exercise_title = self.exercise_title,
            attempt_number = self.attempt_number,
            time_spent = self.time_spent_seconds,
            lessons_completed = self.lessons_completed,
            total_lessons = self.total_lessons,
            lesson_section = lesson_section,
            concepts_section = concepts_section,
            exercise_prompt = self.exercise_prompt,
            transcript = transcript_section,
            hints_summary = hints_section,
        )
    }
}

fn format_files(files: &[ExerciseFile]) -> String {
    if files.len() == 1 {
        files[0].content.clone()
    } else {
        files
            .iter()
            .map(|f| format!("// {}\n{}", f.name, f.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_execution_result(result: &ExecutionResult) -> String {
    match result {
        ExecutionResult::Success => "Success - output matched expected".to_string(),
        ExecutionResult::CompileSuccess => "Compilation succeeded".to_string(),
        ExecutionResult::StepFailed {
            step_name,
            stderr,
            exit_code,
        } => {
            let truncated = if stderr.len() > 500 {
                format!("{}...(truncated)", &stderr[..500])
            } else {
                stderr.clone()
            };
            format!(
                "{} failed (exit code {}):\n{}",
                step_name, exit_code, truncated
            )
        }
        ExecutionResult::ValidationFailed(vr) => match vr {
            crate::exec::validate::ValidationResult::OutputMismatch { expected, actual } => {
                format!(
                    "Output mismatch:\n  Expected: \"{}\"\n  Actual:   \"{}\"",
                    expected, actual
                )
            }
            crate::exec::validate::ValidationResult::RegexMismatch { pattern, actual } => {
                format!(
                    "Pattern mismatch:\n  Pattern: /{}/\n  Actual:  \"{}\"",
                    pattern, actual
                )
            }
            crate::exec::validate::ValidationResult::StateAssertionFailed { results } => {
                let mut lines = vec!["State assertion results:".to_string()];
                for r in results {
                    let icon = if r.passed { "PASS" } else { "FAIL" };
                    lines.push(format!("  [{}] {}: {}", icon, r.description, r.detail));
                }
                lines.join("\n")
            }
            _ => "Validation failed".to_string(),
        },
        ExecutionResult::Timeout { step_name } => format!("{} timed out", step_name),
        ExecutionResult::SetupFailed {
            step_name,
            stderr,
            exit_code,
        } => format!(
            "INFRASTRUCTURE ERROR — setup step '{}' failed (exit code {}). This is a course setup issue, not the student's code.\n{}",
            step_name, exit_code, stderr
        ),
        ExecutionResult::ServiceFailed {
            service_name,
            error,
        } => format!(
            "INFRASTRUCTURE ERROR — service '{}' failed to start. This is a course setup issue, not the student's code.\n{}",
            service_name, error
        ),
        ExecutionResult::Error(msg) => format!("Error: {}", msg),
        ExecutionResult::StageComplete {
            stage_id,
            stage_idx,
            is_final,
        } => {
            if *is_final {
                format!("All stages complete (final: stage {} '{}')", stage_idx + 1, stage_id)
            } else {
                format!("Stage {} '{}' complete — next stage ready", stage_idx + 1, stage_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::types::*;
    use crate::state::signals::SessionState;

    fn make_test_course() -> Course {
        Course {
            name: "C++ Fundamentals".to_string(),
            version: "1.0.0".to_string(),
            description: "Learn C++".to_string(),
            author: "Test".to_string(),
            license: None,
            platform: None,
            language: Language {
                id: "cpp".to_string(),
                display_name: "C++".to_string(),
                extension: ".cpp".to_string(),
                steps: vec![],
                limits: ExecutionLimits::default(),
                provision: crate::course::types::Provision::default(),
                runtime: None,
            },
            lessons: vec![],
            estimated_minutes_per_lesson: None,
            loaded_lessons: vec![Lesson {
                id: "variables".to_string(),
                title: "Variables".to_string(),
                description: None,
                estimated_minutes: None,
                content: "variables".to_string(),
                exercises: vec!["declare".to_string()],
                teaches: vec!["int".to_string(), "variables".to_string()],
                recap: None,
                loaded_exercises: vec![Exercise {
                    id: "declare".to_string(),
                    title: "Declare a Variable".to_string(),
                    exercise_type: ExerciseType::Write,
                    prompt: "Declare int age = 25".to_string(),
                    starter: Some("// your code".to_string()),
                    files: vec![],
                    main_file: None,
                    input: None,
                    validation: Validation {
                        method: ValidationMethod::Output,
                        expected_output: Some("25".to_string()),
                        pattern: None,
                        script: None,
                        assertions: None,
                    },
                    hints: vec!["Use int".to_string(), "int age = 25;".to_string()],
                    solution: Some("int age = 25;".to_string()),
                    solution_files: vec![],
                    explanation: None,
                    environment: None,
                    golf: false,
                    stages: vec![],
                }],
                content_markdown: "# Variables\nLearn about vars.".to_string(),
                content_sections: vec![],
            }],
            source_dir: std::path::PathBuf::new(),
        }
    }

    #[test]
    fn test_context_assembly() {
        let course = make_test_course();
        let lesson = &course.loaded_lessons[0];
        let exercise = &lesson.loaded_exercises[0];
        let session = SessionState::new(exercise.get_starter_files(&course.language.extension));
        let store = ProgressStore::empty();

        let ctx = LlmContext::assemble(&course, lesson, exercise, &session, &store, 0, true, 3);

        assert_eq!(ctx.course_name, "C++ Fundamentals");
        assert_eq!(ctx.language, "C++");
        assert_eq!(ctx.lesson_title, "Variables");
        assert_eq!(ctx.exercise_title, "Declare a Variable");
        assert_eq!(ctx.exercise_type, "write");
        assert_eq!(ctx.attempt_number, 1);
        assert_eq!(ctx.lessons_completed, 0);
        assert_eq!(ctx.total_lessons, 1);
        assert_eq!(ctx.lesson_position, "Lesson 1 of 1");
        assert!(ctx.hints_revealed.is_empty());
        assert!(ctx.lesson_content.contains("Variables"));
    }

    #[test]
    fn test_context_without_lesson_content() {
        let course = make_test_course();
        let lesson = &course.loaded_lessons[0];
        let exercise = &lesson.loaded_exercises[0];
        let session = SessionState::new(exercise.get_starter_files(&course.language.extension));
        let store = ProgressStore::empty();

        let ctx = LlmContext::assemble(&course, lesson, exercise, &session, &store, 0, false, 3);

        assert!(ctx.lesson_content.is_empty());
    }

    #[test]
    fn test_system_prompt_contains_sections() {
        let course = make_test_course();
        let lesson = &course.loaded_lessons[0];
        let exercise = &lesson.loaded_exercises[0];
        let session = SessionState::new(exercise.get_starter_files(&course.language.extension));
        let store = ProgressStore::empty();

        let ctx = LlmContext::assemble(&course, lesson, exercise, &session, &store, 0, true, 3);
        let prompt = ctx.to_system_prompt();

        assert!(prompt.contains("friendly programming tutor"));
        assert!(prompt.contains("C++ Fundamentals"));
        assert!(prompt.contains("Declare a Variable"));
        assert!(prompt.contains("## Exercise"));
        assert!(prompt.contains("## Student's Current Code"));
        assert!(prompt.contains("## Guidelines"));
        assert!(prompt.contains("## Lesson Content"));
        assert!(prompt.contains("## Concepts Being Taught"));
        assert!(prompt.contains("int, variables"));
    }

    #[test]
    fn test_format_execution_result_variants() {
        assert_eq!(
            format_execution_result(&ExecutionResult::Success),
            "Success - output matched expected"
        );
        assert_eq!(
            format_execution_result(&ExecutionResult::Timeout {
                step_name: "compile".to_string()
            }),
            "compile timed out"
        );
        let fail_result = format_execution_result(&ExecutionResult::StepFailed {
            step_name: "compile".to_string(),
            stderr: "error: expected ';'".to_string(),
            exit_code: 1,
        });
        assert!(fail_result.contains("compile failed"));
        assert!(fail_result.contains("expected ';'"));
    }
}
