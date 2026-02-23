# LearnLocal — Architecture & Implementation Reference

## Build Status

| Check | Result |
|---|---|
| `cargo build` | 0 warnings |
| `cargo build --features llm` | 0 warnings |
| `cargo test` | 207 pass |
| `cargo test --features llm` | 223 pass |
| `learnlocal validate courses/cpp-fundamentals` | 55/55 exercises pass |
| `learnlocal validate courses/python-fundamentals` | 54/54 exercises pass |
| `learnlocal validate courses/js-fundamentals` | 56/56 exercises pass |
| `learnlocal validate courses/ai-fundamentals-python` | 56/56 exercises pass |
| `learnlocal validate courses/linux-fundamentals` | 55/55 exercises pass |
| `learnlocal validate courses/env-engine-test` | 6/6 exercises pass |
| `learnlocal` (no args) | Home screen TUI launches |
| `learnlocal start cpp-fundamentals` | Course screen TUI launches |
| `learnlocal list` | Shows all courses (lightweight loading) |
| `learnlocal progress` / `reset` | Working |

---

## Project Structure

```
src/
├── main.rs              # Entry point, CLI dispatch (cmd_home, cmd_start, cmd_list, etc.)
├── cli.rs               # clap derive structs (Cli, Option<Command>)
├── config.rs            # Config loading/saving, SandboxLevelPref, LlmConfig
├── error.rs             # LearnLocalError enum (thiserror)
├── course/
│   ├── mod.rs           # Re-exports: load_course, load_course_info, validate_course, types
│   ├── types.rs         # Course, CourseInfo, Language, Lesson, Exercise, etc.
│   ├── loader.rs        # load_course(), load_course_info(), split_content/display_sections()
│   └── validator.rs     # validate_course() — structural checks, cycle detection
├── exec/
│   ├── mod.rs           # Re-exports
│   ├── sandbox.rs       # SandboxLevel detection, StepOutput, run_command/run_command_with_loopback
│   ├── runner.rs        # run_lifecycle() shared pipeline, RunOutput, ExecutionResult
│   ├── environment.rs   # Environment setup, service readiness, state validation, port allocation
│   ├── placeholder.rs   # substitute() — {dir}, {main}, {output}, {files}
│   ├── validate.rs      # validate_output() — output/regex/compile-only
│   └── toolcheck.rs     # extract_step_commands, extract_env_commands, command_exists, suggest_install
├── state/
│   ├── mod.rs           # Re-exports: ProgressStore, types
│   ├── types.rs         # Progress, CourseProgress, LessonProgress, AttemptRecord
│   ├── progress.rs      # ProgressStore — load/save JSON, atomic writes
│   ├── signals.rs       # SessionState, FullAttempt (in-memory, not persisted)
│   └── sandbox.rs       # sandbox_dir(), save/load/has_sandbox_files() — persistent sandbox code
├── ui/
│   ├── mod.rs           # Module declarations
│   ├── app.rs           # Outer App shell — screen router (Home/HowTo/Settings/Progress/Stats/Course)
│   ├── course_app.rs    # Inner CourseApp — exercise flow state machine (11 states incl Sandbox/Watching)
│   ├── screens.rs       # Screen enum, CourseAction, HomeState, HowToState, SettingsState, StatsState
│   ├── terminal.rs      # setup/restore/leave/enter alternate screen
│   ├── theme.rs         # Theme struct — colors, NO_COLOR support
│   ├── markdown.rs      # render_markdown() — pulldown-cmark → ratatui Lines
│   ├── editor.rs        # detect_editor(), edit_file(), minimal fallback
│   ├── editor_detect.rs # EditorType (Auto/Terminal/Gui), resolve_editor_type, --wait flag detection
│   ├── inline_editor.rs # InlineEditorState — multi-line TUI text editor
│   ├── diagnostics.rs   # Compiler output parser (g++/clang structured, Raw fallback)
│   ├── diff.rs          # render_output_diff() — colored expected/actual diff
│   ├── watch.rs         # WatchState — file watching (notify PollWatcher), auto-test, editor process
│   └── celebration.rs   # CourseStats, AggregateStats, exercise/lesson/course art, mini_progress_bar
└── llm/                 # Optional: behind --features llm
    ├── mod.rs           # Module declarations
    ├── backend.rs       # LlmBackend trait
    ├── channel.rs       # LlmChannel, LlmRequest, LlmEvent (mpsc types)
    ├── chat.rs          # ChatState, ChatMessage, ChatRole
    ├── config.rs        # LlmConfig, OllamaConfig, LlmSettings
    ├── context.rs       # LlmContext — read-only assembly from state
    └── ollama.rs        # OllamaBackend, spawn_llm_thread(), list_available_models()

courses/
├── cpp-fundamentals/    # C++ Fundamentals v2.0.0 (8 lessons, 55 exercises)
├── python-fundamentals/ # Python Fundamentals v1.0.0 (8 lessons, 54 exercises)
├── js-fundamentals/     # JS (Node.js) Fundamentals v1.0.0 (8 lessons, 56 exercises)
├── ai-fundamentals-python/ # AI Fundamentals v1.0.0 (8 lessons, 56 exercises)
├── linux-fundamentals/  # Linux Fundamentals v1.0.0 (8 lessons, 55 exercises, platform: linux)
└── env-engine-test/     # Env Engine Test v1.0.0 (2 lessons, 6 exercises, platform: linux)

tests/
└── fixtures/
    └── minimal-course/     (1 lesson, 1 exercise — for unit tests)
```

---

## TUI Architecture

### Two-Level State Machine

The TUI uses a two-level routing model:

**Outer App** (`src/ui/app.rs`) — routes between screens:
- `Screen::Home` — Two-panel course browser with progress bars and lesson list
- `Screen::HowTo` — Scrollable reference page (getting started, editing, watch mode, run/test, etc.)
- `Screen::Settings` — Editable config (editor, sandbox, AI model with picker)
- `Screen::Progress` — Per-course lesson/exercise breakdown
- `Screen::Stats` — Aggregate learning statistics (overall + per-course)
- `Screen::Course` — Delegates to CourseApp (includes Sandbox and Watching states)

**Inner CourseApp** (`src/ui/course_app.rs`) — exercise flow within Course screen:
- `AppState::LessonContent → ExercisePrompt → InlineEditing/Editing → Executing → RunResult/ResultSuccess/ResultFail → LessonRecap → Sandbox → Watching → CourseComplete`

### Navigation Flow

```
learnlocal (no args)  →  Home screen
learnlocal start X    →  Course screen directly (App::new_with_course)

Home  ──Enter──>  Course (loads full course, creates CourseApp)
Home  ──s──────>  Settings
Home  ──p──────>  Progress (for selected course)
Home  ──h──────>  HowTo (scrollable reference)
Home  ──t──────>  Stats (aggregate learning statistics)
Home  ──→/l────>  LessonList panel (right panel focus)
Settings  ──Esc──>  Home (saves config to disk)
Progress  ──Esc──>  Home
Progress  ──Enter──>  Course (at selected lesson)
Progress  ──s──────>  Sandbox (for selected lesson)
HowTo  ──Esc──>  Home
Stats  ──Esc──>  Home
Course  ──Esc──>  Home (CourseAction::GoHome)
Course  ──q──────>  Quit (CourseAction::Quit)
```

### Key Design Decisions

**CourseInfo vs Course:** Home screen only needs `CourseInfo` (reads `course.yaml` — name, version, lesson count). Full `Course` (loads all lessons + exercises + markdown) is only created when entering Course or Progress screens.

**Borrow checker pattern:** CourseApp methods take `&mut ProgressStore`, `&Config`, and `SandboxLevel` as parameters (not owned). This lets the outer App maintain ownership of shared state while CourseApp operates on it.

**CourseAction return type:** `CourseApp.handle_input()` returns `CourseAction` (Continue/Quit/GoHome) instead of setting `should_quit` directly. The outer App interprets this to decide whether to quit the process or return to Home.

**LLM channel ownership:** Each course session spawns a new LLM thread via `spawn_llm_thread()`. The channel is transferred to CourseApp via `enable_ai()`. When returning to Home, the CourseApp is dropped and the channel with it.

**Progressive reveal:** Lesson content is split by H2 headers into sections using `split_display_sections()`. The first section (intro) is shown immediately; pressing Space reveals the next. State is lazily recomputed when the lesson index changes, avoiding reset logic in multiple navigation paths.

**Run vs Submit:** `[Enter]` runs code without grading (shows output in RunResult screen). `[t]` submits for grading (validates against expected output). This lets learners iterate on code before committing to a graded attempt.

**AI chat during lesson reading:** The `[a]` key opens AI chat from the LessonContent state, allowing learners to ask questions about lesson material as they read it. The LLM context already includes full lesson content.

**Home two-panel layout:** Left panel shows course list with progress bars, right panel shows lesson list for selected course. `HomePanelFocus` enum tracks which panel has focus. Course tools and platform requirements are lazily checked on selection.

**Environment Engine v3:** Exercises can define an `environment:` block with filesystem setup, setup/teardown commands, background services, env vars, and dynamic ports. The runner uses a shared `run_lifecycle()` pipeline that handles the 7-phase execution flow. Services get loopback networking; student code gets loopback only when services are defined. Teardown failures are surfaced as warnings, not errors.

**Watch mode:** `[w]` enters file-watching mode using notify's PollWatcher (500ms interval, 300ms debounce). Auto-recompiles on save. `[t]` toggles auto-test within watch mode. GUI editors auto-enter watch mode via `[E]`.

**Lesson Sandbox:** `[s]` from LessonRecap or Progress opens a freeform coding sandbox for the lesson's language. Code persists to `~/.local/share/learnlocal/sandboxes/`. No grading — runs code and shows output. LLM gets a sandbox-specific system prompt encouraging exploration.

---

## Module Reference

### `src/main.rs` — Entry Point & CLI Dispatch

| Function | Purpose |
|---|---|
| `main()` | Parse CLI, call `run()` |
| `run(cli)` | Match `Option<Command>` → dispatch |
| `discover_courses_dir(custom)` | Find courses/ relative to exe or cwd |
| `cmd_home(courses_dir, config)` | Scan for CourseInfo, launch Home screen TUI |
| `cmd_list(courses_dir)` | Scan dir, load course info, print table |
| `cmd_start(courses_dir, name, lesson, config, ai)` | Load full course + progress, launch Course TUI |
| `check_version_migration(course, store)` | Warn if major version changed |
| `cmd_progress(course_name)` | Load progress, print lesson/exercise stats |
| `cmd_reset(course_name)` | Confirm + delete progress keys |
| `cmd_validate(path, run_solutions)` | Structural validation + solution execution |

### `src/cli.rs` — CLI Parsing (clap)

| Type | Purpose |
|---|---|
| `struct Cli` | Top-level: `--courses` flag + `Option<Command>` |
| `enum Command` | `List`, `Start`, `Progress`, `Reset`, `Validate` |

`command: Option<Command>` — None means launch Home screen.

### `src/config.rs` — Configuration

| Type | Purpose |
|---|---|
| `struct Config` | `editor`, `sandbox_level`, `llm` (behind feature) |
| `enum SandboxLevelPref` | `Auto`, `Basic`, `Contained` |
| `struct LlmConfigSection` | `ollama: OllamaSection` |
| `struct OllamaSection` | `url`, `model`, `fallback_models` |

Config path: `~/.config/learnlocal/config.yaml`. Loaded with defaults, saved on settings screen exit.

### `src/course/types.rs` — Core Data Structures

| Type | Purpose |
|---|---|
| `struct CourseInfo` | Lightweight: `dir_name`, `name`, `version`, `description`, `author`, `lesson_count`, `source_dir`, `step_commands`, `env_commands`, `total_exercise_count`, `platform` |
| `struct Course` | Full: `name`, `version`, `language`, `lessons`, `loaded_lessons`, `source_dir` |
| `struct Language` | `id`, `display_name`, `extension`, `steps`, `limits` |
| `struct Lesson` | `id`, `title`, `exercises`, `teaches`, `recap`, `loaded_exercises`, `content_markdown`, `content_sections` |
| `struct Exercise` | `id`, `title`, `exercise_type`, `prompt`, `starter`/`files`, `validation`, `hints`, `solution` |
| `enum ExerciseType` | `Write`, `Fix`, `FillBlank`, `MultipleChoice`, `Predict` |

### `src/course/loader.rs` — Course Loading

| Function | Purpose |
|---|---|
| `load_course_info(path)` | Read only course.yaml → `CourseInfo` (no lessons loaded) |
| `load_course(path)` | Full load: course.yaml + all lessons + exercises + markdown |
| `load_lesson(lessons_dir, id, ext)` | Parse lesson.yaml, load content.md + exercises |
| `load_exercise(exercises_dir, id, ext)` | Parse exercise YAML |
| `split_content_sections(md)` | Split by H2, discard intro — used for per-exercise context above prompt |
| `split_display_sections(md)` | Split by H2, keep intro as section 0 — used for progressive reveal |

### `src/ui/screens.rs` — Screen Types

| Type | Purpose |
|---|---|
| `enum Screen` | `Home`, `HowTo`, `Settings`, `Progress`, `Stats`, `Course` |
| `enum CourseAction` | `Continue`, `Quit`, `GoHome` — returned by CourseApp |
| `struct HomeState` | `selected_idx`, `summaries`, `focus` (HomePanelFocus), `right_selected_idx`, `tool_check_cache`, `platform_check_cache` |
| `struct HowToState` | `scroll_offset`, `content_height` |
| `struct SettingsState` | `focused_idx`, `fields`, `editing`, `edit_buffer`, AI fields |
| `struct StatsState` | `scroll_offset`, `content_height` |
| `struct ProgressViewState` | `course_idx`, `course: Option<Course>`, `selected_lesson_idx` |
| `enum HomePanelFocus` | `CourseList`, `LessonList` |
| `enum SettingsField` | `Editor`, `SandboxLevel`, `AiEnabled`, `OllamaUrl`, `OllamaModel` |
| `enum CourseStatus` | `NotStarted`, `InProgress`, `Completed` |
| `struct CourseProgressSummary` | `info`, `status`, `completed_lessons/exercises`, `total_lessons/exercises` |

### `src/ui/app.rs` — Outer App Shell

| Method | Purpose |
|---|---|
| `App::new(courses, progress, config, sandbox, dir)` | Create for Home screen |
| `App::new_with_course(course, ...)` | Create for direct Course entry (`learnlocal start`) |
| `enable_ai(channel, config)` | Enable LLM (forwarded to CourseApp when active) |
| `run(terminal)` | Main loop: render → handle input |
| `render_home/howto/settings/progress/stats()` | Screen-specific rendering |
| `handle_home/howto/settings/progress/stats/course_input()` | Screen-specific input handling |
| `start_selected_course(lesson_idx)` | Load full course, create CourseApp, switch to Course |
| `open_progress_for_selected()` | Load full course, switch to Progress |
| `save_settings()` | Apply SettingsState back to Config, save to disk |
| `fetch_and_open_model_picker()` | Short tokio runtime to fetch Ollama /api/tags |

### `src/ui/course_app.rs` — Exercise Flow

| Type/Method | Purpose |
|---|---|
| `enum AppState` | `LessonContent`, `ExercisePrompt`, `InlineEditing`, `Editing`, `Executing`, `RunResult`, `ResultSuccess`, `ResultFail`, `LessonRecap`, `Sandbox`, `Watching`, `CourseComplete` |
| `struct CourseApp` | Course, indices, session, scroll, animation, progressive reveal, inline editor, LLM chat, sandbox, watch, celebrations, teardown_warnings |
| `CourseApp::new(course, progress, lesson, lesson_idx)` | Init with resume detection |
| `handle_input(key, progress, config, sandbox) → CourseAction` | Dispatch by state, return action |
| `render(frame, theme)` | Delegate to state-specific render methods |
| `render_lesson_content()` | Progressive reveal: lazy recompute, visible sections, animated indicator |
| `render_scrollable()` | Scrollable paragraph with content_line_count tracking |
| `launch_editor(config)` | Leave TUI → write tmpfiles → $EDITOR → read back → re-enter |
| `enter_inline_editor()` | Open inline TUI editor for first editable file |
| `run_exercise(sandbox)` | Run code without grading → RunResult screen |
| `submit_exercise(progress, sandbox)` | Execute + grade → record attempt → transition |
| `enable_ai(channel, config)` | Enable LLM chat within course session |
| `drain_llm_events()` | Non-blocking poll of LLM response channel |
| `enter_watch_mode()` | Start PollWatcher on exercise files, enter Watching state |
| `exit_watch_mode()` | Stop watcher, return to ExercisePrompt or Sandbox |
| `tick_watch_mode()` | Poll watcher for file changes, debounce, auto-run/test |
| `enter_sandbox()` | Open lesson sandbox (load or create playground file) |
| `reset_to_starter()` | Restore exercise starter code (compare by filename) |
| Progress methods | `record_attempt`, `mark_exercise_completed/skipped`, `mark_lesson_completed` |

### `src/ui/inline_editor.rs` — Inline TUI Editor

| Type/Method | Purpose |
|---|---|
| `struct InlineEditorState` | Lines, cursor position, file index, scroll offset |
| `enum EditorAction` | `Continue`, `Save`, `SaveAndClose` |
| `new(content, file_idx)` | Parse content into lines, position cursor |
| `handle_key(code, modifiers)` | Arrow nav, insert, delete, backspace, Home/End, Tab, Esc, Ctrl+S |
| `render(frame, area)` | Line numbers, block cursor, gutter lines |
| `content()` | Reassemble lines into single string |

### `src/ui/diagnostics.rs` — Compiler Error Formatting

| Type/Method | Purpose |
|---|---|
| `enum DiagnosticEntry` | `Structured` (file, line, col, severity, message, notes) or `Raw` (unparsed) |
| `parse_compiler_output(stderr)` | Parse g++/clang structured format, fall back to Raw |
| `render_diagnostics(entries, theme)` | Severity-colored rendering with clean relative paths |

### `src/exec/` — Execution Engine

| File | Key Items |
|---|---|
| `sandbox.rs` | `SandboxLevel` (Basic/Contained), `detect()`, `StepOutput`, `run_command()`, `run_command_with_loopback()` with timeout, `spawn_service()` |
| `runner.rs` | `run_lifecycle()` shared 7-phase pipeline, `LifecycleError`/`LifecycleOutput` internal types, `RunOutput` (with `teardown_warnings`), `ExecutionResult` enum, `run_exercise_with_sandbox()`, `execute_exercise_with_sandbox()` returns `(ExecutionResult, Vec<String>)`, `ServiceGuard` RAII, `drain_service_pipes()` with optional capture |
| `environment.rs` | `setup_environment()` (dirs/files/symlinks/env vars/cwd/ports), `allocate_ports()`, `run_env_command()`/`run_env_command_full()`, `spawn_stream_reader()`, `wait_for_service_ready()` (pattern mode with configurable streams, delay mode), `validate_state()` with 11 assertion types |
| `placeholder.rs` | `substitute()` — `{dir}`, `{main}`, `{output}`, `{files}` |
| `validate.rs` | `ValidationResult` enum, `validate_output()` — output/regex/compile-only |
| `toolcheck.rs` | `extract_step_commands()`, `extract_env_commands()`, `command_exists()`, `suggest_install()`, `check_platform()`, `PlatformStatus` |

### Environment Engine (v3)

The environment engine enables exercises that need filesystem setup, background services, dynamic ports, and state-based validation. It's fully backward compatible — exercises without an `environment:` block work exactly as before.

**EnvironmentSpec fields:**

| Field | Type | Description |
|---|---|---|
| `files` | `Vec<EnvFile>` | Files to create in sandbox (path, content, permissions) |
| `dirs` | `Vec<String>` | Directories to create |
| `symlinks` | `Vec<EnvSymlink>` | Symbolic links (source -> target) |
| `env` | `HashMap<String, String>` | Environment variables (supports `{dir}` substitution) |
| `cwd` | `Option<String>` | Working directory override (relative to sandbox) |
| `ports` | `usize` | Number of dynamic ports to allocate (injected as `LEARNLOCAL_PORT_0..N`) |
| `setup` | `Vec<EnvCommand>` | Commands run after filesystem setup, before student code |
| `services` | `Vec<EnvService>` | Background services started before student code |
| `teardown` | `Vec<EnvCommand>` | Commands run after student code, before services killed |
| `assertions` | `Vec<StateAssertion>` | State validation (used with `validation.method: state`) |

**EnvService fields:**

| Field | Type | Default | Description |
|---|---|---|---|
| `name` | `String` | required | Service identifier |
| `command` | `String` | required | Command to run |
| `args` | `Vec<String>` | `[]` | Arguments (supports `{dir}` substitution) |
| `ready_pattern` | `Option<String>` | `None` | Regex to match on output for readiness |
| `ready_stream` | `Option<String>` | `"both"` | Which stream to watch: `"stdout"`, `"stderr"`, or `"both"` |
| `ready_timeout_seconds` | `u64` | `10` | Timeout for readiness check |
| `ready_delay_ms` | `u64` | `200` | Delay when no ready_pattern (process alive = ready) |
| `capture_stdout` | `Option<String>` | `None` | Sandbox-relative path to capture stdout |
| `capture_stderr` | `Option<String>` | `None` | Sandbox-relative path to capture stderr |

**StateAssertion variants (11):**
`FileExists`, `DirExists`, `FileNotExists`, `DirNotExists`, `FileContains`, `FileMatches`, `FileEquals`, `Permissions`, `Symlink`, `FileCount`, `DirEmpty`

**Execution lifecycle (7 phases):**
1. Sandbox creation (tmpdir + optional firejail/bwrap)
2. Filesystem setup (dirs, files with `{dir}` substitution, symlinks, env vars, ports)
3. Setup commands (run with `{dir}` placeholder only — student files don't exist yet)
4. Start services (with `{dir}` substitution in args, loopback networking, readiness wait)
5. Write student files + run language steps (loopback networking when services defined)
6. Teardown commands (full `{dir}/{main}/{output}/{files}` substitution, failures -> warnings)
7. Kill services (ServiceGuard RAII drop)

**Networking:** Services always get loopback networking (firejail `--net=lo`, bwrap omits `--unshare-net`). Student code gets loopback only when the exercise defines services — this is computed automatically.

**Teardown guarantees:** Teardown runs unconditionally — even when setup, services, or student code fail. The pipeline captures phase errors without early returns, runs teardown, then attaches teardown warnings to the error. All `LifecycleError` variants carry `teardown_warnings: Vec<String>`. This ensures diagnostic data (DB dumps, log captures) is always available.

**Path safety:** All environment paths (files, dirs, symlinks, assertion paths) are validated at both build time (validator) and runtime:
- Rejects absolute paths and `..` traversal components
- `check_env_path()` in validator catches authoring mistakes
- `validate_path_safety()` in `check_assertion()` guards runtime execution
- Assertion regex patterns validated at build time (`FileMatches`)

**Permission handling:** Invalid octal permission strings (e.g., `"abc"` instead of `"755"`) return a clear error instead of being silently ignored.

**Output truncation:** `truncate_output()` in sandbox.rs finds the last valid UTF-8 char boundary before the byte limit, preventing panics from slicing mid-character on multi-byte UTF-8 output.

### Workspace View

When an exercise defines an `environment:` block with visible content, the exercise view shows a **WORKSPACE** panel between the prompt text and the code box. This gives students context about what files, directories, and services exist before they write code.

**What's shown:**
- Files (`📄`): filename + first 4 lines of content (truncated with "... (N more lines)" if longer). Permissions shown if set.
- Directories (`📁`): path with trailing `/`
- Symlinks (`🔗`): link → target
- Services (`🔌`): name + command
- Dynamic ports (`🌐`): count of allocated ports
- Setup commands (`⚙️`): count of setup steps (summarized)

**Visibility rules:**
- Only rendered when `exercise.environment` is `Some` AND has visible content (files, dirs, symlinks, services, ports, or setup commands)
- Empty `EnvironmentSpec` (all defaults) → nothing shown
- Pure compute exercises (no environment block) → nothing shown

**Implementation:** `render_workspace_lines()` static method on `CourseApp`. Returns `Vec<Line>` inserted into the scrollable content. Border in `theme.code_border`, labels in `theme.keyword`, content in `theme.body_text`, muted details in `theme.muted`.

### Assertion Checklist

Exercises using `validation.method: state` show a visual checklist of assertions — before running (unchecked) and after running (pass/fail).

**Three rendering modes:**

| Mode | When | Icon | Function |
|---|---|---|---|
| Pre-run | ExercisePrompt, before student runs | `○` (unchecked) | `render_assertion_checklist_lines()` |
| Post-run success | ResultSuccess, after all assertions pass | `✔` (green) | `render_assertion_results_lines()` |
| Post-run failure | ResultFail, with pass/fail per item | `✔`/`✘` (green/red) | `render_assertion_results_lines()` |

**Human-readable descriptions:** `assertion_description()` maps all 11 `StateAssertion` variants to natural language:
- `FileExists("output.txt")` → `"output.txt exists"`
- `FileContains({path: "log.txt", content: "Total:"})` → `"log.txt contains \"Total:\""`
- `Permissions({path: "script.sh", mode: "755"})` → `"script.sh has permissions 755"`
- `FileCount({path: "output", count: 3})` → `"output/ has 3 entries"`
- Content previews truncated at 30 chars with `...`

**Spoiler prevention:** Pre-run checklist is hidden for `ExerciseType::Predict` (the expected state IS the answer).

**State tracking:** `last_assertion_results: Option<Vec<AssertionResult>>` on `CourseApp`. Set after graded submission, cleared on exercise change via `reset_session_for_current_exercise()`. Failed assertions show detail text (e.g., `→ file does not contain "Total:"`) in muted color.

**Post-run ResultFail view:** Assertions also appear in the existing `FailureDetail::StateAssertionFailed` rendering (which was already implemented). The new `last_assertion_results` field enables showing results in both ResultSuccess and ExercisePrompt views.

### `src/state/` — Progress & Session

| File | Key Items |
|---|---|
| `types.rs` | `Progress`, `CourseProgress`, `LessonProgress`, `ExerciseProgress`, `AttemptRecord`, `progress_key()` |
| `progress.rs` | `ProgressStore` — load/save JSON at `~/.local/share/learnlocal/progress.json`, atomic writes |
| `signals.rs` | `SessionState` — in-memory per-exercise state (current code, attempt history, hints revealed) |
| `sandbox.rs` | `sandbox_dir()`, `save_sandbox_files()`, `load_sandbox_files()`, `has_sandbox_files()` — persistent sandbox code at `~/.local/share/learnlocal/sandboxes/` |

### `src/llm/` — LLM Integration (behind `--features llm`)

| File | Key Items |
|---|---|
| `backend.rs` | `LlmBackend` trait |
| `channel.rs` | `LlmChannel` (mpsc sender/receiver pair), `LlmRequest`, `LlmEvent` |
| `chat.rs` | `ChatState`, `ChatMessage`, `ChatRole` — manages conversation history |
| `config.rs` | `LlmConfig`, `OllamaConfig`, `LlmSettings` |
| `context.rs` | `LlmContext::assemble()` / `assemble_sandbox()` — read-only view for LLM (course, lesson, exercise, session, progress). `sandbox_mode` flag, `to_sandbox_system_prompt()` |
| `ollama.rs` | `spawn_llm_thread()`, `list_available_models()`, HTTP streaming via reqwest |

**Async/sync boundary:** `spawn_llm_thread()` creates a `std::thread` with a tokio `current_thread` runtime inside. Communication uses `std::sync::mpsc` channels. The main TUI thread remains fully synchronous.

---

## Key Patterns

| Pattern | Detail |
|---|---|
| Feature gating | `#[cfg(feature = "llm")]` on struct fields, methods, imports. Core binary has zero async/HTTP deps. |
| Borrow checker | CourseApp methods take shared resources as params, not owned. `#[allow(unused_mut)]` for vars that are mut only with `cfg(feature = "llm")`. |
| CourseAction | CourseApp returns action enum instead of mutating outer state. Keeps ownership clean. |
| Config serde | `alias = "kebab-case"` for YAML fields with hyphens (e.g., `sandbox-level`). |
| Progress keying | `{course_id}@{major_version}` — patch/minor updates preserve progress. |
| reqwest | Uses `rustls-tls` (not native-tls) to avoid system OpenSSL dependency. |
| Settings model picker | Blocks with short tokio `current_thread` runtime to fetch `/api/tags`. Falls back to text edit if unreachable. |
| Progressive reveal | `split_display_sections()` keeps intro + H2 sections. Lazy recompute via `reveal_lesson_idx` mismatch check. Animated `▼ Space to continue` indicator with cycling dots (500ms). |
| Run vs Submit | `[Enter]` = run without grading (RunResult screen). `[t]` = submit for grading (validates output). Separates experimentation from assessment. |
| Inline editor | `[e]` opens TUI editor, `[E]` opens external $EDITOR. InlineEditorState tracks lines/cursor, returns EditorAction. |
| Diagnostics | Compiler stderr parsed into structured entries (file:line:col:severity:message) or Raw fallback. Clean relative paths, severity coloring. |
| Scroll | `render_scrollable()` tracks `content_line_count`, clamps offset. `▲▼` indicators at edges. PgUp/PgDn/Home/End handled centrally. Page position (1/N) in status bar. |
| AI chat | Markdown rendered for assistant messages. Multi-line input (Enter=newline, Ctrl+Enter/Tab=send). Braille spinner while waiting, animated dots during streaming. Available during lesson reading via `[a]`. |
| Environment engine | `run_lifecycle()` is the shared 7-phase pipeline. LifecycleError/LifecycleOutput map to RunOutput or (ExecutionResult, Vec<String>). Teardown runs unconditionally — errors captured, teardown executed, warnings attached to error. |
| Workspace view | `render_workspace_lines()` shows environment setup (files with content preview, dirs, symlinks, services, ports, setup steps) in a bordered panel between prompt and code box. Only for exercises with visible `environment:` content. |
| Assertion checklist | Pre-run `○` checklist, post-run `✔`/`✘` results. `assertion_description()` maps 11 variants to natural language. `last_assertion_results` on CourseApp. Hidden for Predict exercises (spoiler prevention). |
| Service readiness | `wait_for_service_ready()` uses `spawn_stream_reader()` for concurrent stdout/stderr watching via mpsc. `ready_stream` controls which streams to watch. |
| Loopback networking | `run_command_with_loopback()` grants loopback only when exercise defines services. Services always get loopback. |
| Teardown guarantees | Teardown runs unconditionally (even on setup/service/step failure). All LifecycleError variants carry teardown_warnings. Warnings surfaced as yellow text, never block student results. |
| Platform blocking | Course-level `platform:` field dims unavailable courses on Home screen. `check_platform()` validates at startup. |
| Watch mode | PollWatcher (500ms), debounce (300ms), `[t]` toggles auto-test. GUI editors auto-enter via `[E]`. |
| Lesson sandbox | Persistent code at `~/.local/share/learnlocal/sandboxes/`. No grading. LLM gets sandbox-specific prompt. |
| UX discoverability | Rotating tips after 4s idle, condensed key bars, `[?]` help overlay, quickstart banner on first exercise. |
| Home two-panel | Left=courses, right=lessons. `HomePanelFocus` enum. Focus border colors (Cyan/DarkGray). |
| Celebrations | Exercise flash (400ms), lesson box with progress, course complete with stats. |

---

## Test Coverage

**207 tests without features, 223 with `--features llm`:**

| Module | Tests | What's tested |
|---|---|---|
| `config` | 6 | Config loading, defaults, serde, sandbox pref |
| `course::types` | 9 | Serde roundtrips for all types, env service with capture, env ports |
| `course::loader` | 7 | Missing dir, split_content_sections (3), split_display_sections (3), fixture loading |
| `course::validator` | 3 | Valid course, cycle detection, bad semver |
| `exec::sandbox` | 13 | File I/O, echo, stdin, exit codes, truncation, timeout, sandbox detection, loopback |
| `exec::placeholder` | 3 | All placeholders, files list, output derivation |
| `exec::validate` | 4 | Output match, mismatch, regex, compile-only |
| `exec::environment` | 16 | Env setup (dirs/files/symlinks/env vars/cwd/permissions), port allocation, service readiness (delay/pattern/stderr/both/timeout/crasher), file content dir substitution |
| `exec::toolcheck` | 3 | Step command extraction, env command extraction, suggest_install |
| `state::types` | 4 | Progress key, serde roundtrips |
| `state::progress` | 2 | Default creation, save/reload |
| `ui::app` | 1 | Screen enum comparison |
| `ui::course_app` | 1 | AppState enum comparison |
| `ui::celebration` | 3 | Stats computation, format_duration, mini_progress_bar |
| `ui::diff` | 6 | Output diff rendering (identical, mismatches, empty, multi-line, NO_COLOR) |
| `ui::editor` | 2 | Editor detection (config priority, env fallback) |
| `ui::inline_editor` | 17 | Arrow nav, insert/delete, backspace merge, Home/End, Tab, wrap, Esc/Ctrl+S actions |
| `ui::markdown` | 8 | Headings (H1/H2 underlines), code blocks, bold, lists, inline code, tables, horizontal rules |
| `ui::diagnostics` | 8 | gcc/clang parsing, fatal errors, notes, multi-diagnostic, raw fallback, NO_COLOR |
| `llm::chat` | 3 | Chat state lifecycle, reset, role serialization |
| `llm::config` | 4 | LLM config loading, defaults, partial YAML |
| `llm::context` | 4 | Context assembly, content inclusion toggle, system prompt sections, execution result formatting |
| `llm::ollama` | 5 | Response parsing, done detection, request serialization, model list |

---

## Courses

### C++ Fundamentals v2.0.0

**8 lessons, 55 exercises** — validated with g++ -std=c++17 -Wall -Wextra

| Lesson | Dir | Exercises | Key Topics |
|---|---|---|---|
| Hello World & Program Structure | `01-hello-world/` | 6 | #include, main(), cout, comments, escapes |
| Variables and Types | `02-variables/` | 7 | int/double/char/bool, const, auto, scope |
| Operators and Expressions | `03-operators/` | 7 | arithmetic, comparison, logical, precedence, casting |
| Control Flow | `04-control-flow/` | 8 | if/switch, while/for/do-while, break/continue, nested |
| Functions | `05-functions/` | 7 | definition, params, return, overloading, pass-by-ref, recursion |
| Arrays and Strings | `06-arrays-strings/` | 7 | C-arrays, iteration, std::string, bounds checking |
| Pointers and References | `07-pointers/` | 7 | address-of, dereference, arithmetic, nullptr, array pointers |
| Structs | `08-structs/` | 6 | definition, init, functions, arrays-of-structs, nested |

**Language steps:** compile (g++ -std=c++17) → run

### Python Fundamentals v1.0.0

**8 lessons, 54 exercises** — validated with python3

| Lesson | Dir | Exercises | Key Topics |
|---|---|---|---|
| Hello World & Getting Started | `00-hello-world/` | 6 | print(), strings, comments, escapes, input() |
| Variables and Types | `01-variables/` | 7 | assignment, types, type(), dynamic typing, f-strings |
| Operators and Expressions | `02-operators/` | 7 | arithmetic, comparison, logical, //, %, casting |
| Control Flow | `03-control-flow/` | 8 | if/elif/else, while, for/range(), break/continue, nested |
| Functions | `04-functions/` | 7 | def, params, return, defaults, multiple return, scope, recursion |
| Lists and Tuples | `05-lists-tuples/` | 7 | indexing, methods, slicing, comprehensions, tuples, sorting |
| Dictionaries | `06-dictionaries/` | 6 | creation, methods, iteration, .get(), word counting |
| String Methods | `07-string-methods/` | 6 | case, find/replace, split/join, formatting, palindrome |

**Language steps:** run (python3) — no compile step

**Exercise types used:** write (from scratch), fix (buggy starter code). Some exercises use `input:` field for stdin piping.

### JS (Node.js) Fundamentals v1.0.0

**8 lessons, 56 exercises** — validated with node

| Lesson | Dir | Exercises | Key Topics |
|---|---|---|---|
| Hello World | `00-hello-world/` | 6 | console.log, strings, template literals |
| Variables | `01-variables/` | 7 | let/const, types, typeof, type coercion |
| Operators | `02-operators/` | 7 | arithmetic, comparison, strict equality, ternary |
| Control Flow | `03-control-flow/` | 8 | if/switch, while/for, for..of, break/continue |
| Functions | `04-functions/` | 7 | declaration, arrow, defaults, rest params, closures |
| Arrays | `05-arrays/` | 7 | methods, iteration, destructuring, spread |
| Objects | `06-objects/` | 7 | literals, methods, destructuring, this, classes |
| Modern Syntax | `07-modern-syntax/` | 7 | modules, promises, async/await, optional chaining |

**Language steps:** run (node) — no compile step

### AI Fundamentals (Python) v1.0.0

**8 lessons, 56 exercises** — validated with python3 (pure stdlib, no pip)

| Lesson | Dir | Exercises | Key Topics |
|---|---|---|---|
| Vectors and Math | `00-vectors-and-math/` | 7 | dot product, magnitude, normalization |
| Similarity and Search | `01-similarity-and-search/` | 7 | cosine similarity, k-nearest neighbors |
| The Perceptron | `02-the-perceptron/` | 7 | weights, bias, step function, training |
| Neural Networks | `03-neural-networks/` | 7 | layers, forward pass, activation functions |
| Training and Optimization | `04-training-and-optimization/` | 7 | loss, gradients, learning rate, batches |
| Data Quality | `05-data-quality/` | 7 | cleaning, normalization, train/test split |
| Fairness and Bias | `06-fairness-and-bias/` | 7 | demographic parity, equal opportunity, mitigation |
| Attention Mechanism | `07-attention-mechanism/` | 7 | queries/keys/values, softmax, multi-head |

**Language steps:** run (python3) — no compile step. All exercises use fixed seeds for deterministic output.

### Linux Fundamentals v1.0.0

**8 lessons, 55 exercises** — validated with bash (platform: linux)

| Lesson | Dir | Exercises | Key Topics |
|---|---|---|---|
| Filesystem | `01-filesystem/` | 7 | paths, ls, mkdir, cd, pwd |
| File Operations | `02-file-operations/` | 7 | cp, mv, rm, touch, find |
| Viewing and Searching | `03-viewing-searching/` | 7 | cat, head, tail, grep, wc |
| Permissions | `04-permissions/` | 7 | chmod, chown, umask, special bits |
| Pipes and Redirection | `05-pipes-redirection/` | 7 | \|, >, >>, 2>, tee, xargs |
| Text Processing | `06-text-processing/` | 7 | sed, awk, sort, cut, tr, uniq |
| Processes | `07-processes/` | 7 | ps, kill, bg/fg, signals, /proc |
| Users and Networking | `08-users-networking/` | 6 | whoami, id, groups, curl, ssh basics |

**Language steps:** run (bash). All exercises create own test data in temp dirs. System-dependent output uses regex validation.

### Env Engine Test v1.0.0

**2 lessons, 6 exercises** — test course for environment engine features (platform: linux)

---

## Dependencies

```toml
# Core
clap = "4.4"          # CLI parsing (derive macros)
thiserror = "1.0"      # Typed error enums
anyhow = "1.0"         # Top-level error handling
serde = "1.0"          # Serialization (YAML + JSON)
serde_yaml = "0.9"     # Course YAML parsing
serde_json = "1.0"     # Progress JSON
ratatui = "0.25"       # Terminal UI framework
crossterm = "0.27"     # Terminal backend
pulldown-cmark = "0.9"  # Markdown parsing
dirs = "5.0"           # XDG dirs (~/.local/share, ~/.config)
chrono = "0.4"         # Timestamps
tempfile = "3.8"       # Sandbox temp dirs
semver = "1.0"         # Course version parsing
regex = "1.10"         # Regex validation method
notify = "6.1"         # File watching (PollWatcher for watch mode)

# Optional (behind --features llm):
tokio = "1.0"          # Async runtime (rt, net, time, io-util, macros, sync)
tokio-stream = "0.1"   # StreamExt for byte-stream LLM responses
reqwest = "0.11"       # HTTP client (json, rustls-tls, stream)
```

Rust edition: 2021
