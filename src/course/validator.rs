use super::types::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ValidationResult {
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl ValidationResult {
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}

pub fn validate_course(course: &Course) -> ValidationResult {
    let mut checks = Vec::new();

    // Check valid semver
    checks.push(check_semver(&course.version));

    // Check platform field
    checks.push(check_platform(&course.platform));

    // Check provision/runtime
    checks.extend(check_provision(&course.language));

    // Check all lessons referenced exist
    checks.push(check_lessons_exist(course));

    // Check no dependency cycles
    checks.push(check_no_cycles(course));

    // Check exercises
    for lesson in &course.loaded_lessons {
        for exercise in &lesson.loaded_exercises {
            checks.extend(check_exercise(exercise, &lesson.id));
        }
    }

    ValidationResult { checks }
}

fn check_platform(platform: &Option<String>) -> ValidationCheck {
    match platform {
        None => ValidationCheck {
            name: "platform field valid".to_string(),
            passed: true,
            message: "No platform restriction (runs everywhere)".to_string(),
        },
        Some(p) if KNOWN_PLATFORMS.contains(&p.as_str()) => ValidationCheck {
            name: "platform field valid".to_string(),
            passed: true,
            message: format!("Platform: {}", p),
        },
        Some(p) => ValidationCheck {
            name: "platform field valid".to_string(),
            passed: false,
            message: format!(
                "Unknown platform '{}'. Known values: {}",
                p,
                KNOWN_PLATFORMS.join(", ")
            ),
        },
    }
}

fn check_provision(language: &Language) -> Vec<ValidationCheck> {
    use super::types::Provision;
    let mut checks = Vec::new();

    match language.provision {
        Provision::Embedded => match &language.runtime {
            None => {
                checks.push(ValidationCheck {
                    name: "provision: embedded requires runtime".to_string(),
                    passed: false,
                    message: "provision is 'embedded' but no runtime specified".to_string(),
                });
            }
            Some(rt) if rt == "sqlite" => {
                checks.push(ValidationCheck {
                    name: "provision valid".to_string(),
                    passed: true,
                    message: "provision: embedded, runtime: sqlite".to_string(),
                });
            }
            Some(rt) => {
                checks.push(ValidationCheck {
                    name: "provision: embedded runtime known".to_string(),
                    passed: false,
                    message: format!("unknown embedded runtime '{}'. Known: sqlite", rt),
                });
            }
        },
        Provision::Auto => {
            checks.push(ValidationCheck {
                name: "provision valid".to_string(),
                passed: true,
                message: "provision: auto (system → portable fallback)".to_string(),
            });
        }
        Provision::System => {
            // Default, always fine
        }
        Provision::Manual => {
            checks.push(ValidationCheck {
                name: "provision valid".to_string(),
                passed: true,
                message: "provision: manual (user installs tools themselves)".to_string(),
            });
        }
    }

    checks
}

fn check_semver(version: &str) -> ValidationCheck {
    match semver::Version::parse(version) {
        Ok(_) => ValidationCheck {
            name: "course.yaml schema valid".to_string(),
            passed: true,
            message: format!("Version {} is valid semver", version),
        },
        Err(e) => ValidationCheck {
            name: "course.yaml schema valid".to_string(),
            passed: false,
            message: format!("Invalid semver '{}': {}", version, e),
        },
    }
}

fn check_lessons_exist(course: &Course) -> ValidationCheck {
    let loaded_ids: HashSet<&str> = course
        .loaded_lessons
        .iter()
        .map(|l| l.id.as_str())
        .collect();
    let referenced_ids: Vec<&str> = course.lessons.iter().map(|l| l.id.as_str()).collect();

    let missing: Vec<&&str> = referenced_ids
        .iter()
        .filter(|id| !loaded_ids.contains(**id))
        .collect();

    if missing.is_empty() {
        ValidationCheck {
            name: "All lessons referenced in course.yaml exist".to_string(),
            passed: true,
            message: format!("{} lessons found", loaded_ids.len()),
        }
    } else {
        ValidationCheck {
            name: "All lessons referenced in course.yaml exist".to_string(),
            passed: false,
            message: format!("Missing lessons: {:?}", missing),
        }
    }
}

fn check_no_cycles(course: &Course) -> ValidationCheck {
    let lesson_map: HashMap<&str, &LessonRef> =
        course.lessons.iter().map(|l| (l.id.as_str(), l)).collect();

    // Topological sort using DFS
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    for lesson_ref in &course.lessons {
        if !visited.contains(lesson_ref.id.as_str())
            && has_cycle(
                lesson_ref.id.as_str(),
                &lesson_map,
                &mut visited,
                &mut in_stack,
            )
        {
            return ValidationCheck {
                name: "No dependency cycles in lesson graph".to_string(),
                passed: false,
                message: "Dependency cycle detected in lesson graph".to_string(),
            };
        }
    }

    ValidationCheck {
        name: "No dependency cycles in lesson graph".to_string(),
        passed: true,
        message: "No cycles found".to_string(),
    }
}

fn has_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, &'a LessonRef>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
) -> bool {
    visited.insert(node);
    in_stack.insert(node);

    if let Some(lesson_ref) = graph.get(node) {
        for dep in &lesson_ref.requires {
            if !visited.contains(dep.as_str()) {
                if has_cycle(dep.as_str(), graph, visited, in_stack) {
                    return true;
                }
            } else if in_stack.contains(dep.as_str()) {
                return true;
            }
        }
    }

    in_stack.remove(node);
    false
}

fn check_exercise(exercise: &Exercise, lesson_id: &str) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();
    let prefix = format!("{}/{}", lesson_id, exercise.id);

    // Check hints exist
    checks.push(if exercise.hints.is_empty() {
        ValidationCheck {
            name: format!("{}: has hints", prefix),
            passed: false,
            message: "Exercise has no hints".to_string(),
        }
    } else {
        ValidationCheck {
            name: format!("{}: has hints", prefix),
            passed: true,
            message: format!("{} hints", exercise.hints.len()),
        }
    });

    // Check solution exists
    let has_solution = exercise.solution.is_some() || !exercise.solution_files.is_empty();
    checks.push(if has_solution {
        ValidationCheck {
            name: format!("{}: solution provided", prefix),
            passed: true,
            message: "Solution present".to_string(),
        }
    } else {
        ValidationCheck {
            name: format!("{}: solution provided", prefix),
            passed: false,
            message: "No solution provided".to_string(),
        }
    });

    // Check starter/files mutual exclusivity
    let has_starter = exercise.starter.is_some();
    let has_files = !exercise.files.is_empty();
    if has_starter && has_files {
        checks.push(ValidationCheck {
            name: format!("{}: starter/files exclusive", prefix),
            passed: false,
            message: "Exercise has both 'starter' and 'files' — use one or the other".to_string(),
        });
    }

    // Check that write/fix/fill-blank exercises have starter or files
    if matches!(
        exercise.exercise_type,
        ExerciseType::Write | ExerciseType::Fix | ExerciseType::FillBlank | ExerciseType::Command
    ) && !has_starter
        && !has_files
    {
        checks.push(ValidationCheck {
            name: format!("{}: has starter code", prefix),
            passed: false,
            message: "Code exercise needs 'starter' or 'files'".to_string(),
        });
    }

    // Check environment paths are safe
    if let Some(ref env) = exercise.environment {
        for file in &env.files {
            if let Err(msg) = check_env_path(&file.path) {
                checks.push(ValidationCheck {
                    name: format!("{}: environment path valid", prefix),
                    passed: false,
                    message: format!("file path '{}': {}", file.path, msg),
                });
            }
            // Check valid octal permissions
            if let Some(ref perm) = file.permissions {
                if u32::from_str_radix(perm, 8).is_err() || perm.len() > 4 {
                    checks.push(ValidationCheck {
                        name: format!("{}: environment permissions valid", prefix),
                        passed: false,
                        message: format!("invalid octal permissions '{}'", perm),
                    });
                }
            }
        }
        for dir in &env.dirs {
            if let Err(msg) = check_env_path(dir) {
                checks.push(ValidationCheck {
                    name: format!("{}: environment path valid", prefix),
                    passed: false,
                    message: format!("dir '{}': {}", dir, msg),
                });
            }
        }
        for sym in &env.symlinks {
            if let Err(msg) = check_env_path(&sym.link) {
                checks.push(ValidationCheck {
                    name: format!("{}: environment path valid", prefix),
                    passed: false,
                    message: format!("symlink '{}': {}", sym.link, msg),
                });
            }
            if let Err(msg) = check_env_path(&sym.target) {
                checks.push(ValidationCheck {
                    name: format!("{}: environment path valid", prefix),
                    passed: false,
                    message: format!("symlink target '{}': {}", sym.target, msg),
                });
            }
        }
        if let Some(ref cwd) = env.cwd {
            if let Err(msg) = check_env_path(cwd) {
                checks.push(ValidationCheck {
                    name: format!("{}: environment path valid", prefix),
                    passed: false,
                    message: format!("cwd '{}': {}", cwd, msg),
                });
            }
        }
    }

    // Check ports count
    if let Some(ref env) = exercise.environment {
        if env.ports > 10 {
            checks.push(ValidationCheck {
                name: format!("{}: ports count valid", prefix),
                passed: false,
                message: format!("ports must be 0-10, got {}", env.ports),
            });
        }
    }

    // Check setup/services/teardown in environment
    if let Some(ref env) = exercise.environment {
        for step in &env.setup {
            if step.name.is_empty() || step.command.is_empty() {
                checks.push(ValidationCheck {
                    name: format!("{}: setup step valid", prefix),
                    passed: false,
                    message: "setup step must have non-empty name and command".to_string(),
                });
            }
            if let Some(t) = step.timeout_seconds {
                if t == 0 || t > 60 {
                    checks.push(ValidationCheck {
                        name: format!("{}: setup timeout valid", prefix),
                        passed: false,
                        message: format!("setup '{}': timeout must be 1-60s, got {}", step.name, t),
                    });
                }
            }
            if let Some(ref path) = step.capture_to {
                if let Err(msg) = check_env_path(path) {
                    checks.push(ValidationCheck {
                        name: format!("{}: setup capture_to path valid", prefix),
                        passed: false,
                        message: format!("setup '{}' capture_to '{}': {}", step.name, path, msg),
                    });
                }
            }
            // Info: report what setup commands the course will run
            checks.push(ValidationCheck {
                name: format!("{}: setup runs '{}'", prefix, step.command),
                passed: true,
                message: format!("setup step '{}' runs '{}'", step.name, step.command),
            });
        }

        for svc in &env.services {
            if svc.name.is_empty() || svc.command.is_empty() {
                checks.push(ValidationCheck {
                    name: format!("{}: service valid", prefix),
                    passed: false,
                    message: "service must have non-empty name and command".to_string(),
                });
            }
            if let Some(ref pattern) = svc.ready_pattern {
                if regex::Regex::new(pattern).is_err() {
                    checks.push(ValidationCheck {
                        name: format!("{}: service ready_pattern valid", prefix),
                        passed: false,
                        message: format!(
                            "service '{}': invalid ready_pattern regex '{}'",
                            svc.name, pattern
                        ),
                    });
                }
            }
            if let Some(ref stream) = svc.ready_stream {
                if !["stdout", "stderr", "both"].contains(&stream.as_str()) {
                    checks.push(ValidationCheck {
                        name: format!("{}: service ready_stream valid", prefix),
                        passed: false,
                        message: format!(
                            "service '{}': ready_stream must be 'stdout', 'stderr', or 'both', got '{}'",
                            svc.name, stream
                        ),
                    });
                }
            }
            if let Some(ref path) = svc.capture_stdout {
                if let Err(msg) = check_env_path(path) {
                    checks.push(ValidationCheck {
                        name: format!("{}: service capture_stdout path valid", prefix),
                        passed: false,
                        message: format!(
                            "service '{}' capture_stdout '{}': {}",
                            svc.name, path, msg
                        ),
                    });
                }
            }
            if let Some(ref path) = svc.capture_stderr {
                if let Err(msg) = check_env_path(path) {
                    checks.push(ValidationCheck {
                        name: format!("{}: service capture_stderr path valid", prefix),
                        passed: false,
                        message: format!(
                            "service '{}' capture_stderr '{}': {}",
                            svc.name, path, msg
                        ),
                    });
                }
            }
            checks.push(ValidationCheck {
                name: format!("{}: service runs '{}' (background)", prefix, svc.command),
                passed: true,
                message: format!(
                    "service '{}' runs '{}' as background process",
                    svc.name, svc.command
                ),
            });
        }

        for step in &env.teardown {
            if step.name.is_empty() || step.command.is_empty() {
                checks.push(ValidationCheck {
                    name: format!("{}: teardown step valid", prefix),
                    passed: false,
                    message: "teardown step must have non-empty name and command".to_string(),
                });
            }
            if let Some(t) = step.timeout_seconds {
                if t == 0 || t > 60 {
                    checks.push(ValidationCheck {
                        name: format!("{}: teardown timeout valid", prefix),
                        passed: false,
                        message: format!(
                            "teardown '{}': timeout must be 1-60s, got {}",
                            step.name, t
                        ),
                    });
                }
            }
            if let Some(ref path) = step.capture_to {
                if let Err(msg) = check_env_path(path) {
                    checks.push(ValidationCheck {
                        name: format!("{}: teardown capture_to path valid", prefix),
                        passed: false,
                        message: format!("teardown '{}' capture_to '{}': {}", step.name, path, msg),
                    });
                }
            }
            checks.push(ValidationCheck {
                name: format!("{}: teardown runs '{}'", prefix, step.command),
                passed: true,
                message: format!("teardown step '{}' runs '{}'", step.name, step.command),
            });
        }
    }

    // Check state validation consistency
    if exercise.validation.method == ValidationMethod::State {
        let has_assertions = exercise
            .validation
            .assertions
            .as_ref()
            .is_some_and(|a| !a.is_empty());
        if !has_assertions {
            checks.push(ValidationCheck {
                name: format!("{}: state validation has assertions", prefix),
                passed: false,
                message: "method 'state' requires non-empty 'assertions' list".to_string(),
            });
        }
    }

    // Validate assertion paths and patterns
    if let Some(ref assertions) = exercise.validation.assertions {
        for (i, assertion) in assertions.iter().enumerate() {
            let assertion_prefix = format!("{}: assertion[{}]", prefix, i);
            match assertion {
                StateAssertion::FileExists(p)
                | StateAssertion::DirExists(p)
                | StateAssertion::FileNotExists(p)
                | StateAssertion::DirNotExists(p)
                | StateAssertion::DirEmpty(p) => {
                    if let Err(e) = check_env_path(p) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", p, e),
                        });
                    }
                }
                StateAssertion::FileContains(check) | StateAssertion::FileEquals(check) => {
                    if let Err(e) = check_env_path(&check.path) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", check.path, e),
                        });
                    }
                }
                StateAssertion::FileMatches(check) => {
                    if let Err(e) = check_env_path(&check.path) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", check.path, e),
                        });
                    }
                    if regex::Regex::new(&check.pattern).is_err() {
                        checks.push(ValidationCheck {
                            name: format!("{}: regex pattern", assertion_prefix),
                            passed: false,
                            message: format!("invalid regex pattern '{}'", check.pattern),
                        });
                    }
                }
                StateAssertion::Permissions(check) => {
                    if let Err(e) = check_env_path(&check.path) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", check.path, e),
                        });
                    }
                }
                StateAssertion::Symlink(check) => {
                    if let Err(e) = check_env_path(&check.path) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", check.path, e),
                        });
                    }
                    if let Err(e) = check_env_path(&check.target) {
                        checks.push(ValidationCheck {
                            name: format!("{}: target path safety", assertion_prefix),
                            passed: false,
                            message: format!("target '{}': {}", check.target, e),
                        });
                    }
                }
                StateAssertion::FileCount(check) => {
                    if let Err(e) = check_env_path(&check.path) {
                        checks.push(ValidationCheck {
                            name: format!("{}: path safety", assertion_prefix),
                            passed: false,
                            message: format!("path '{}': {}", check.path, e),
                        });
                    }
                }
            }
        }
    }

    // Check staged exercise constraints
    if !exercise.stages.is_empty() {
        // Stage IDs must be unique
        let mut seen_ids = HashSet::new();
        for stage in &exercise.stages {
            if !seen_ids.insert(&stage.id) {
                checks.push(ValidationCheck {
                    name: format!("{}: stage IDs unique", prefix),
                    passed: false,
                    message: format!("Duplicate stage ID '{}'", stage.id),
                });
            }
        }
        if seen_ids.len() == exercise.stages.len() {
            checks.push(ValidationCheck {
                name: format!("{}: stage IDs unique", prefix),
                passed: true,
                message: format!("{} unique stage IDs", exercise.stages.len()),
            });
        }

        // Validate each stage
        for (i, stage) in exercise.stages.iter().enumerate() {
            let stage_prefix = format!("{}/stage[{}:{}]", prefix, i, stage.id);

            // Stage must have hints
            checks.push(if stage.hints.is_empty() {
                ValidationCheck {
                    name: format!("{}: has hints", stage_prefix),
                    passed: false,
                    message: "Stage has no hints".to_string(),
                }
            } else {
                ValidationCheck {
                    name: format!("{}: has hints", stage_prefix),
                    passed: true,
                    message: format!("{} hints", stage.hints.len()),
                }
            });

            // Stage must have solution
            let has_solution = stage.solution.is_some() || !stage.solution_files.is_empty();
            checks.push(if has_solution {
                ValidationCheck {
                    name: format!("{}: solution provided", stage_prefix),
                    passed: true,
                    message: "Solution present".to_string(),
                }
            } else {
                ValidationCheck {
                    name: format!("{}: solution provided", stage_prefix),
                    passed: false,
                    message: "No solution provided for stage".to_string(),
                }
            });

            // Stage state validation must have assertions
            if stage.validation.method == ValidationMethod::State {
                let has_assertions = stage
                    .validation
                    .assertions
                    .as_ref()
                    .is_some_and(|a| !a.is_empty());
                if !has_assertions {
                    checks.push(ValidationCheck {
                        name: format!("{}: state validation has assertions", stage_prefix),
                        passed: false,
                        message: "method 'state' requires non-empty 'assertions' list".to_string(),
                    });
                }
            }
        }
    }

    checks
}

/// Validate that a path is relative and has no `..` components.
fn check_env_path(path: &str) -> std::result::Result<(), String> {
    use std::path::{Component, Path};
    let p = Path::new(path);
    if p.is_absolute() {
        return Err("absolute paths not allowed".to_string());
    }
    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err("'..' components not allowed".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_course() -> Course {
        Course {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: None,
            platform: None,
            language: Language {
                id: "cpp".to_string(),
                display_name: "C++".to_string(),
                extension: ".cpp".to_string(),
                steps: vec![],
                limits: ExecutionLimits::default(),
                provision: Provision::default(),
                runtime: None,
            },
            lessons: vec![
                LessonRef {
                    id: "a".to_string(),
                    title: "A".to_string(),
                    requires: vec![],
                },
                LessonRef {
                    id: "b".to_string(),
                    title: "B".to_string(),
                    requires: vec!["a".to_string()],
                },
            ],
            estimated_minutes_per_lesson: None,
            loaded_lessons: vec![
                Lesson {
                    id: "a".to_string(),
                    title: "A".to_string(),
                    description: None,
                    estimated_minutes: None,
                    content: "content.md".to_string(),
                    exercises: vec![],
                    teaches: vec![],
                    recap: None,
                    loaded_exercises: vec![],
                    content_markdown: String::new(),
                    content_sections: vec![],
                },
                Lesson {
                    id: "b".to_string(),
                    title: "B".to_string(),
                    description: None,
                    estimated_minutes: None,
                    content: "content.md".to_string(),
                    exercises: vec![],
                    teaches: vec![],
                    recap: None,
                    loaded_exercises: vec![],
                    content_markdown: String::new(),
                    content_sections: vec![],
                },
            ],
            source_dir: std::path::PathBuf::new(),
        }
    }

    #[test]
    fn test_valid_course_passes() {
        let course = make_test_course();
        let result = validate_course(&course);
        assert!(result.all_passed());
    }

    #[test]
    fn test_cycle_detection() {
        let mut course = make_test_course();
        // Create cycle: a requires b, b requires a
        course.lessons[0].requires = vec!["b".to_string()];
        let result = validate_course(&course);
        let cycle_check = result
            .checks
            .iter()
            .find(|c| c.name.contains("cycle"))
            .unwrap();
        assert!(!cycle_check.passed);
    }

    #[test]
    fn test_bad_semver() {
        let mut course = make_test_course();
        course.version = "not-a-version".to_string();
        let result = validate_course(&course);
        let semver_check = result
            .checks
            .iter()
            .find(|c| c.name.contains("schema"))
            .unwrap();
        assert!(!semver_check.passed);
    }

    #[test]
    fn test_platform_none_passes() {
        let course = make_test_course();
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("platform"))
            .unwrap();
        assert!(check.passed);
    }

    #[test]
    fn test_platform_known_passes() {
        let mut course = make_test_course();
        course.platform = Some("linux".to_string());
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("platform"))
            .unwrap();
        assert!(check.passed);
    }

    #[test]
    fn test_platform_unknown_fails() {
        let mut course = make_test_course();
        course.platform = Some("beos".to_string());
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("platform"))
            .unwrap();
        assert!(!check.passed);
        assert!(check.message.contains("beos"));
    }

    #[test]
    fn test_provision_embedded_requires_runtime() {
        let mut course = make_test_course();
        course.language.provision = Provision::Embedded;
        course.language.runtime = None;
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("provision"))
            .unwrap();
        assert!(!check.passed);
        assert!(check.message.contains("no runtime"));
    }

    #[test]
    fn test_provision_embedded_sqlite_passes() {
        let mut course = make_test_course();
        course.language.provision = Provision::Embedded;
        course.language.runtime = Some("sqlite".to_string());
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("provision"))
            .unwrap();
        assert!(check.passed);
    }

    #[test]
    fn test_provision_embedded_unknown_runtime_fails() {
        let mut course = make_test_course();
        course.language.provision = Provision::Embedded;
        course.language.runtime = Some("lua".to_string());
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("provision"))
            .unwrap();
        assert!(!check.passed);
        assert!(check.message.contains("lua"));
    }

    #[test]
    fn test_provision_auto_passes() {
        let mut course = make_test_course();
        course.language.provision = Provision::Auto;
        let result = validate_course(&course);
        let check = result
            .checks
            .iter()
            .find(|c| c.name.contains("provision"))
            .unwrap();
        assert!(check.passed);
    }

    fn make_valid_staged_exercise() -> Exercise {
        Exercise {
            id: "staged-ex".to_string(),
            title: "Staged Exercise".to_string(),
            exercise_type: ExerciseType::Write,
            prompt: "Do it in stages".to_string(),
            starter: Some("// code".to_string()),
            files: vec![],
            main_file: None,
            input: None,
            validation: Validation {
                method: ValidationMethod::Output,
                expected_output: Some("base".to_string()),
                pattern: None,
                script: None,
                assertions: None,
            },
            hints: vec!["base hint".to_string()],
            solution: Some("base solution".to_string()),
            solution_files: vec![],
            explanation: None,
            environment: None,
            golf: false,
            stages: vec![
                ExerciseStage {
                    id: "basic".to_string(),
                    title: "Basic".to_string(),
                    prompt: Some("Basic version".to_string()),
                    validation: Validation {
                        method: ValidationMethod::Output,
                        expected_output: Some("basic".to_string()),
                        pattern: None,
                        script: None,
                        assertions: None,
                    },
                    hints: vec!["basic hint".to_string()],
                    solution: Some("basic solution".to_string()),
                    solution_files: vec![],
                    explanation: Some("Explanation".to_string()),
                    additional_files: vec![],
                },
                ExerciseStage {
                    id: "advanced".to_string(),
                    title: "Advanced".to_string(),
                    prompt: Some("Advanced version".to_string()),
                    validation: Validation {
                        method: ValidationMethod::Regex,
                        expected_output: None,
                        pattern: Some("done".to_string()),
                        script: None,
                        assertions: None,
                    },
                    hints: vec!["advanced hint".to_string()],
                    solution: Some("advanced solution".to_string()),
                    solution_files: vec![],
                    explanation: None,
                    additional_files: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_valid_staged_exercise_passes() {
        let exercise = make_valid_staged_exercise();
        let checks = check_exercise(&exercise, "lesson-1");
        assert!(
            checks.iter().all(|c| c.passed),
            "Failed checks: {:?}",
            checks.iter().filter(|c| !c.passed).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_staged_exercise_duplicate_ids() {
        let mut exercise = make_valid_staged_exercise();
        exercise.stages[1].id = "basic".to_string(); // duplicate
        let checks = check_exercise(&exercise, "lesson-1");
        let dup_check = checks
            .iter()
            .find(|c| c.name.contains("stage IDs unique") && !c.passed)
            .expect("Should have a failing unique ID check");
        assert!(dup_check.message.contains("Duplicate"));
    }

    #[test]
    fn test_staged_exercise_missing_hints() {
        let mut exercise = make_valid_staged_exercise();
        exercise.stages[0].hints.clear();
        let checks = check_exercise(&exercise, "lesson-1");
        let hint_check = checks
            .iter()
            .find(|c| c.name.contains("stage[0:basic]") && c.name.contains("hints") && !c.passed);
        assert!(hint_check.is_some(), "Should flag missing stage hints");
    }

    #[test]
    fn test_staged_exercise_missing_solution() {
        let mut exercise = make_valid_staged_exercise();
        exercise.stages[1].solution = None;
        exercise.stages[1].solution_files.clear();
        let checks = check_exercise(&exercise, "lesson-1");
        let sol_check = checks.iter().find(|c| {
            c.name.contains("stage[1:advanced]") && c.name.contains("solution") && !c.passed
        });
        assert!(sol_check.is_some(), "Should flag missing stage solution");
    }

    #[test]
    fn test_staged_exercise_state_validation_needs_assertions() {
        let mut exercise = make_valid_staged_exercise();
        exercise.stages[0].validation.method = ValidationMethod::State;
        exercise.stages[0].validation.assertions = None;
        let checks = check_exercise(&exercise, "lesson-1");
        let assertion_check = checks.iter().find(|c| {
            c.name.contains("stage[0:basic]") && c.name.contains("state validation") && !c.passed
        });
        assert!(
            assertion_check.is_some(),
            "Should flag missing assertions for state validation"
        );
    }
}
