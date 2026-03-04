use std::io::BufRead;
use std::thread::JoinHandle;

use crate::course::types::{Course, Exercise, ExerciseFile, ExerciseType, ValidationMethod};
use crate::error::Result;
use crate::exec::embedded;
use crate::exec::environment;
use crate::exec::placeholder::substitute;
use crate::exec::provision::{self, ToolchainResolution};
use crate::exec::sandbox::{Sandbox, SandboxLevel, StepOutput};
use crate::exec::validate::{self, ValidationResult};

#[derive(Debug)]
pub struct RunOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub step_failed: Option<String>,
    pub timed_out: bool,
    pub teardown_warnings: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ExecutionResult {
    Success,
    CompileSuccess,
    StepFailed {
        step_name: String,
        stderr: String,
        exit_code: i32,
    },
    ValidationFailed(ValidationResult),
    Timeout {
        step_name: String,
    },
    SetupFailed {
        step_name: String,
        stderr: String,
        exit_code: i32,
    },
    ServiceFailed {
        service_name: String,
        error: String,
    },
    Error(String),
}

/// RAII guard that kills all background service processes on drop.
struct ServiceGuard(Vec<(String, std::process::Child)>);

impl Drop for ServiceGuard {
    fn drop(&mut self) {
        for (_, child) in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            ExecutionResult::Success | ExecutionResult::CompileSuccess
        )
    }
}

/// Spawn drain threads for any remaining un-consumed stdout/stderr handles on a service child.
/// Prevents pipe backup that could cause the service to hang.
/// If capture paths are provided, writes output to those files instead of discarding.
/// Safe to call unconditionally — `take()` returns `None` if the handle was already consumed.
/// Returns join handles so callers can wait for capture files to be written.
pub(crate) fn drain_service_pipes(
    child: &mut std::process::Child,
    sandbox_dir: &std::path::Path,
    capture_stdout: Option<&str>,
    capture_stderr: Option<&str>,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        let capture_path = capture_stdout.map(|p| sandbox_dir.join(p));
        handles.push(std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            if let Some(path) = capture_path {
                let mut content = String::new();
                for line in reader.lines().map_while(|l| l.ok()) {
                    content.push_str(&line);
                    content.push('\n');
                }
                let _ = std::fs::write(path, content);
            } else {
                for _ in reader.lines() {}
            }
        }));
    }
    if let Some(stderr) = child.stderr.take() {
        let capture_path = capture_stderr.map(|p| sandbox_dir.join(p));
        handles.push(std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stderr);
            if let Some(path) = capture_path {
                let mut content = String::new();
                for line in reader.lines().map_while(|l| l.ok()) {
                    content.push_str(&line);
                    content.push('\n');
                }
                let _ = std::fs::write(path, content);
            } else {
                for _ in reader.lines() {}
            }
        }));
    }
    handles
}

/// Strip sandbox temp directory prefix from compiler output so paths show as relative filenames.
pub(crate) fn clean_sandbox_paths(text: &str, sandbox_dir: &std::path::Path) -> String {
    let dir_str = format!("{}/", sandbox_dir.to_string_lossy());
    text.replace(&dir_str, "")
}

// --- Shared lifecycle ---

/// Errors from the exercise lifecycle pipeline (distinct from Rust I/O errors).
/// Each variant carries teardown_warnings so they're never lost on failure.
enum LifecycleError {
    SetupFailed {
        step_name: String,
        stderr: String,
        exit_code: i32,
        timed_out: bool,
        teardown_warnings: Vec<String>,
    },
    ServiceFailed {
        service_name: String,
        error: String,
        teardown_warnings: Vec<String>,
    },
    StepFailed {
        step_name: String,
        stderr: String,
        exit_code: i32,
        teardown_warnings: Vec<String>,
    },
    Timeout {
        step_name: String,
        stderr: String,
        teardown_warnings: Vec<String>,
    },
}

/// Successful output from the exercise lifecycle pipeline.
struct LifecycleOutput {
    last_output: StepOutput,
    teardown_warnings: Vec<String>,
}

/// Run the complete exercise lifecycle: env setup → setup commands → services →
/// student files → language steps → teardown → kill services.
///
/// Both `run_exercise_with_sandbox` and `execute_exercise_with_sandbox` delegate here,
/// then map the result to their own return types.
fn run_lifecycle(
    course: &Course,
    exercise: &Exercise,
    user_files: &[ExerciseFile],
    sandbox: &Sandbox,
) -> Result<std::result::Result<LifecycleOutput, LifecycleError>> {
    let file_names: Vec<String> = user_files.iter().map(|f| f.name.clone()).collect();
    let main_file = exercise.get_main_file(&course.language.extension);

    // 1. Set up environment (dirs, files, symlinks, env vars, cwd)
    let (env_vars, cwd_override) = if let Some(ref env_spec) = exercise.environment {
        let setup =
            environment::setup_environment(sandbox.dir(), env_spec, &main_file, &file_names)?;
        (Some(setup.env_vars), setup.cwd_override)
    } else {
        (None, None)
    };

    // 2. Run setup commands — capture error but don't return early
    let mut phase_error: Option<LifecycleError> = None;
    let mut service_guard = ServiceGuard(Vec::new());

    if let Some(ref env_spec) = exercise.environment {
        for step in &env_spec.setup {
            let output = environment::run_env_command(
                sandbox,
                step,
                env_vars.as_ref(),
                cwd_override.as_deref(),
                course.language.limits.timeout_seconds,
            )?;
            if output.exit_code != 0 {
                phase_error = Some(LifecycleError::SetupFailed {
                    step_name: step.name.clone(),
                    stderr: clean_sandbox_paths(&output.stderr, sandbox.dir()),
                    exit_code: output.exit_code,
                    timed_out: output.timed_out,
                    teardown_warnings: Vec::new(), // filled after teardown
                });
                break;
            }
        }
    }

    // 3. Start background services (skip if setup failed)
    let mut drain_handles: Vec<JoinHandle<()>> = Vec::new();
    if phase_error.is_none() {
        if let Some(ref env_spec) = exercise.environment {
            for svc in &env_spec.services {
                let svc_args: Vec<String> = svc
                    .args
                    .iter()
                    .map(|a| a.replace("{dir}", &sandbox.dir().to_string_lossy()))
                    .collect();
                let mut child = sandbox.spawn_service(
                    &svc.command,
                    &svc_args,
                    env_vars.as_ref(),
                    cwd_override.as_deref(),
                )?;
                match environment::wait_for_service_ready(&mut child, svc, sandbox.dir()) {
                    Ok(reader_handles) => {
                        drain_handles.extend(reader_handles);
                        let handles = drain_service_pipes(
                            &mut child,
                            sandbox.dir(),
                            svc.capture_stdout.as_deref(),
                            svc.capture_stderr.as_deref(),
                        );
                        drain_handles.extend(handles);
                        service_guard.0.push((svc.name.clone(), child));
                    }
                    Err(e) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        phase_error = Some(LifecycleError::ServiceFailed {
                            service_name: svc.name.clone(),
                            error: format!("{}", e),
                            teardown_warnings: Vec::new(),
                        });
                        break;
                    }
                }
            }
        }
    }

    // 4. Write student files after environment setup (skip if earlier phase failed)
    if phase_error.is_none() {
        for file in user_files {
            sandbox.write_file(&file.name, &file.content)?;
        }
    }

    // 5. Run student code (skip if earlier phase failed)
    let mut last_output = StepOutput::default();

    if phase_error.is_none() {
        if exercise.exercise_type == ExerciseType::Command {
            // Command exercises: run the student's script with sh directly.
            // No provision resolution or language steps needed — sh is always available.
            let needs_loopback = exercise
                .environment
                .as_ref()
                .is_some_and(|env| !env.services.is_empty());

            let script_path = format!("{}/{}", sandbox.dir().to_string_lossy(), main_file);
            let output = sandbox.run_command_with_loopback(
                "sh",
                &[script_path],
                exercise.input.as_deref(),
                env_vars.as_ref(),
                cwd_override.as_deref(),
                needs_loopback,
            )?;

            if output.timed_out {
                phase_error = Some(LifecycleError::Timeout {
                    step_name: "command".to_string(),
                    stderr: clean_sandbox_paths(&output.stderr, sandbox.dir()),
                    teardown_warnings: Vec::new(),
                });
            } else if output.exit_code != 0 {
                phase_error = Some(LifecycleError::StepFailed {
                    step_name: "command".to_string(),
                    stderr: clean_sandbox_paths(&output.stderr, sandbox.dir()),
                    exit_code: output.exit_code,
                    teardown_warnings: Vec::new(),
                });
            } else {
                last_output = output;
            }
        } else {
            // Normal exercises: resolve toolchain and run language steps
            let resolution = provision::resolve_toolchain(&course.language);

            match resolution {
                ToolchainResolution::Embedded(ref runtime) if runtime == "sqlite" => {
                    // Find setup.sql in environment files
                    let setup_sql = exercise.environment.as_ref().and_then(|env| {
                        env.files
                            .iter()
                            .find(|f| f.path.ends_with(".sql") && f.path != main_file)
                            .map(|f| f.content.clone())
                    });

                    // Read the student's SQL file from sandbox
                    let student_sql =
                        if let Some(file) = user_files.iter().find(|f| f.name == main_file) {
                            file.content.clone()
                        } else {
                            String::new()
                        };

                    let result = embedded::execute_sql(setup_sql.as_deref(), &student_sql)?;

                    last_output = StepOutput {
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit_code: result.exit_code,
                        timed_out: false,
                    };

                    if result.exit_code != 0 {
                        phase_error = Some(LifecycleError::StepFailed {
                            step_name: "sql-execute".to_string(),
                            stderr: last_output.stderr.clone(),
                            exit_code: result.exit_code,
                            teardown_warnings: Vec::new(),
                        });
                    }
                }
                ToolchainResolution::NotAvailable(msg) => {
                    phase_error = Some(LifecycleError::SetupFailed {
                        step_name: "toolchain".to_string(),
                        stderr: msg,
                        exit_code: 127,
                        timed_out: false,
                        teardown_warnings: Vec::new(),
                    });
                }
                _ => {
                    // System or Portable — run normal language steps
                    let extra_path = if let ToolchainResolution::Portable(ref bin_dir) = resolution
                    {
                        Some(bin_dir.clone())
                    } else {
                        None
                    };

                    let needs_loopback = exercise
                        .environment
                        .as_ref()
                        .is_some_and(|env| !env.services.is_empty());

                    for step in &course.language.steps {
                        let command =
                            substitute(&step.command, sandbox.dir(), &main_file, &file_names);
                        let args: Vec<String> = step
                            .args
                            .iter()
                            .map(|a| substitute(a, sandbox.dir(), &main_file, &file_names))
                            .collect();

                        let stdin_input = if step.capture_output {
                            exercise.input.as_deref()
                        } else {
                            None
                        };

                        // If we have a portable toolchain, prepend its bin dir to PATH
                        let step_env_vars = if let Some(ref bin_dir) = extra_path {
                            let path_prefix = bin_dir.to_string_lossy().to_string();
                            let mut vars = env_vars.clone().unwrap_or_default();
                            let existing_path = vars
                                .get("PATH")
                                .cloned()
                                .unwrap_or_else(|| std::env::var("PATH").unwrap_or_default());
                            vars.insert(
                                "PATH".to_string(),
                                format!("{}:{}", path_prefix, existing_path),
                            );
                            Some(vars)
                        } else {
                            env_vars.clone()
                        };

                        let output = sandbox.run_command_with_loopback(
                            &command,
                            &args,
                            stdin_input,
                            step_env_vars.as_ref(),
                            cwd_override.as_deref(),
                            needs_loopback,
                        )?;

                        if output.timed_out {
                            phase_error = Some(LifecycleError::Timeout {
                                step_name: step.name.clone(),
                                stderr: clean_sandbox_paths(&output.stderr, sandbox.dir()),
                                teardown_warnings: Vec::new(),
                            });
                            break;
                        }

                        if step.check_exit_code && output.exit_code != 0 {
                            phase_error = Some(LifecycleError::StepFailed {
                                step_name: step.name.clone(),
                                stderr: clean_sandbox_paths(&output.stderr, sandbox.dir()),
                                exit_code: output.exit_code,
                                teardown_warnings: Vec::new(),
                            });
                            break;
                        }

                        if step.capture_output {
                            last_output = output;
                        }
                    }
                }
            }
        } // end else (non-Command exercises)
    }

    // 6. Run teardown commands UNCONDITIONALLY — collect warnings on failure
    let mut teardown_warnings = Vec::new();
    if let Some(ref env_spec) = exercise.environment {
        for step in &env_spec.teardown {
            match environment::run_env_command_full(
                sandbox,
                step,
                env_vars.as_ref(),
                cwd_override.as_deref(),
                course.language.limits.timeout_seconds,
                &main_file,
                &file_names,
            ) {
                Ok(output) => {
                    if let Some(ref capture_path) = step.capture_to {
                        let _ = sandbox.write_file(capture_path, &output.stdout);
                    }
                    if output.exit_code != 0 {
                        teardown_warnings.push(format!(
                            "teardown '{}' failed (exit {}): {}",
                            step.name,
                            output.exit_code,
                            output.stderr.trim()
                        ));
                    }
                }
                Err(e) => {
                    teardown_warnings.push(format!("teardown '{}' error: {}", step.name, e));
                }
            }
        }
    }

    // 7. Kill all services (ServiceGuard dropped here), then wait for drain threads
    drop(service_guard);
    for handle in drain_handles {
        let _ = handle.join();
    }

    // Return error with teardown warnings attached, or success
    if let Some(mut err) = phase_error {
        match &mut err {
            LifecycleError::SetupFailed {
                teardown_warnings: tw,
                ..
            }
            | LifecycleError::ServiceFailed {
                teardown_warnings: tw,
                ..
            }
            | LifecycleError::StepFailed {
                teardown_warnings: tw,
                ..
            }
            | LifecycleError::Timeout {
                teardown_warnings: tw,
                ..
            } => {
                *tw = teardown_warnings;
            }
        }
        Ok(Err(err))
    } else {
        Ok(Ok(LifecycleOutput {
            last_output,
            teardown_warnings,
        }))
    }
}

// --- Public API ---

pub fn run_exercise_with_sandbox(
    course: &Course,
    exercise: &Exercise,
    user_files: &[ExerciseFile],
    sandbox_level: SandboxLevel,
) -> Result<RunOutput> {
    let sandbox = Sandbox::new(&course.language.limits, sandbox_level)?;

    match run_lifecycle(course, exercise, user_files, &sandbox)? {
        Ok(output) => Ok(RunOutput {
            stdout: output.last_output.stdout,
            stderr: clean_sandbox_paths(&output.last_output.stderr, sandbox.dir()),
            success: true,
            step_failed: None,
            timed_out: false,
            teardown_warnings: output.teardown_warnings,
        }),
        Err(LifecycleError::SetupFailed {
            step_name,
            stderr,
            timed_out,
            teardown_warnings,
            ..
        }) => Ok(RunOutput {
            stdout: String::new(),
            stderr,
            success: false,
            step_failed: Some(format!("setup: {}", step_name)),
            timed_out,
            teardown_warnings,
        }),
        Err(LifecycleError::ServiceFailed {
            service_name,
            error,
            teardown_warnings,
        }) => Ok(RunOutput {
            stdout: String::new(),
            stderr: error,
            success: false,
            step_failed: Some(format!("service: {}", service_name)),
            timed_out: false,
            teardown_warnings,
        }),
        Err(LifecycleError::StepFailed {
            step_name,
            stderr,
            teardown_warnings,
            ..
        }) => Ok(RunOutput {
            stdout: String::new(),
            stderr,
            success: false,
            step_failed: Some(step_name),
            timed_out: false,
            teardown_warnings,
        }),
        Err(LifecycleError::Timeout {
            step_name,
            stderr,
            teardown_warnings,
        }) => Ok(RunOutput {
            stdout: String::new(),
            stderr,
            success: false,
            step_failed: Some(step_name),
            timed_out: true,
            teardown_warnings,
        }),
    }
}

pub fn execute_exercise(
    course: &Course,
    exercise: &Exercise,
    user_files: &[ExerciseFile],
) -> Result<(ExecutionResult, Vec<String>)> {
    execute_exercise_with_sandbox(course, exercise, user_files, SandboxLevel::Basic)
}

pub fn execute_exercise_with_sandbox(
    course: &Course,
    exercise: &Exercise,
    user_files: &[ExerciseFile],
    sandbox_level: SandboxLevel,
) -> Result<(ExecutionResult, Vec<String>)> {
    let sandbox = Sandbox::new(&course.language.limits, sandbox_level)?;

    match run_lifecycle(course, exercise, user_files, &sandbox)? {
        Ok(output) => {
            // Run validation on successful execution
            let teardown_warnings = output.teardown_warnings;

            if exercise.validation.method == ValidationMethod::State {
                if let Some(ref assertions) = exercise.validation.assertions {
                    let results = environment::validate_state(sandbox.dir(), assertions);
                    let all_passed = results.iter().all(|r| r.passed);

                    if !all_passed {
                        return Ok((
                            ExecutionResult::ValidationFailed(
                                ValidationResult::StateAssertionFailed { results },
                            ),
                            teardown_warnings,
                        ));
                    }

                    // Also check expected_output if present (combined validation)
                    if exercise.validation.expected_output.is_some() {
                        let output_result =
                            validate::validate_output(&exercise.validation, &output.last_output);
                        if !output_result.is_success() {
                            return Ok((
                                ExecutionResult::ValidationFailed(output_result),
                                teardown_warnings,
                            ));
                        }
                    }

                    return Ok((ExecutionResult::Success, teardown_warnings));
                }
            }

            // Standard output-based validation
            let validation_result =
                validate::validate_output(&exercise.validation, &output.last_output);

            if validation_result.is_success() {
                match validation_result {
                    ValidationResult::CompileSuccess => {
                        Ok((ExecutionResult::CompileSuccess, teardown_warnings))
                    }
                    _ => Ok((ExecutionResult::Success, teardown_warnings)),
                }
            } else {
                Ok((
                    ExecutionResult::ValidationFailed(validation_result),
                    teardown_warnings,
                ))
            }
        }
        Err(LifecycleError::SetupFailed {
            step_name,
            stderr,
            exit_code,
            teardown_warnings,
            ..
        }) => Ok((
            ExecutionResult::SetupFailed {
                step_name,
                stderr,
                exit_code,
            },
            teardown_warnings,
        )),
        Err(LifecycleError::ServiceFailed {
            service_name,
            error,
            teardown_warnings,
        }) => Ok((
            ExecutionResult::ServiceFailed {
                service_name,
                error,
            },
            teardown_warnings,
        )),
        Err(LifecycleError::StepFailed {
            step_name,
            stderr,
            exit_code,
            teardown_warnings,
        }) => Ok((
            ExecutionResult::StepFailed {
                step_name,
                stderr,
                exit_code,
            },
            teardown_warnings,
        )),
        Err(LifecycleError::Timeout {
            step_name,
            teardown_warnings,
            ..
        }) => Ok((ExecutionResult::Timeout { step_name }, teardown_warnings)),
    }
}
