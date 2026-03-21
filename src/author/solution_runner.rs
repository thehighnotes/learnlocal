use std::path::Path;

use crate::cli_fmt;
use crate::course;
use crate::exec;
use crate::exit_codes;

/// Run a single exercise's solution and show output.
/// With `--update`, auto-update expected_output in the exercise YAML.
pub fn run_solution(
    path: &Path,
    lesson_id: &str,
    exercise_id: &str,
    update: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    let c = course::load_course(path)?;

    let lesson = c
        .loaded_lessons
        .iter()
        .find(|l| l.id == lesson_id)
        .ok_or_else(|| anyhow::anyhow!("Lesson '{}' not found in course", lesson_id))?;

    let exercise = lesson
        .loaded_exercises
        .iter()
        .find(|e| e.id == exercise_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Exercise '{}' not found in lesson '{}'",
                exercise_id,
                lesson_id
            )
        })?;

    println!(
        "Running solution for {}/{} ...",
        cli_fmt::bold(lesson_id),
        cli_fmt::bold(exercise_id)
    );

    // Run base solution (or only solution for non-staged)
    let solution_files = exercise.get_solution_files(&c.language.extension);
    if solution_files.is_empty() {
        println!("  {} No solution provided", cli_fmt::yellow("\u{26a0}"));
        return Ok(());
    }

    let (result, _teardown_warnings) = exec::execute_exercise(&c, exercise, &solution_files)?;

    if result.is_success() {
        // Get the captured output
        let output = exec::runner::run_exercise_with_sandbox(
            &c,
            exercise,
            &solution_files,
            exec::sandbox::SandboxLevel::Basic,
        )?;
        let stdout = output.stdout.trim_end();

        println!(
            "  {} Solution passes validation",
            cli_fmt::green("\u{2713}")
        );
        if !stdout.is_empty() {
            println!("  Output: \"{}\"", stdout);
        }

        if update && !stdout.is_empty() {
            update_expected_output(path, lesson_id, exercise_id, stdout, None, verbose)?;
        }
    } else {
        println!("  {} Solution FAILS validation", cli_fmt::red("\u{2717}"));
        print_failure(&result, verbose);
    }

    // Run staged solutions if present
    if exercise.is_staged() {
        println!();
        for (idx, stage) in exercise.stages.iter().enumerate() {
            let stage_solution = exercise.get_stage_solution_files(idx, &c.language.extension);
            if stage_solution.is_empty() {
                println!(
                    "  {} Stage '{}': no solution provided",
                    cli_fmt::yellow("\u{26a0}"),
                    stage.id
                );
                continue;
            }

            let (stage_result, _tw) = exec::runner::execute_exercise_staged(
                &c,
                exercise,
                &stage_solution,
                idx,
                exec::sandbox::SandboxLevel::Basic,
            )?;

            if stage_result.is_success() {
                let stage_output = exec::runner::run_exercise_with_sandbox(
                    &c,
                    exercise,
                    &stage_solution,
                    exec::sandbox::SandboxLevel::Basic,
                )?;
                let stdout = stage_output.stdout.trim_end();

                println!(
                    "  {} Stage '{}': passes",
                    cli_fmt::green("\u{2713}"),
                    stage.id
                );
                if !stdout.is_empty() && verbose {
                    println!("    Output: \"{}\"", stdout);
                }

                if update && !stdout.is_empty() {
                    update_expected_output(
                        path,
                        lesson_id,
                        exercise_id,
                        stdout,
                        Some(idx),
                        verbose,
                    )?;
                }
            } else {
                println!("  {} Stage '{}': FAILS", cli_fmt::red("\u{2717}"), stage.id);
                if verbose {
                    print_failure(&stage_result, true);
                }
            }
        }
    }

    Ok(())
}

/// Run ALL solutions in a course. With `--update`, auto-update expected_output.
pub fn run_all_solutions(path: &Path, update: bool, verbose: bool) -> anyhow::Result<i32> {
    let c = course::load_course(path)?;
    println!(
        "Running all solutions for {} v{} ...\n",
        cli_fmt::bold(&c.name),
        c.version
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for lesson in &c.loaded_lessons {
        for exercise in &lesson.loaded_exercises {
            let solution_files = exercise.get_solution_files(&c.language.extension);
            if solution_files.is_empty() {
                skipped += 1;
                continue;
            }

            if verbose {
                println!(
                    "  {} {}/{}",
                    cli_fmt::dim("running"),
                    lesson.id,
                    exercise.id
                );
            }

            match exec::execute_exercise(&c, exercise, &solution_files) {
                Ok((result, _tw)) => {
                    if result.is_success() {
                        println!(
                            "  {} {}/{}",
                            cli_fmt::green("\u{2713}"),
                            lesson.id,
                            exercise.id
                        );
                        passed += 1;

                        if update {
                            if let Ok(output) = exec::runner::run_exercise_with_sandbox(
                                &c,
                                exercise,
                                &solution_files,
                                exec::sandbox::SandboxLevel::Basic,
                            ) {
                                let stdout = output.stdout.trim_end();
                                if !stdout.is_empty() {
                                    let _ = update_expected_output(
                                        path,
                                        &lesson.id,
                                        &exercise.id,
                                        stdout,
                                        None,
                                        verbose,
                                    );
                                }
                            }
                        }
                    } else {
                        let msg = format_failure(&result);
                        println!(
                            "  {} {}/{}: {}",
                            cli_fmt::red("\u{2717}"),
                            lesson.id,
                            exercise.id,
                            msg
                        );
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!(
                        "  {} {}/{}: {}",
                        cli_fmt::red("\u{2717}"),
                        lesson.id,
                        exercise.id,
                        e
                    );
                    failed += 1;
                }
            }

            // Run staged solutions
            if exercise.is_staged() {
                for (idx, stage) in exercise.stages.iter().enumerate() {
                    let stage_sol = exercise.get_stage_solution_files(idx, &c.language.extension);
                    if stage_sol.is_empty() {
                        continue;
                    }

                    match exec::runner::execute_exercise_staged(
                        &c,
                        exercise,
                        &stage_sol,
                        idx,
                        exec::sandbox::SandboxLevel::Basic,
                    ) {
                        Ok((result, _tw)) => {
                            if result.is_success() {
                                if verbose {
                                    println!(
                                        "    {} stage '{}'",
                                        cli_fmt::green("\u{2713}"),
                                        stage.id
                                    );
                                }

                                if update {
                                    if let Ok(output) = exec::runner::run_exercise_with_sandbox(
                                        &c,
                                        exercise,
                                        &stage_sol,
                                        exec::sandbox::SandboxLevel::Basic,
                                    ) {
                                        let stdout = output.stdout.trim_end();
                                        if !stdout.is_empty() {
                                            let _ = update_expected_output(
                                                path,
                                                &lesson.id,
                                                &exercise.id,
                                                stdout,
                                                Some(idx),
                                                verbose,
                                            );
                                        }
                                    }
                                }
                            } else {
                                let msg = format_failure(&result);
                                println!(
                                    "    {} stage '{}': {}",
                                    cli_fmt::red("\u{2717}"),
                                    stage.id,
                                    msg
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "    {} stage '{}': {}",
                                cli_fmt::red("\u{2717}"),
                                stage.id,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    println!();
    println!(
        "Results: {} passed, {} failed, {} skipped",
        cli_fmt::green(&passed.to_string()),
        if failed > 0 {
            cli_fmt::red(&failed.to_string())
        } else {
            "0".to_string()
        },
        skipped
    );

    if failed > 0 {
        Ok(exit_codes::ERROR)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}

/// Update the expected_output field in an exercise YAML file.
/// If `stage_idx` is Some, updates the stage's expected_output instead.
fn update_expected_output(
    course_path: &Path,
    lesson_id: &str,
    exercise_id: &str,
    new_output: &str,
    stage_idx: Option<usize>,
    verbose: bool,
) -> anyhow::Result<()> {
    let lessons_dir = course_path.join("lessons");
    let lesson_dir = course::loader::find_lesson_dir(&lessons_dir, lesson_id)?;
    let exercises_dir = lesson_dir.join("exercises");
    let yaml_path = course::loader::find_exercise_file(&exercises_dir, exercise_id)?;

    let contents = std::fs::read_to_string(&yaml_path)?;

    let updated = if let Some(idx) = stage_idx {
        update_stage_expected_output_in_yaml(&contents, idx, new_output)
    } else {
        update_expected_output_in_yaml(&contents, new_output)
    };

    match updated {
        Some(new_contents) => {
            std::fs::write(&yaml_path, new_contents)?;
            if verbose {
                let label = if let Some(idx) = stage_idx {
                    format!("stage[{}]", idx)
                } else {
                    "base".to_string()
                };
                println!(
                    "    {} Updated expected_output ({}) in {}",
                    cli_fmt::green("\u{2713}"),
                    label,
                    yaml_path.display()
                );
            }
        }
        None => {
            if verbose {
                println!(
                    "    {} Could not locate expected_output field to update",
                    cli_fmt::yellow("\u{26a0}")
                );
            }
        }
    }

    Ok(())
}

/// Text-based replacement of expected_output in YAML.
/// Handles both quoted and unquoted values.
fn update_expected_output_in_yaml(yaml: &str, new_output: &str) -> Option<String> {
    // Find `expected_output:` at the top level (not indented under stages)
    // We look for the FIRST occurrence that is inside a `validation:` block
    // but NOT inside a `stages:` block.
    let lines: Vec<&str> = yaml.lines().collect();
    let mut in_stages = false;
    let mut result_lines = Vec::new();
    let mut replaced = false;

    for line in &lines {
        let trimmed = line.trim();

        // Track whether we're in the stages block
        if trimmed == "stages:" || trimmed.starts_with("stages:") {
            in_stages = true;
        } else if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
            // Top-level key resets stages context
            if !trimmed.starts_with('-') && !trimmed.starts_with('#') {
                in_stages = false;
            }
        }

        if !in_stages && !replaced && trimmed.starts_with("expected_output:") {
            let indent = line.len() - line.trim_start().len();
            let prefix = &line[..indent];
            let needs_quotes = new_output.contains('"')
                || new_output.contains(':')
                || new_output.contains('#')
                || new_output.contains('\n');
            if needs_quotes {
                result_lines.push(format!(
                    "{}expected_output: '{}'",
                    prefix,
                    new_output.replace('\'', "''")
                ));
            } else {
                result_lines.push(format!("{}expected_output: \"{}\"", prefix, new_output));
            }
            replaced = true;
        } else {
            result_lines.push(line.to_string());
        }
    }

    if replaced {
        Some(result_lines.join("\n") + "\n")
    } else {
        None
    }
}

/// Text-based replacement of expected_output within a specific stage.
fn update_stage_expected_output_in_yaml(
    yaml: &str,
    stage_idx: usize,
    new_output: &str,
) -> Option<String> {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut result_lines = Vec::new();
    let mut in_stages = false;
    let mut current_stage: i32 = -1;
    let mut replaced = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed == "stages:" || trimmed.starts_with("stages:") {
            in_stages = true;
            current_stage = -1;
            result_lines.push(line.to_string());
            continue;
        }

        if in_stages {
            // Detect stage entries (list items starting with "- id:")
            if trimmed.starts_with("- id:") {
                current_stage += 1;
            }

            if current_stage == stage_idx as i32
                && !replaced
                && trimmed.starts_with("expected_output:")
            {
                let indent = line.len() - line.trim_start().len();
                let prefix = &line[..indent];
                let needs_quotes = new_output.contains('"')
                    || new_output.contains(':')
                    || new_output.contains('#')
                    || new_output.contains('\n');
                if needs_quotes {
                    result_lines.push(format!(
                        "{}expected_output: '{}'",
                        prefix,
                        new_output.replace('\'', "''")
                    ));
                } else {
                    result_lines.push(format!("{}expected_output: \"{}\"", prefix, new_output));
                }
                replaced = true;
                continue;
            }
        }

        result_lines.push(line.to_string());
    }

    if replaced {
        Some(result_lines.join("\n") + "\n")
    } else {
        None
    }
}

fn print_failure(result: &exec::runner::ExecutionResult, _verbose: bool) {
    match result {
        exec::runner::ExecutionResult::StepFailed {
            step_name, stderr, ..
        } => {
            println!(
                "    {} failed: {}",
                step_name,
                stderr.lines().next().unwrap_or("")
            );
        }
        exec::runner::ExecutionResult::ValidationFailed(vr) => match vr {
            exec::validate::ValidationResult::OutputMismatch { expected, actual } => {
                println!("    Expected: \"{}\"", expected);
                println!("    Actual:   \"{}\"", actual);
            }
            _ => println!("    Validation failed"),
        },
        exec::runner::ExecutionResult::Timeout { step_name } => {
            println!("    {} timed out", step_name);
        }
        _ => println!("    Failed"),
    }
}

fn format_failure(result: &exec::runner::ExecutionResult) -> String {
    match result {
        exec::runner::ExecutionResult::StepFailed {
            step_name, stderr, ..
        } => format!(
            "{} failed: {}",
            step_name,
            stderr.lines().next().unwrap_or("")
        ),
        exec::runner::ExecutionResult::ValidationFailed(vr) => match vr {
            exec::validate::ValidationResult::OutputMismatch { expected, actual } => {
                format!("expected \"{}\" got \"{}\"", expected, actual)
            }
            _ => "validation failed".to_string(),
        },
        exec::runner::ExecutionResult::Timeout { step_name } => {
            format!("{} timed out", step_name)
        }
        _ => "failed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_expected_output_basic() {
        let yaml = r#"id: test
validation:
  method: output
  expected_output: "old value"
hints:
  - "hint"
"#;
        let result = update_expected_output_in_yaml(yaml, "new value").unwrap();
        assert!(result.contains("expected_output: \"new value\""));
        assert!(!result.contains("old value"));
    }

    #[test]
    fn test_update_expected_output_with_special_chars() {
        let yaml = r#"id: test
validation:
  method: output
  expected_output: "old"
"#;
        let result = update_expected_output_in_yaml(yaml, "value: with colon").unwrap();
        assert!(result.contains("expected_output: 'value: with colon'"));
    }

    #[test]
    fn test_update_expected_output_no_field() {
        let yaml = r#"id: test
validation:
  method: compile-only
"#;
        assert!(update_expected_output_in_yaml(yaml, "value").is_none());
    }

    #[test]
    fn test_update_expected_output_skips_stages() {
        let yaml = r#"id: test
validation:
  method: output
  expected_output: "base"
stages:
  - id: s1
    validation:
      method: output
      expected_output: "stage1"
"#;
        let result = update_expected_output_in_yaml(yaml, "new base").unwrap();
        assert!(result.contains("expected_output: \"new base\""));
        // Stage expected_output should NOT be changed
        assert!(result.contains("expected_output: \"stage1\""));
    }

    #[test]
    fn test_update_stage_expected_output() {
        let yaml = r#"id: test
validation:
  method: output
  expected_output: "base"
stages:
  - id: s1
    validation:
      method: output
      expected_output: "old stage1"
  - id: s2
    validation:
      method: output
      expected_output: "old stage2"
"#;
        let result = update_stage_expected_output_in_yaml(yaml, 0, "new stage1").unwrap();
        assert!(result.contains("expected_output: \"new stage1\""));
        assert!(result.contains("expected_output: \"old stage2\""));
        assert!(result.contains("expected_output: \"base\""));

        let result = update_stage_expected_output_in_yaml(yaml, 1, "new stage2").unwrap();
        assert!(result.contains("expected_output: \"old stage1\""));
        assert!(result.contains("expected_output: \"new stage2\""));
    }
}
