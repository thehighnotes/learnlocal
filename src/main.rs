mod cli;
mod cli_fmt;
mod config;
mod course;
mod error;
mod exec;
mod exit_codes;
mod llm;
mod state;
mod ui;

use clap::{CommandFactory, Parser};
use std::path::{Path, PathBuf};

use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let code = match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", cli_fmt::red("Error:"), e);
            exit_codes::ERROR
        }
    };

    std::process::exit(code);
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    let config = config::Config::load();
    let verbose = cli.verbose;

    match cli.command {
        None => cmd_home(&cli.courses_dir, &config).map(|()| exit_codes::SUCCESS),
        Some(Command::List) => cmd_list(&cli.courses_dir).map(|()| exit_codes::SUCCESS),
        Some(Command::Start { course, lesson }) => {
            cmd_start(&cli.courses_dir, &course, lesson.as_deref(), &config)
                .map(|()| exit_codes::SUCCESS)
        }
        Some(Command::Progress { course }) => {
            cmd_progress(&course, &cli.courses_dir).map(|()| exit_codes::SUCCESS)
        }
        Some(Command::Reset { course }) => cmd_reset(&course).map(|()| exit_codes::SUCCESS),
        Some(Command::Validate {
            path,
            run_solutions,
        }) => cmd_validate(&path, run_solutions, verbose),
        Some(Command::Completions { shell }) => {
            cmd_completions(shell).map(|()| exit_codes::SUCCESS)
        }
        Some(Command::Man) => cmd_man().map(|()| exit_codes::SUCCESS),
        Some(Command::Doctor) => cmd_doctor(&cli.courses_dir),
        Some(Command::Init { name }) => cmd_init(&name).map(|()| exit_codes::SUCCESS),
        Some(Command::Export { course, format }) => {
            cmd_export(course.as_deref(), &format).map(|()| exit_codes::SUCCESS)
        }
    }
}

fn discover_courses_dir(custom: &Option<PathBuf>) -> PathBuf {
    if let Some(dir) = custom {
        return dir.clone();
    }

    // Try relative to the executable
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(Path::new("."));
        let courses = exe_dir.join("courses");
        if courses.exists() {
            return courses;
        }
        // Try one level up (for target/debug/learnlocal)
        if let Some(parent) = exe_dir.parent() {
            let courses = parent.join("courses");
            if courses.exists() {
                return courses;
            }
            // Try two levels up (for target/debug/)
            if let Some(grandparent) = parent.parent() {
                let courses = grandparent.join("courses");
                if courses.exists() {
                    return courses;
                }
            }
        }
    }

    // Try current directory
    let cwd_courses = PathBuf::from("courses");
    if cwd_courses.exists() {
        return cwd_courses;
    }

    // Default
    PathBuf::from("courses")
}

/// Discover courses and launch the Home screen TUI.
fn cmd_home(courses_dir: &Option<PathBuf>, config: &config::Config) -> anyhow::Result<()> {
    let dir = discover_courses_dir(courses_dir);

    let mut course_infos = Vec::new();
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let course_yaml = entry.path().join("course.yaml");
                    if course_yaml.exists() {
                        match course::load_course_info(&entry.path()) {
                            Ok(info) => course_infos.push(info),
                            Err(e) => {
                                eprintln!("Warning: skipping {}: {}", entry.path().display(), e)
                            }
                        }
                    }
                }
            }
        }
    }

    let progress_store = state::ProgressStore::load()?;
    let sandbox_level = exec::sandbox::SandboxLevel::detect(&config.sandbox_level);

    let mut terminal = ui::terminal::setup()?;
    let mut app = ui::app::App::new(
        course_infos,
        progress_store,
        config.clone(),
        sandbox_level,
        dir,
    );

    let result = app.run(&mut terminal);
    ui::terminal::restore_terminal()?;

    result?;
    Ok(())
}

fn cmd_list(courses_dir: &Option<PathBuf>) -> anyhow::Result<()> {
    let dir = discover_courses_dir(courses_dir);

    if !dir.exists() {
        println!("No courses directory found at {}", dir.display());
        println!("Use --courses to specify a custom directory.");
        return Ok(());
    }

    println!("Available courses:");

    let mut found = false;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let course_yaml = entry.path().join("course.yaml");
            if course_yaml.exists() {
                match course::load_course_info(&entry.path()) {
                    Ok(info) => {
                        println!(
                            "  {:<24} {} v{} ({} lessons)",
                            info.dir_name, info.name, info.version, info.lesson_count
                        );
                        found = true;
                    }
                    Err(e) => {
                        println!(
                            "  {:<24} (error loading: {})",
                            entry.file_name().to_string_lossy(),
                            e
                        );
                    }
                }
            }
        }
    }

    if !found {
        println!("  (no courses found)");
    }

    Ok(())
}

fn cmd_start(
    courses_dir: &Option<PathBuf>,
    course_name: &str,
    lesson: Option<&str>,
    config: &config::Config,
) -> anyhow::Result<()> {
    let dir = discover_courses_dir(courses_dir);
    let course_path = dir.join(course_name);

    if !course_path.exists() {
        anyhow::bail!("Course '{}' not found in {}", course_name, dir.display());
    }

    let c = course::load_course(&course_path)?;

    // Check platform requirement
    let platform_status = exec::toolcheck::check_platform(&c.platform);
    if !platform_status.supported {
        let req = c.platform.as_deref().unwrap_or("?");
        anyhow::bail!(
            "Course '{}' requires {} but you are on {}.",
            c.name,
            req,
            platform_status.current
        );
    }

    // Check tool requirements
    let tool_statuses = exec::toolcheck::check_language_tools(&c.language);
    let missing: Vec<_> = tool_statuses.iter().filter(|t| !t.found).collect();
    if !missing.is_empty() {
        let mut msg = format!(
            "Course '{}' requires tools that are not installed:\n",
            c.name
        );
        for t in &missing {
            msg.push_str(&format!("  - {} (not found)", t.command));
            if let Some(ref hint) = t.install_hint {
                msg.push_str(&format!("\n    Install: {}", hint));
            }
            msg.push('\n');
        }
        anyhow::bail!(msg);
    }

    // Validate --lesson flag before entering TUI
    if let Some(lesson_id) = lesson {
        let valid_ids: Vec<&str> = c.loaded_lessons.iter().map(|l| l.id.as_str()).collect();
        if !valid_ids.contains(&lesson_id) {
            eprintln!("Unknown lesson '{}'. Available lessons:", lesson_id);
            for l in &c.loaded_lessons {
                eprintln!("  {} - {}", l.id, l.title);
            }
            anyhow::bail!("Invalid lesson ID '{}'", lesson_id);
        }
    }

    // Feature gate: AI requires --features llm at compile time
    #[cfg(not(feature = "llm"))]
    if config
        .llm
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        anyhow::bail!(
            "AI features require building with --features llm.\n\
             Rebuild with: cargo build --features llm"
        );
    }

    let progress_store = state::ProgressStore::load()?;

    // Check for major version change
    check_version_migration(&c, &progress_store)?;

    // Detect sandbox level
    let sandbox_level = exec::sandbox::SandboxLevel::detect(&config.sandbox_level);
    if sandbox_level == exec::sandbox::SandboxLevel::Basic {
        eprintln!("Note: Running with basic sandboxing (timeout + tmpdir only).");
        eprintln!("      Install firejail or bubblewrap for stronger isolation.");
    }

    let mut terminal = ui::terminal::setup()?;
    let mut app = ui::app::App::new_with_course(
        c,
        progress_store,
        config.clone(),
        sandbox_level,
        lesson,
        dir,
    );

    // Spawn LLM background thread if AI is enabled in config
    #[cfg(feature = "llm")]
    if config.llm.enabled {
        let llm_config = config.llm.clone();
        let channel = llm::ollama::spawn_llm_thread(llm_config);
        app.enable_ai(channel);
    }

    let result = app.run(&mut terminal);
    ui::terminal::restore_terminal()?;

    result?;
    Ok(())
}

fn check_version_migration(
    course: &course::Course,
    store: &state::ProgressStore,
) -> anyhow::Result<()> {
    let course_id = course.name.to_lowercase().replace(' ', "-");

    for (key, cp) in &store.data.courses {
        if key.starts_with(&format!("{}@", course_id)) {
            if let Ok(existing_ver) = semver::Version::parse(&cp.course_version) {
                if let Ok(new_ver) = semver::Version::parse(&course.version) {
                    if existing_ver.major != new_ver.major {
                        println!(
                            "Course \"{}\" updated from v{}.x to v{}.0.0.",
                            course.name, existing_ver.major, new_ver.major
                        );
                        println!(
                            "Your progress was for v{}. Exercises have changed.",
                            existing_ver.major
                        );
                        println!("Keeping existing progress (use 'learnlocal reset' or [r] on the Progress screen to start fresh).");
                    }
                }
            }
        }
    }

    Ok(())
}

fn cmd_progress(course_name: &str, courses_dir: &Option<PathBuf>) -> anyhow::Result<()> {
    let store = state::ProgressStore::load()?;

    let dir = discover_courses_dir(courses_dir);
    let course_path = dir.join(course_name);
    let loaded_course = if course_path.exists() {
        course::load_course(&course_path).ok()
    } else {
        None
    };

    let matching: Vec<_> = store
        .data
        .courses
        .iter()
        .filter(|(key, _)| key.starts_with(&format!("{}@", course_name)))
        .collect();

    if matching.is_empty() && loaded_course.is_none() {
        println!("No progress found for '{}'", course_name);
        return Ok(());
    }

    if let Some(ref c) = loaded_course {
        let course_id = c.name.to_lowercase().replace(' ', "-");
        let key = state::types::progress_key(&course_id, &c.version);
        let cp = store.data.courses.get(&key);

        println!("{} v{}:", c.name, c.version);
        if let Some(cp) = cp {
            println!("  Started: {}", cp.started_at);
            println!("  Last activity: {}", cp.last_activity);
        }

        let total_lessons = c.loaded_lessons.len();
        let completed_lessons = c
            .loaded_lessons
            .iter()
            .filter(|l| {
                cp.and_then(|cp| cp.lessons.get(&l.id))
                    .map(|lp| lp.status == state::types::ProgressStatus::Completed)
                    .unwrap_or(false)
            })
            .count();

        println!(
            "  Lessons: {}/{} complete",
            completed_lessons, total_lessons
        );
        println!();

        let lesson_refs: std::collections::HashMap<&str, &course::types::LessonRef> =
            c.lessons.iter().map(|lr| (lr.id.as_str(), lr)).collect();

        for lesson in &c.loaded_lessons {
            let total_ex = lesson.loaded_exercises.len();
            let completed_ex = cp
                .and_then(|cp| cp.lessons.get(&lesson.id))
                .map(|lp| {
                    lp.exercises
                        .values()
                        .filter(|e| e.status == state::types::ProgressStatus::Completed)
                        .count()
                })
                .unwrap_or(0);

            let is_complete = cp
                .and_then(|cp| cp.lessons.get(&lesson.id))
                .map(|lp| lp.status == state::types::ProgressStatus::Completed)
                .unwrap_or(false);

            let status_icon = if is_complete {
                "\u{2714}"
            } else if completed_ex > 0 {
                "\u{25CB}"
            } else {
                " "
            };

            let requires = lesson_refs
                .get(lesson.id.as_str())
                .map(|lr| &lr.requires)
                .cloned()
                .unwrap_or_default();

            let requires_str = if requires.is_empty() {
                String::new()
            } else {
                format!("  requires: {}", requires.join(", "))
            };

            println!(
                "  [{}] {:<20} ({}/{} exercises){}",
                status_icon, lesson.id, completed_ex, total_ex, requires_str
            );
        }
    } else {
        for (key, cp) in matching {
            println!("{} (v{}):", key, cp.course_version);
            println!("  Started: {}", cp.started_at);
            println!("  Last activity: {}", cp.last_activity);

            let total_lessons = cp.lessons.len();
            let completed_lessons = cp
                .lessons
                .values()
                .filter(|l| l.status == state::types::ProgressStatus::Completed)
                .count();

            println!(
                "  Lessons: {}/{} complete",
                completed_lessons, total_lessons
            );
            println!();

            for (lesson_id, lp) in &cp.lessons {
                let status_icon = match lp.status {
                    state::types::ProgressStatus::Completed => "\u{2714}",
                    state::types::ProgressStatus::InProgress => "\u{25CB}",
                    state::types::ProgressStatus::Skipped => "\u{2192}",
                };

                let total_ex = lp.exercises.len();
                let completed_ex = lp
                    .exercises
                    .values()
                    .filter(|e| e.status == state::types::ProgressStatus::Completed)
                    .count();

                println!(
                    "  [{}] {} ({}/{} exercises)",
                    status_icon, lesson_id, completed_ex, total_ex
                );
            }
        }
    }

    Ok(())
}

fn cmd_reset(course_name: &str) -> anyhow::Result<()> {
    let mut store = state::ProgressStore::load()?;

    let keys_to_remove: Vec<String> = store
        .data
        .courses
        .keys()
        .filter(|key| key.starts_with(&format!("{}@", course_name)))
        .cloned()
        .collect();

    if keys_to_remove.is_empty() {
        println!("No progress found for '{}'", course_name);
        return Ok(());
    }

    println!("Reset progress for {}? This cannot be undone.", course_name);
    println!("Type 'yes' to confirm:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() == "yes" {
        for key in &keys_to_remove {
            store.data.courses.remove(key);
        }
        store.save()?;
        println!("Progress reset for '{}'", course_name);
    } else {
        println!("Reset cancelled.");
    }

    Ok(())
}

fn cmd_validate(path: &Path, run_solutions: bool, verbose: bool) -> anyhow::Result<i32> {
    if verbose {
        println!("Loading course from {}...", path.display());
    }

    let c = course::load_course(path)?;
    println!("Validating {} v{}...", cli_fmt::bold(&c.name), c.version);

    let result = course::validate_course(&c);

    for check in &result.checks {
        if check.passed {
            println!("  {} {}", cli_fmt::green("\u{2713}"), check.name);
        } else {
            println!("  {} {}", cli_fmt::red("\u{2717}"), check.name);
            println!("    {}", cli_fmt::yellow(&check.message));
        }
    }

    let structural_passed = result.all_passed();

    if run_solutions && structural_passed {
        println!("\nRunning solutions against validation...");

        let mut solution_passed = 0;
        let mut solution_failed = 0;

        for lesson in &c.loaded_lessons {
            for exercise in &lesson.loaded_exercises {
                let solution_files = exercise.get_solution_files(&c.language.extension);
                if solution_files.is_empty() {
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
                    Ok((exec_result, _teardown_warnings)) => {
                        if exec_result.is_success() {
                            println!(
                                "  {} {}/{}: passes",
                                cli_fmt::green("\u{2713}"),
                                lesson.id,
                                exercise.id
                            );
                            solution_passed += 1;
                        } else {
                            let msg = match exec_result {
                                exec::runner::ExecutionResult::StepFailed {
                                    step_name,
                                    stderr,
                                    ..
                                } => format!(
                                    "{} failed: {}",
                                    step_name,
                                    stderr.lines().next().unwrap_or("")
                                ),
                                exec::runner::ExecutionResult::ValidationFailed(ref vr) => match vr
                                {
                                    exec::validate::ValidationResult::OutputMismatch {
                                        expected,
                                        actual,
                                    } => format!("expected \"{}\" got \"{}\"", expected, actual),
                                    exec::validate::ValidationResult::StateAssertionFailed {
                                        results,
                                    } => {
                                        let failed: Vec<_> = results
                                            .iter()
                                            .filter(|r| !r.passed)
                                            .map(|r| format!("{}: {}", r.description, r.detail))
                                            .collect();
                                        format!("state assertions failed: {}", failed.join("; "))
                                    }
                                    _ => "validation failed".to_string(),
                                },
                                exec::runner::ExecutionResult::Timeout { step_name } => {
                                    format!("{} timed out", step_name)
                                }
                                exec::runner::ExecutionResult::SetupFailed {
                                    step_name,
                                    stderr,
                                    ..
                                } => format!(
                                    "setup '{}' failed: {}",
                                    step_name,
                                    stderr.lines().next().unwrap_or("")
                                ),
                                exec::runner::ExecutionResult::ServiceFailed {
                                    service_name,
                                    error,
                                } => format!("service '{}' failed: {}", service_name, error),
                                _ => "unknown error".to_string(),
                            };
                            println!(
                                "  {} {}/{}: {}",
                                cli_fmt::red("\u{2717}"),
                                lesson.id,
                                exercise.id,
                                msg
                            );
                            solution_failed += 1;
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
                        solution_failed += 1;
                    }
                }
            }
        }

        let total = solution_passed + solution_failed;
        if solution_failed > 0 {
            println!(
                "\n{}",
                cli_fmt::red(&format!(
                    "Validation: {}/{} passed, {} failed.",
                    solution_passed, total, solution_failed
                ))
            );
            return Ok(exit_codes::VALIDATION_FAIL);
        } else {
            println!(
                "\n{}",
                cli_fmt::green(&format!(
                    "Validation: {}/{} passed.",
                    solution_passed, total
                ))
            );
        }
    } else if !structural_passed {
        let failed = result.checks.iter().filter(|c| !c.passed).count();
        println!(
            "\n{}",
            cli_fmt::red(&format!(
                "Structural validation failed: {} issues found.",
                failed
            ))
        );
        return Ok(exit_codes::VALIDATION_FAIL);
    } else {
        println!("\n{}", cli_fmt::green("All structural checks passed."));
    }

    Ok(exit_codes::SUCCESS)
}

fn cmd_completions(shell: clap_complete::Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "learnlocal", &mut std::io::stdout());
    Ok(())
}

fn cmd_man() -> anyhow::Result<()> {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    man.render(&mut std::io::stdout())?;
    Ok(())
}

fn cmd_doctor(courses_dir: &Option<PathBuf>) -> anyhow::Result<i32> {
    println!("{}", cli_fmt::bold("LearnLocal Doctor"));
    println!();

    let mut has_critical_failure = false;

    // Platform
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    println!("  {} Platform: {} {}", cli_fmt::green("\u{2713}"), os, arch);

    // Terminal size
    match crossterm::terminal::size() {
        Ok((cols, rows)) => {
            if cols < 80 || rows < 24 {
                println!(
                    "  {} Terminal: {}x{} (recommend at least 80x24)",
                    cli_fmt::yellow("!"),
                    cols,
                    rows
                );
            } else {
                println!(
                    "  {} Terminal: {}x{}",
                    cli_fmt::green("\u{2713}"),
                    cols,
                    rows
                );
            }
        }
        Err(_) => {
            println!("  {} Terminal: unable to detect size", cli_fmt::yellow("!"));
        }
    }

    // $EDITOR
    match std::env::var("EDITOR") {
        Ok(editor) => {
            let cmd = editor.split_whitespace().next().unwrap_or(&editor);
            if exec::toolcheck::command_exists(cmd) {
                println!("  {} Editor: {}", cli_fmt::green("\u{2713}"), editor);
            } else {
                println!(
                    "  {} Editor: {} (set in $EDITOR but not found in PATH)",
                    cli_fmt::yellow("!"),
                    editor
                );
            }
        }
        Err(_) => {
            println!("  {} Editor: $EDITOR not set", cli_fmt::yellow("!"));
        }
    }

    // Sandbox tools
    let has_firejail = exec::toolcheck::command_exists("firejail");
    let has_bwrap = exec::toolcheck::command_exists("bwrap");
    if has_firejail {
        println!("  {} Sandbox: firejail", cli_fmt::green("\u{2713}"));
    } else if has_bwrap {
        println!("  {} Sandbox: bubblewrap", cli_fmt::green("\u{2713}"));
    } else {
        println!(
            "  {} Sandbox: basic only (install firejail or bubblewrap for stronger isolation)",
            cli_fmt::yellow("!")
        );
    }

    // Course toolchains
    let dir = discover_courses_dir(courses_dir);
    if dir.exists() {
        let mut all_commands = std::collections::BTreeSet::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(info) = course::load_course_info(&entry.path()) {
                        for cmd in &info.step_commands {
                            all_commands.insert(cmd.clone());
                        }
                    }
                }
            }
        }

        for cmd in &all_commands {
            if exec::toolcheck::command_exists(cmd) {
                println!("  {} Tool: {}", cli_fmt::green("\u{2713}"), cmd);
            } else {
                let hint = exec::toolcheck::suggest_install(cmd)
                    .unwrap_or_else(|| "check your package manager".to_string());
                println!(
                    "  {} Tool: {} \u{2014} not found ({})",
                    cli_fmt::red("\u{2717}"),
                    cmd,
                    hint
                );
                has_critical_failure = true;
            }
        }
    }

    // Ollama
    if exec::toolcheck::command_exists("ollama") {
        println!("  {} Ollama: installed", cli_fmt::green("\u{2713}"));
    } else {
        println!(
            "  {} Ollama: not installed (optional, for AI features)",
            cli_fmt::yellow("!")
        );
    }

    // Config path
    let config_path = dirs::config_dir()
        .map(|d| d.join("learnlocal").join("config.yaml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/learnlocal/config.yaml"));
    if config_path.exists() {
        println!(
            "  {} Config: {}",
            cli_fmt::green("\u{2713}"),
            config_path.display()
        );
    } else {
        println!(
            "  {} Config: {} (not yet created)",
            cli_fmt::dim("-"),
            config_path.display()
        );
    }

    // Progress path
    let progress_path = dirs::data_dir()
        .map(|d| d.join("learnlocal").join("progress.json"))
        .unwrap_or_else(|| PathBuf::from("~/.local/share/learnlocal/progress.json"));
    if progress_path.exists() {
        println!(
            "  {} Progress: {}",
            cli_fmt::green("\u{2713}"),
            progress_path.display()
        );
    } else {
        println!(
            "  {} Progress: {} (no progress yet)",
            cli_fmt::dim("-"),
            progress_path.display()
        );
    }

    if has_critical_failure {
        Ok(exit_codes::MISSING_TOOL)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}

fn cmd_init(name: &str) -> anyhow::Result<()> {
    let base = PathBuf::from(name);
    if base.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    let exercises_dir = base
        .join("lessons")
        .join("01-getting-started")
        .join("exercises");
    std::fs::create_dir_all(&exercises_dir)?;

    // course.yaml
    std::fs::write(
        base.join("course.yaml"),
        format!(
            r#"name: "{name}"
version: "1.0.0"
description: "A new LearnLocal course"
author: "Your Name"

language:
  id: python3
  display_name: Python
  extension: .py
  steps:
    - name: run
      command: "python3 {{dir}}/{{main}}"
      check_exit_code: true
      capture_output: true

lessons:
  - id: getting-started
    title: Getting Started
"#,
            name = name
        ),
    )?;

    // lesson.yaml
    std::fs::write(
        base.join("lessons")
            .join("01-getting-started")
            .join("lesson.yaml"),
        r#"id: getting-started
title: "Getting Started"
description: "Your first steps"
content: content.md
exercises:
  - hello
"#,
    )?;

    // content.md
    std::fs::write(
        base.join("lessons")
            .join("01-getting-started")
            .join("content.md"),
        r#"# Getting Started

Welcome to this course! Let's start with a classic Hello World.

## Hello World

Your first exercise: print "Hello, World!" to the screen.
"#,
    )?;

    // exercise YAML (flat file, not directory)
    std::fs::write(
        exercises_dir.join("01-hello.yaml"),
        r#"id: hello
title: "Hello World"
type: write

prompt: |
  Print 'Hello, World!' to stdout.

starter: |
  # Write your code here

validation:
  method: output
  expected_output: "Hello, World!"

solution: |
  print("Hello, World!")

hints:
  - "Use the print() function"
"#,
    )?;

    println!(
        "{} Created course scaffold in {}/",
        cli_fmt::green("\u{2713}"),
        name
    );
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/course.yaml to set your course details", name);
    println!("  2. Add lessons and exercises");
    println!("  3. Validate with: learnlocal validate {}/", name);

    Ok(())
}

fn cmd_export(course_filter: Option<&str>, format: &str) -> anyhow::Result<()> {
    let store = state::ProgressStore::load()?;

    match format {
        "json" => {
            let data = if let Some(filter) = course_filter {
                let filtered: std::collections::HashMap<_, _> = store
                    .data
                    .courses
                    .iter()
                    .filter(|(key, _)| key.starts_with(&format!("{}@", filter)))
                    .collect();
                serde_json::to_string_pretty(&filtered)?
            } else {
                serde_json::to_string_pretty(&store.data)?
            };
            println!("{}", data);
        }
        "csv" => {
            println!("course,lesson,exercise,status,attempts,last_activity");
            for (course_key, cp) in &store.data.courses {
                if let Some(filter) = course_filter {
                    if !course_key.starts_with(&format!("{}@", filter)) {
                        continue;
                    }
                }
                for (lesson_id, lp) in &cp.lessons {
                    for (exercise_id, ep) in &lp.exercises {
                        let status = match ep.status {
                            state::types::ProgressStatus::Completed => "completed",
                            state::types::ProgressStatus::InProgress => "in_progress",
                            state::types::ProgressStatus::Skipped => "skipped",
                        };
                        let last = ep
                            .attempts
                            .last()
                            .map(|a| a.timestamp.as_str())
                            .unwrap_or(&cp.last_activity);
                        println!(
                            "{},{},{},{},{},{}",
                            course_key,
                            lesson_id,
                            exercise_id,
                            status,
                            ep.attempts.len(),
                            last
                        );
                    }
                }
            }
        }
        _ => {
            anyhow::bail!("Unknown format '{}'. Supported: json, csv", format);
        }
    }

    Ok(())
}
