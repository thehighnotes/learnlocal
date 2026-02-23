# LearnLocal: Offline Programming Tutorial Framework

**Version:** 0.2 Draft
**Date:** February 7, 2026
**Author:** Mark Wind
**Purpose:** Specification for Claude Code implementation
**Revision note:** v2 — incorporates architectural review. Changes from v1: generalized execution model, multi-file exercises, stdin support, sandboxed execution, $EDITOR-first editing, LLM decoupled but context-ready, course validation in Phase 1, semver course versioning.

---

## 1. Project Overview

### 1.1 Problem Statement

No fully-local, guided, structured programming tutorial exists. Current options require:
- Web connectivity (Codecademy, Sololearn)
- External mentoring (Exercism)
- Non-interactive formats (books, PDFs)

Developers wanting to learn a new language offline have no `vimtutor`-style experience.

### 1.2 Solution

LearnLocal is a terminal-based tutorial runtime that:
- Runs entirely offline
- Provides guided lessons with explanations
- Validates exercises via actual compilers/interpreters
- Tracks progress locally
- Supports any programming language via pluggable course packs
- Optionally integrates local LLM for hints (Ollama, etc.)

### 1.3 Design Principles

1. **Offline-first** — No network required for core functionality
2. **Language-agnostic** — Course format works for any language
3. **Single binary** — Minimal dependencies for end user
4. **Content-code separation** — Runtime knows nothing about specific languages; courses define everything
5. **Progressive** — Lessons build on each other with dependency tracking
6. **Safe by default** — Exercise execution is sandboxed and time-limited

### 1.4 Contribution Model

Two distinct contributor surfaces:
- **Course authors** write YAML + Markdown. No Rust knowledge needed. This is where the library grows.
- **Runtime contributors** work on the Rust codebase. Smaller pool, focused scope.

---

## 2. Architecture

### 2.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                        User                                 │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     Terminal UI                             │
│  - Lesson renderer (markdown subset)                        │
│  - Exercise prompt                                          │
│  - $EDITOR integration                                      │
│  - Progress display                                         │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    Core Runtime                             │
│  - Course loader/parser                                     │
│  - Lesson sequencer                                         │
│  - State tracker (progress + context signals)               │
│  - Execution orchestrator (sandboxed)                       │
└──────────┬──────────────────┬───────────────────┬───────────┘
           │                  │                   │
┌──────────▼──────┐ ┌─────────▼────────┐ ┌───────▼──────────┐
│  Course Packs   │ │ Language Backends│ │  LLM Plugin      │
│  (YAML + MD)    │ │ (sandboxed exec) │ │  (optional,      │
│                 │ │                  │ │   feature-gated) │
└─────────────────┘ └──────────────────┘ └──────────────────┘
```

### 2.2 Directory Structure

```
learnlocal/
├── src/
│   ├── main.rs                   # Entry point
│   ├── course/
│   │   ├── mod.rs
│   │   ├── loader.rs             # YAML parsing
│   │   ├── validator.rs          # Course structure validation
│   │   └── types.rs              # Course/Lesson/Exercise structs
│   ├── exec/
│   │   ├── mod.rs
│   │   ├── runner.rs             # Step-based execution
│   │   └── sandbox.rs            # Sandboxing (timeout, temp dir, firejail)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── terminal.rs           # TUI rendering
│   │   ├── markdown.rs           # MD subset renderer
│   │   └── editor.rs             # $EDITOR integration + minimal fallback
│   ├── state/
│   │   ├── mod.rs
│   │   ├── progress.rs           # Completion tracking
│   │   └── signals.rs            # Raw context signals (attempts, time, hints)
│   └── llm/                      # Behind --features llm
│       ├── mod.rs
│       ├── context.rs            # Context view over state signals
│       ├── ollama.rs
│       └── remote.rs
│
├── courses/
│   ├── cpp-fundamentals/
│   ├── python-fundamentals/
│   └── rust-fundamentals/
│
├── Cargo.toml
└── README.md
```

### 2.3 Technology Choice: Rust

**Rationale:**
- Single static binary (no runtime dependencies)
- Excellent terminal UI crates (ratatui, crossterm)
- Fast startup time
- Cross-platform (Linux primary, macOS, Windows)
- Memory safe
- Contributor model: course authors don't touch Rust; runtime is a focused codebase

**Core dependencies:**
```toml
[dependencies]
ratatui = "0.25"
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
pulldown-cmark = "0.9"
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }

[features]
llm = ["tokio", "reqwest"]

[dependencies.tokio]
version = "1.0"
features = ["full"]
optional = true

[dependencies.reqwest]
version = "0.11"
features = ["json"]
optional = true
```

Note: `tokio` and `reqwest` are only compiled when `--features llm` is enabled. The core binary has no async runtime and no HTTP dependencies.

---

## 3. Course Format Specification

### 3.1 Course Structure

```
courses/
└── cpp-fundamentals/
    ├── course.yaml               # Course metadata + execution model
    ├── lessons/
    │   ├── 01-variables/
    │   │   ├── lesson.yaml       # Lesson config
    │   │   ├── content.md        # Explanation text
    │   │   ├── exercises/
    │   │   │   ├── 01-declare.yaml
    │   │   │   ├── 02-assign.yaml
    │   │   │   └── 03-types.yaml
    │   │   └── assets/           # Optional (ASCII diagrams, etc.)
    │   │       └── memory.txt
    │   ├── 02-pointers/
    │   └── ...
    └── templates/
        └── main.cpp              # Shared boilerplate (optional)
```

### 3.2 course.yaml Schema

```yaml
name: "C++ Fundamentals"
version: "1.0.0"                  # Semver. See Section 3.8.
description: "Learn C++ from the ground up"
author: "LearnLocal Community"
license: "CC-BY-4.0"

language:
  id: cpp
  display_name: "C++"
  extension: ".cpp"

  # Execution steps (replaces fixed compile/run model)
  # Steps run in order. Each step can use placeholders:
  #   {files}    — space-separated list of all exercise files
  #   {main}     — the primary source file
  #   {output}   — derived binary name (stem of main file)
  #   {dir}      — the sandbox temp directory
  steps:
    - name: compile
      command: "g++"
      args: ["-std=c++17", "-Wall", "-Wextra", "-o", "{dir}/{output}", "{dir}/{main}"]
      check_exit_code: true       # Fail exercise if this step fails
    - name: run
      command: "{dir}/{output}"
      args: []
      capture_output: true        # This step's stdout is checked by validation

  # Execution limits
  limits:
    timeout_seconds: 10           # Per-step timeout
    max_output_bytes: 65536       # Truncate output beyond this

# Lesson order and dependencies
lessons:
  - id: variables
    title: "Variables and Types"

  - id: operators
    title: "Operators and Expressions"
    requires: [variables]

  - id: control-flow
    title: "Control Flow"
    requires: [variables, operators]

  - id: pointers
    title: "Pointers and References"
    requires: [variables]

  - id: functions
    title: "Functions"
    requires: [control-flow]

  - id: classes
    title: "Classes and Objects"
    requires: [functions, pointers]

estimated_minutes_per_lesson: 30

# Optional: restrict course to a specific platform
# platform: linux       # linux | macos | windows (omit for cross-platform)
```

**Interpreted language example (Python):**

```yaml
language:
  id: python
  display_name: "Python"
  extension: ".py"
  steps:
    - name: run
      command: "python3"
      args: ["{dir}/{main}"]
      capture_output: true
  limits:
    timeout_seconds: 10
    max_output_bytes: 65536
```

**Java example (demonstrates why generalized steps matter):**

```yaml
language:
  id: java
  display_name: "Java"
  extension: ".java"
  steps:
    - name: compile
      command: "javac"
      args: ["{dir}/{main}"]
      check_exit_code: true
    - name: run
      command: "java"
      args: ["-cp", "{dir}", "Main"]  # Course convention: entry class is Main
      capture_output: true
  limits:
    timeout_seconds: 15
    max_output_bytes: 65536
```

### 3.3 lesson.yaml Schema

```yaml
id: variables
title: "Variables and Types"
description: "Learn how to declare and use variables in C++"
estimated_minutes: 25

content: content.md

exercises:
  - 01-declare
  - 02-assign
  - 03-types
  - 04-constants

teaches:
  - variable-declaration
  - primitive-types
  - initialization

recap: |
  You learned:
  - Variables store data with a specific type
  - int, float, double, char, bool are primitive types
  - Variables must be declared before use
```

### 3.4 Exercise YAML Schema

```yaml
id: declare
title: "Declare a Variable"
type: "write"                     # write | fix | multiple-choice | fill-blank | predict

prompt: |
  Declare an integer variable named `age` with the value `25`.

# Single-file exercise: use starter
starter: |
  #include <iostream>

  int main() {
      // Your code here

      std::cout << age << std::endl;
      return 0;
  }

# Multi-file exercise: use files instead of starter
# files:
#   - name: "main.cpp"
#     editable: true              # Student edits this file
#     content: |
#       #include "utils.h"
#       int main() {
#           // Your code here
#           return 0;
#       }
#   - name: "utils.h"
#     editable: false             # Provided, read-only
#     content: |
#       #pragma once
#       int add(int a, int b);
#   - name: "utils.cpp"
#     editable: true
#     content: |
#       #include "utils.h"
#       // Implement add() here

# stdin input piped to the program during validation
# input: "5\n10\n"

# Validation
validation:
  method: "output"                # output | compile-only | regex | custom
  expected_output: "25"
  # For regex: pattern: "^25$"
  # For custom: script: "validate.sh" (receives temp dir as $1)

# Progressive hints
hints:
  - "Variables in C++ need a type before the name"
  - "The syntax is: type name = value;"
  - "For integers, the type is `int`"

# Shown if user gives up
solution: |
  #include <iostream>

  int main() {
      int age = 25;

      std::cout << age << std::endl;
      return 0;
  }

# For multi-file: solution_files instead of solution
# solution_files:
#   - name: "main.cpp"
#     content: |
#       ...
#   - name: "utils.cpp"
#     content: |
#       ...

# Environment specification (optional)
# Defines filesystem setup, background services, and state assertions
# for exercises that need more than simple "write code → check output"
# environment:
#   # Files to create in sandbox before student code runs
#   files:
#     - path: "config.json"
#       content: '{"port": 8080, "workdir": "{dir}"}'
#       permissions: "644"            # Optional, octal string
#
#   # Directories to create
#   dirs:
#     - "output"
#     - "data/raw"
#
#   # Symbolic links
#   symlinks:
#     - source: "latest"
#       target: "data/v1"
#
#   # Environment variables (supports {dir} substitution)
#   env:
#     APP_CONFIG: "{dir}/config.json"
#     DATABASE_URL: "sqlite:{dir}/test.db"
#
#   # Working directory override (relative to sandbox)
#   cwd: "src"
#
#   # Dynamic port allocation (injected as LEARNLOCAL_PORT_0, _1, etc.)
#   ports: 2                          # 0-10
#
#   # Setup commands (run before student code, {dir} placeholder only)
#   setup:
#     - name: "init-db"
#       command: "sqlite3"
#       args: ["{dir}/test.db", ".read {dir}/schema.sql"]
#       timeout_seconds: 5            # Optional, defaults to course timeout
#       capture_to: "setup.log"       # Optional, capture stdout to sandbox file
#
#   # Background services (started before student code, killed after)
#   services:
#     - name: "api-server"
#       command: "python3"
#       args: ["{dir}/server.py"]     # {dir} substituted
#       ready_pattern: "listening on"  # Regex — wait for this on stdout/stderr
#       ready_stream: "stderr"         # stdout | stderr | both (default: both)
#       ready_timeout_seconds: 10      # Default: 10
#       ready_delay_ms: 200            # Used when no ready_pattern (default: 200)
#       capture_stdout: "server_out.log"  # Optional: capture to sandbox file
#       capture_stderr: "server_err.log"  # Optional: capture to sandbox file
#
#   # Teardown commands (run after student code, full placeholder substitution)
#   teardown:
#     - name: "dump-db"
#       command: "sqlite3"
#       args: ["{dir}/test.db", ".dump"]
#       capture_to: "db_dump.txt"
#
#   # State assertions (used with validation.method: state)
#   assertions:
#     - file_exists: "output/report.txt"
#     - dir_exists: "output"
#     - file_not_exists: "temp.txt"
#     - file_contains:
#         path: "output/report.txt"
#         pattern: "Total: \\d+"      # Regex
#     - file_equals:
#         path: "output/count.txt"
#         content: "42\n"
#     - permissions:
#         path: "script.sh"
#         expected: "755"

explanation: |
  `int age = 25;` does three things:
  1. Declares a variable named `age`
  2. Specifies its type as `int` (integer)
  3. Initializes it with the value `25`
```

### 3.5 Exercise Types

| Type | Description | Validation | Needs Compilation |
|------|-------------|------------|-------------------|
| `write` | Write code from scratch | Output, regex, or custom | Yes |
| `fix` | Fix buggy code | Output match | Yes |
| `fill-blank` | Complete partial code | Output match | Yes |
| `multiple-choice` | Select correct answer | Exact match | No |
| `predict` | Predict output of code | Exact match | No |

### 3.6 Multi-File Exercise Rules

When an exercise uses `files:` instead of `starter:`:

1. All files are written to the sandbox temp directory.
2. Files with `editable: true` are opened for editing (sequentially or in a single `$EDITOR` session).
3. Files with `editable: false` are provided as-is.
4. The `{main}` placeholder resolves to the first `editable: true` file (or can be overridden with `main_file:` in the exercise yaml).
5. The `{files}` placeholder resolves to all files, space-separated.
6. Solutions use `solution_files:` to match the `files:` structure.

### 3.7 content.md Format

Standard markdown with terminal-renderable subset:

**Supported:**
- Headings (# ## ###)
- Bold, italic, code spans
- Code blocks with language hint
- Tables
- Blockquotes
- Ordered/unordered lists
- Horizontal rules

**Not supported (use ASCII alternatives):**
- Images (use ASCII art)
- Links (display URL inline)
- HTML

### 3.8 Course Versioning

Courses use semantic versioning:

- **Patch** (1.0.0 → 1.0.1): Typo fixes, hint improvements, explanation rewording. Progress carries over.
- **Minor** (1.0.0 → 1.1.0): New exercises added, content expanded, no existing exercises changed. Progress carries over.
- **Major** (1.0.0 → 2.0.0): Exercises changed, reordered, or removed. Progress resets with a user prompt:

```
Course "C++ Fundamentals" updated from v1.x to v2.0.0.
Your progress was for v1. Exercises have changed.
  [k] Keep progress (may skip new content)
  [r] Reset and start fresh
  [b] Back up progress and reset
```

Progress is keyed to `{course_id}@{major_version}` internally.

### 3.9 Environment Specification

Exercises can define an `environment:` block to set up filesystem state, background services, and state-based validation. This enables exercises that go beyond "write code → check output" — such as database queries, API clients, file manipulation, and system administration tasks.

#### Design Principles

1. **Backward compatible** — Exercises without `environment:` work exactly as before. All new fields use `#[serde(default)]`.
2. **Sandboxed** — All environment operations happen within the sandbox temp directory. Paths are validated to prevent escape (`..`, absolute paths).
3. **Deterministic** — Exercises should create their own test data rather than relying on system state. Dynamic ports use `LEARNLOCAL_PORT_N` env vars to avoid conflicts.
4. **Fail-safe** — Teardown failures are warnings, not errors. Services are killed via RAII guard even if the exercise panics.

#### Execution Lifecycle

The environment engine extends the base execution model with a 7-phase pipeline:

```
1. Sandbox creation (tmpdir + optional firejail/bwrap)
2. Filesystem setup
   - Create directories
   - Write files (with {dir} placeholder substitution in content)
   - Create symbolic links
   - Set file permissions
   - Inject environment variables (with {dir} substitution)
   - Allocate dynamic ports → LEARNLOCAL_PORT_0..N env vars
   - Compute cwd override
3. Setup commands
   - Run sequentially, {dir} placeholder only (student files don't exist yet)
   - Any non-zero exit code → SetupFailed (exercise stops)
4. Start background services
   - Each service spawned with loopback networking (firejail --net=lo, bwrap omits --unshare-net)
   - {dir} substituted in service args
   - Wait for readiness (pattern match on stdout/stderr, or delay)
   - Drain unconsumed pipes to prevent service hangs
   - Optional: capture stdout/stderr to sandbox files
5. Write student files + run language steps
   - Student code gets loopback networking when services are defined
   - Normal execution: each step runs, check_exit_code, capture_output
6. Teardown commands
   - Run sequentially, full placeholder substitution ({dir}, {main}, {output}, {files})
   - Failures collected as warnings (yellow text in UI), not errors
   - Optional: capture stdout to sandbox file
7. Kill services (ServiceGuard RAII drop)
```

#### Service Readiness

Two modes:

**Pattern mode** (`ready_pattern` set): Spawns reader threads on stdout and/or stderr (controlled by `ready_stream`). First regex match on either watched stream triggers readiness. Reader threads continue draining after match to prevent pipe backup. Times out after `ready_timeout_seconds`.

**Delay mode** (no `ready_pattern`): Sleeps `ready_delay_ms`, then checks the process hasn't crashed (non-zero exit).

#### State Validation

When `validation.method: state`, the engine validates assertions against the sandbox filesystem after student code runs. Supports 11 assertion types:

| Assertion | YAML Key | Description |
|---|---|---|
| File exists | `file_exists: path` | Check file exists |
| Dir exists | `dir_exists: path` | Check directory exists |
| File not exists | `file_not_exists: path` | Check file doesn't exist |
| Dir not exists | `dir_not_exists: path` | Check directory doesn't exist |
| File contains | `file_contains: {path, pattern}` | Regex match on file content |
| File matches | `file_matches: {path, pattern}` | Full-content regex match |
| File equals | `file_equals: {path, content}` | Exact content match |
| Permissions | `permissions: {path, expected}` | Octal permission match |
| Symlink | `symlink: {path, target}` | Symlink target match |
| File count | `file_count: {path, count}` | Number of files in directory |
| Dir empty | `dir_empty: path` | Directory is empty |

State assertions can be combined with `expected_output` for exercises that need both filesystem state and output validation.

#### Dynamic Port Allocation

Setting `ports: N` in the environment spec causes the engine to bind N TcpListeners to `127.0.0.1:0`, record the assigned ports, and immediately drop the listeners. The ports are injected as environment variables `LEARNLOCAL_PORT_0` through `LEARNLOCAL_PORT_{N-1}`.

Course authors use `$LEARNLOCAL_PORT_0` in their scripts — this works naturally in bash, Python, Node.js, Go, etc. The TOCTOU race (port freed before service binds) is acceptable for a tutorial framework.

#### Platform Restriction

Courses can set `platform: linux` (or `macos`, `windows`) in `course.yaml` to restrict which OS can run them. The home screen dims unavailable courses, hides the Enter key, and shows a red "linux only" (etc.) label. The CLI `start` command bails with a clear error message.

---

## 4. Execution & Sandboxing

### 4.1 Execution Model

Exercises are validated by running the course-defined execution steps in a sandboxed environment.

```rust
// Pseudocode — simplified from actual run_lifecycle() implementation
fn execute_exercise(
    course: &Course,
    exercise: &Exercise,
    user_files: &[ExerciseFile],
) -> ExecutionResult {
    // 1. Create sandbox
    let sandbox = Sandbox::new(&course.language.limits)?;

    // 2. Set up environment (if defined)
    let (env_vars, cwd_override) = if let Some(env_spec) = &exercise.environment {
        setup_environment(sandbox.dir(), env_spec)?  // dirs, files, symlinks, env vars, ports
    } else {
        (None, None)
    };

    // 3. Run setup commands
    for step in env_spec.setup {
        run_env_command(&sandbox, step)?;  // {dir} placeholder only
    }

    // 4. Start background services
    for svc in env_spec.services {
        let child = sandbox.spawn_service(&svc.command, &svc.args)?;  // loopback networking
        wait_for_service_ready(&mut child, &svc)?;  // pattern or delay mode
        drain_service_pipes(&mut child);  // prevent pipe backup
    }

    // 5. Write student files, run language steps
    for file in user_files {
        sandbox.write_file(&file.name, &file.content)?;
    }
    let needs_loopback = !env_spec.services.is_empty();
    for step in &course.language.steps {
        last_output = sandbox.run_command_with_loopback(
            &step.command, &step.args, needs_loopback,
        )?;
    }

    // 6. Run teardown commands (failures → warnings, not errors)
    for step in env_spec.teardown {
        run_env_command_full(&sandbox, step)?;  // full placeholder substitution
    }

    // 7. Kill services (ServiceGuard RAII drop), validate output or state
    validate(&exercise.validation, &last_output, sandbox.dir())
}
```

### 4.2 Sandbox Specification

Every exercise execution runs inside a sandbox with these constraints:

| Constraint | Implementation | Default |
|-----------|---------------|---------|
| **Timeout** | SIGKILL after N seconds | 10s (course-configurable) |
| **Temp directory** | All files written to isolated tmpdir, cleaned up after | Always |
| **Output limit** | Truncate stdout/stderr beyond N bytes | 64KB |
| **Filesystem** | Process runs inside temp dir only | Via firejail/bwrap if available |
| **Network** | Blocked if sandbox tool available | Best-effort |
| **Loopback** | Enabled when exercise defines services | Via run_command_with_loopback() |

**Tiered sandboxing:**

```rust
pub enum SandboxLevel {
    /// Just timeout + temp dir. Always available.
    Basic,
    /// Uses firejail/bwrap for filesystem + network isolation.
    /// Detected at startup, used if available.
    Contained,
}

impl Sandbox {
    pub fn detect_best() -> SandboxLevel {
        if which("firejail").is_ok() || which("bwrap").is_ok() {
            SandboxLevel::Contained
        } else {
            SandboxLevel::Basic
        }
    }
}
```

On first run, if only `Basic` is available:

```
Note: For extra safety when running community courses, install
firejail or bubblewrap:
  sudo apt install firejail    # or: sudo apt install bubblewrap

LearnLocal works fine without them, but they isolate exercise
code from your filesystem. Recommended if you use third-party courses.
```

### 4.3 stdin Support

Exercises can define input that gets piped to the program:

```yaml
# Exercise: read two numbers and print their sum
id: read-sum
title: "Read and Sum"
type: write

prompt: |
  Read two integers from stdin and print their sum.

starter: |
  #include <iostream>

  int main() {
      // Read two integers and print their sum
      return 0;
  }

input: "5\n10\n"

validation:
  method: output
  expected_output: "15"
```

The `input` field is piped to the process's stdin during the `capture_output: true` step.

---

## 5. State Tracking

### 5.1 Design Principle: Raw Signals

The state layer records **raw signals**, not derived interpretations. This keeps the core simple and gives the LLM (or future analytics) maximum flexibility.

Raw signals recorded per exercise attempt:
- Timestamp
- User code submitted
- Compile result (exit code, stdout, stderr)
- Run result (exit code, stdout, stderr, matched)
- Hints revealed so far
- Time spent (seconds since exercise was displayed)

The core runtime does NOT compute "struggling" or "mastery." It stores facts. Interpretation is the LLM's job (or a future dashboard's job).

### 5.2 Progress File

Location: `~/.local/share/learnlocal/progress.json`

```json
{
  "version": 2,
  "courses": {
    "cpp-fundamentals@1": {
      "course_version": "1.2.0",
      "started_at": "2026-02-07T10:30:00Z",
      "last_activity": "2026-02-07T14:22:00Z",
      "lessons": {
        "variables": {
          "status": "completed",
          "completed_at": "2026-02-07T11:00:00Z",
          "exercises": {
            "01-declare": {
              "status": "completed",
              "attempts": [
                {
                  "timestamp": "2026-02-07T10:35:00Z",
                  "time_spent_seconds": 45,
                  "compile_success": true,
                  "run_exit_code": 0,
                  "output_matched": true,
                  "hints_revealed": 0
                }
              ]
            },
            "02-assign": {
              "status": "completed",
              "attempts": [
                {
                  "timestamp": "2026-02-07T10:40:00Z",
                  "time_spent_seconds": 120,
                  "compile_success": false,
                  "hints_revealed": 0
                },
                {
                  "timestamp": "2026-02-07T10:43:00Z",
                  "time_spent_seconds": 180,
                  "compile_success": true,
                  "run_exit_code": 0,
                  "output_matched": true,
                  "hints_revealed": 1
                }
              ]
            }
          }
        },
        "pointers": {
          "status": "in_progress",
          "exercises": {
            "01-address": {
              "status": "completed",
              "attempts": [{"...": "..."}]
            },
            "02-dereference": {
              "status": "in_progress",
              "attempts": [
                {
                  "timestamp": "2026-02-07T14:10:00Z",
                  "time_spent_seconds": 300,
                  "compile_success": true,
                  "run_exit_code": 0,
                  "output_matched": false,
                  "hints_revealed": 2
                },
                {
                  "timestamp": "2026-02-07T14:18:00Z",
                  "time_spent_seconds": 480,
                  "compile_success": true,
                  "run_exit_code": 0,
                  "output_matched": false,
                  "hints_revealed": 3
                }
              ]
            }
          }
        }
      }
    }
  }
}
```

Note: Full user code is NOT stored in progress.json (could be large). Only the signals. The LLM gets the current code from the live session state.

### 5.3 Session State (In-Memory)

Held in memory during an active session, not persisted:

```rust
pub struct SessionState {
    /// Current code the user is editing
    pub current_code: Vec<ExerciseFile>,

    /// Full attempt history for this exercise (code included)
    pub attempt_history: Vec<FullAttempt>,

    /// Clock: when this exercise was first shown
    pub exercise_started_at: Instant,

    /// Which hints have been revealed
    pub hints_revealed: usize,

    /// Last compile/run output (for display and LLM context)
    pub last_execution: Option<ExecutionResult>,
}

pub struct FullAttempt {
    pub code: Vec<ExerciseFile>,
    pub execution_result: ExecutionResult,
    pub timestamp: DateTime<Utc>,
    pub time_spent_seconds: u64,
    pub hints_revealed: usize,
}
```

This session state is what the LLM context view reads from. It's rich (includes code), but ephemeral.

---

## 6. Terminal UI

### 6.1 Screen Architecture

The TUI uses a **two-level state machine**:

- **Outer App** routes between screens: Home, Settings, Progress, Course.
- **Inner CourseApp** handles the exercise flow (8 states) when a course is active.

`learnlocal` (no args) launches the Home screen. `learnlocal start <course>` jumps directly into Course.

```
Screen::Home       →  Course list with progress, entry point
Screen::Settings   →  Editable config (editor, sandbox, AI model)
Screen::Progress   →  Per-course lesson/exercise breakdown
Screen::Course     →  Exercise flow (LessonContent → ExercisePrompt → ... → CourseComplete)
```

Navigation flow:
```
Home  ──Enter──>  Course (loads full course, creates CourseApp)
Home  ──s──────>  Settings
Home  ──p──────>  Progress (for selected course)
Settings  ──Esc──>  Home (saves config)
Progress  ──Esc──>  Home
Progress  ──Enter──>  Course (at selected lesson)
Course  ──Esc──>  Home (from LessonContent or CourseComplete)
```

### 6.2 Home Screen

```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal                                                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Select a course to begin or continue learning.              │
│                                                              │
│  > C++ Fundamentals         v1.0.0   60% | 3/5 lessons      │
│    Rust Basics              v1.2.0   Not started             │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ [Enter] Start  [p] Progress  [s] Settings  [q] Quit         │
└─────────────────────────────────────────────────────────────┘
```

The Home screen uses `CourseInfo` — a lightweight struct that only reads `course.yaml` (no lesson/exercise loading). Progress summaries are computed from `ProgressStore` cross-referenced with `CourseInfo`.

Keys: `j/k` or arrows to navigate, Enter to start/continue, `p` for progress detail, `s` for settings, `q` to quit.

### 6.3 Settings Screen

```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal | Settings                                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  GENERAL                                                     │
│                                                              │
│  > Editor              nvim                                  │
│    Sandbox Level       < auto >                              │
│                                                              │
│  AI                                                          │
│                                                              │
│    Ollama URL          http://localhost:11434                 │
│    Model               qwen3:4b                              │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ [Enter] Edit  [Left/Right] Toggle  [Esc] Save & Back        │
└─────────────────────────────────────────────────────────────┘
```

- Text fields (Editor, Ollama URL): Enter to edit, type, Enter to confirm, Esc to cancel.
- Enum fields (Sandbox Level): Left/Right to cycle between auto/basic/contained.
- Model field: Enter fetches Ollama models (blocking short-timeout HTTP call), shows selectable list. Falls back to text edit if Ollama is unreachable.
- Saves via `Config::save()` on Esc (leaving screen).
- AI section only visible with `--features llm` build.

### 6.4 Progress Screen

```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal | C++ Fundamentals v1.0.0                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Started: 2026-01-15    Last active: 2026-02-07              │
│  Overall: 60%  (3/5 lessons, 12/20 exercises)                │
│                                                              │
│    [x] 01. Variables          4/4 exercises                  │
│    [x] 02. Operators          4/4 exercises                  │
│    [x] 03. Control Flow       4/4 exercises                  │
│  > [~] 04. Functions          2/4 exercises                  │
│    [ ] 05. Pointers           0/4 exercises                  │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ [Enter] Resume from here  [Esc] Back                         │
└─────────────────────────────────────────────────────────────┘
```

Loads the full `Course` on entry (needed for lesson titles + exercise counts). Enter on a lesson jumps into Course at that lesson.

### 6.5 Course Screen (Exercise Flow)

The exercise flow is identical to the pre-redesign TUI. It lives in `CourseApp` and cycles through these states:

```
LessonContent → ExercisePrompt → Editing → Executing →
  ResultSuccess/ResultFail → LessonRecap → CourseComplete
```

Layout within Course screen:
```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal │ C++ Fundamentals │ Lesson 4/5 │ Exercise 2/4   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Exercise: Parameters                                        │
│  [WRITE]                                                     │
│                                                              │
│  Write a function that takes two int parameters...           │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  1  #include <iostream>                                │  │
│  │  2  // Your code here                                  │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ [e] Edit  [Enter] Submit  [h] Hint  [s] Skip  [Esc] Home    │
└─────────────────────────────────────────────────────────────┘
```

`CourseApp` methods accept `&mut ProgressStore`, `&Config`, and `SandboxLevel` as parameters (not owned) to satisfy the borrow checker when the outer `App` needs disjoint access. It returns `CourseAction` (Continue/Quit/GoHome) so the outer shell can route.

### 6.6 Code Editing: $EDITOR-First

The primary editing experience uses the user's preferred editor.

**Flow:**

1. User presses `[e]` (or exercise begins).
2. Starter code is written to `$TMPDIR/learnlocal/exercise.cpp`.
3. `$EDITOR` (or `$VISUAL`, falling back to `vi`) opens the file.
4. User edits, saves, quits editor.
5. LearnLocal reads back the file and displays the updated preview.
6. User presses `[Enter]` to submit for validation.

**Benefits:**
- Users get their own editor (vim, nano, helix, micro, VS Code, etc.)
- Syntax highlighting, keybindings, autocomplete — all free
- No TUI text editor to build or maintain

**Minimal fallback:**
If `$EDITOR` is unset and `vi` is not available, a simple line-based input mode:

```
Line 7 (current: "      // Your code here"):
> int age = 25;
```

This is intentionally basic to encourage setting `$EDITOR`.

**External editor configuration:**
```bash
# In user's shell profile
export EDITOR=vim
# or
export LEARNLOCAL_EDITOR=helix  # overrides $EDITOR for learnlocal only
```

### 6.7 Colors and Themes

```rust
struct Theme {
    heading: Color,      // Cyan
    code: Color,         // White
    keyword: Color,      // Yellow
    string_lit: Color,   // Green
    comment: Color,      // Gray
    error: Color,        // Red
    success: Color,      // Green
    prompt: Color,       // Blue
    muted: Color,        // DarkGray
}

// Respect NO_COLOR standard
if std::env::var("NO_COLOR").is_ok() {
    // All colors become default terminal color
}
```

---

## 7. Runtime Behavior

### 7.1 CLI Interface

```bash
# Launch Home screen (course browser, settings, progress)
$ learnlocal

# List available courses (non-interactive)
$ learnlocal list
Available courses:
  cpp-fundamentals    C++ Fundamentals v1.0.0 (5 lessons)

# Jump directly into a course (bypasses Home screen)
$ learnlocal start cpp-fundamentals

# Jump to specific lesson
$ learnlocal start cpp-fundamentals --lesson pointers

# Check progress (non-interactive)
$ learnlocal progress cpp-fundamentals

# Reset progress
$ learnlocal reset cpp-fundamentals

# Validate a course (for course authors)
$ learnlocal validate courses/cpp-fundamentals

# Specify custom courses directory
$ learnlocal --courses ~/my-courses list
```

`learnlocal` (no subcommand) is the primary entry point. It discovers courses via `load_course_info()` (lightweight — only reads `course.yaml`, no lesson/exercise loading) and launches the Home screen TUI. `learnlocal start` is preserved for scripting and direct access.

### 7.2 State Machine

The TUI uses a two-level state machine:

**Outer level (App)** — screen routing:
```
                    ┌─────────────┐
                    │    HOME     │◄──────────────────┐
                    │ (courses)   │                   │
                    └──┬───┬───┬──┘                   │
                       │   │   │                      │
              ┌────────┘   │   └────────┐             │
              ▼            ▼            ▼             │
        ┌──────────┐ ┌──────────┐ ┌──────────┐       │
        │ SETTINGS │ │ PROGRESS │ │  COURSE  │───────┘
        │  (edit   │ │ (detail) │ │(exercise │  GoHome
        │  config) │ │          │ │  flow)   │
        └────┬─────┘ └────┬─────┘ └────┬─────┘
             │            │            │
             └────────────┴────────────┘
                    Esc → Home
```

**Inner level (CourseApp)** — exercise flow within Course screen:
```
                    ┌──────▼──────┐
                    │ SHOW LESSON │◄────────────────┐
                    │  (content)  │                 │
                    └──────┬──────┘                 │
                           │                        │
                    ┌──────▼──────┐                 │
              ┌────►│  EXERCISE   │                 │
              │     │  (prompt)   │                 │
              │     └──────┬──────┘                 │
              │            │                        │
              │     ┌──────▼──────┐                 │
              │     │   EDITOR    │                 │
              │     │  ($EDITOR)  │                 │
              │     └──────┬──────┘                 │
              │            │                        │
              │     ┌──────▼──────┐                 │
              │     │  VALIDATE   │                 │
              │     │ (sandboxed) │                 │
              │     └──────┬──────┘                 │
              │            │                        │
              │    ┌───────┴───────┐                │
              │    │               │                │
         ┌────▼────▼──┐      ┌─────▼─────┐         │
         │   FAIL     │      │  SUCCESS  │         │
         │ show error │      │ next ex?  │         │
         │ offer hint │      └─────┬─────┘         │
         └────────────┘            │               │
                           ┌───────┴───────┐       │
                           │               │       │
                    ┌──────▼──────┐ ┌──────▼──────┐│
                    │ NEXT EXER.  │ │ LESSON DONE ├┘
                    └─────────────┘ │  (recap)    │
                                    └──────┬──────┘
                                           │
                                    ┌──────▼──────┐
                                    │ NEXT LESSON │
                                    │  or DONE    │
                                    └─────────────┘
```

`CourseApp.handle_input()` returns `CourseAction` (Continue/Quit/GoHome) so the outer App can route back to Home when the user presses Esc or finishes the course.

### 7.3 Course Validation Tool

Available in Phase 1 for course authors:

```bash
$ learnlocal validate courses/cpp-fundamentals

Validating cpp-fundamentals v1.0.0...
  [ok]  course.yaml schema valid
  [ok]  All lessons referenced in course.yaml exist
  [ok]  All exercises referenced in lesson.yaml exist
  [ok]  No dependency cycles in lesson graph
  [ok]  All exercises have valid schemas
  [ok]  All exercises have at least one hint
  [ok]  All solutions provided

Running solutions against validation...
  [ok]  variables/01-declare: compiles, output matches
  [ok]  variables/02-assign: compiles, output matches
  [ok]  variables/03-types: compiles, output matches
  [FAIL] operators/02-precedence: expected "15" got "12"
         ^^^ Solution may be wrong, or expected_output needs fixing

Validation: 11/12 passed, 1 failed.
```

This catches broken exercises before they ship.

---

## 8. LLM Integration (Optional, Feature-Gated)

### 8.1 Architectural Boundary

The LLM subsystem is strictly optional:

- **Compile-time:** Behind `--features llm` Cargo feature flag. Default build has no async runtime or HTTP dependencies.
- **Runtime:** Behind `config.llm.enabled` (toggled in Settings screen). Without it, AI keybindings are hidden.
- **Data flow:** The LLM reads from the state layer (Section 5) via a context view. It never writes to state. One-way dependency: `llm → state`, never `state → llm`.

```
┌──────────────┐     reads     ┌──────────────┐
│  State Layer │ ◄──────────── │  LLM Plugin  │
│  (progress   │               │  (context    │
│   + signals  │               │   view +     │
│   + session) │               │   backends)  │
└──────────────┘               └──────────────┘
       ▲                              │
       │ writes                       │ displays
       │                              ▼
┌──────────────┐               ┌──────────────┐
│ Core Runtime │               │   UI Layer   │
└──────────────┘               └──────────────┘
```

### 8.2 Context View

The LLM context is a **read-only view** assembled from data the core already tracks:

```rust
/// Assembled from SessionState + Progress + Course data.
/// This struct exists only in the llm module.
pub struct LlmContext {
    // Course
    pub course_name: String,
    pub language: String,

    // Lesson
    pub lesson_title: String,
    pub lesson_content: String,          // Full markdown
    pub lesson_position: String,         // "Lesson 4 of 12"
    pub concepts_taught: Vec<String>,

    // Exercise
    pub exercise_title: String,
    pub exercise_prompt: String,
    pub exercise_type: String,
    pub starter_code: Vec<ExerciseFile>,

    // Live session state (from SessionState)
    pub current_code: Vec<ExerciseFile>,
    pub attempt_number: u32,
    pub attempt_history: Vec<FullAttempt>,  // Includes code
    pub hints_revealed: Vec<String>,
    pub time_spent_seconds: u64,
    pub last_execution: Option<ExecutionResult>,

    // Previous exercises in this lesson (from Progress)
    pub completed_exercises: Vec<CompletedExerciseSummary>,

    // Course-wide signals (from Progress)
    pub lessons_completed: u32,
    pub total_lessons: u32,
}
```

Notably absent: no `struggles: Vec<String>`. The raw signals (attempt counts, time spent, hints used per exercise) are all present. The LLM interprets what "struggling" means from those signals.

### 8.3 Context Rendering

```rust
impl LlmContext {
    pub fn to_system_prompt(&self) -> String {
        format!(r#"
You are a friendly programming tutor helping a student learn {language}.

## Current Position
- Course: {course_name}
- {lesson_position}
- Exercise: "{exercise_title}" (attempt #{attempt_number})

## Lesson Content
{lesson_content}

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

## Attempt History
{attempt_summary}

## Hints Already Revealed
{hints_summary}

## Guidelines
- Be encouraging but don't give away the answer
- Reference specific line numbers when relevant
- Connect to concepts from the lesson content
- If they're close, tell them
- If they're stuck in a loop, try a different explanation angle
- Keep responses concise (under 200 words unless explaining a concept)
"#,
            // ... field substitutions
        )
    }
}
```

### 8.4 Backend Abstraction

```rust
pub trait LlmBackend: Send + Sync {
    /// Check if backend is available
    fn is_available(&self) -> impl Future<Output = bool> + Send;

    /// Send a message with context, get response
    fn chat(&self, context: &LlmContext, message: &str) -> impl Future<Output = Result<String>> + Send;

    /// Stream response token by token
    fn chat_stream(
        &self,
        context: &LlmContext,
        message: &str,
        callback: Box<dyn Fn(&str) + Send>,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Backend display name
    fn name(&self) -> &str;
}
```

Backends: Ollama (local), Remote (e.g. AGX via WireGuard), OpenAI-compatible (any endpoint).

### 8.5 Configuration

Location: `~/.config/learnlocal/config.yaml`

```yaml
llm:
  backend: ollama           # ollama | remote | openai-compatible | disabled

  ollama:
    url: "http://localhost:11434"
    model: "qwen3:4b"
    fallback_models: ["llama3:8b", "mistral:7b"]

  remote:
    url: "http://10.0.0.50:8080"
    api_key: "${LEARNLOCAL_REMOTE_KEY}"
    model: "qwen3-4b-trt"
    api_format: "ollama"    # ollama | openai | custom
    timeout_seconds: 30

  openai_compatible:
    url: "https://api.anthropic.com/v1"
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-sonnet-4-5-20250929"

  settings:
    max_tokens: 500
    temperature: 0.3
    include_lesson_content: true
    include_previous_exercises: true
    max_history_attempts: 3
```

### 8.6 Quick Actions vs Chat

**Quick Actions (single keypress):**

| Key | Action | Canned Prompt |
|-----|--------|---------------|
| `h` | Hint | "Give me a small hint without revealing the answer" |
| `e` | Explain Error | "Explain this compiler error in simple terms: {error}" |
| `c` | Check Approach | "Is my approach correct? Don't give the answer" |
| `w` | Why Wrong | "My output is {actual} but expected {expected}. Why?" |

**Chat Mode (`a` key):**
- Opens chat panel (bottom split)
- Full conversation history within session
- Free-form questions
- Context automatically included

### 8.7 Backend Selection

```rust
pub async fn select_best_backend(config: &LlmConfig) -> Result<Box<dyn LlmBackend>> {
    let preferred = &config.backend;

    // Try preferred first, then fall through
    let order = match preferred.as_str() {
        "remote" => vec!["remote", "ollama"],
        "ollama" => vec!["ollama"],
        "openai-compatible" => vec!["openai-compatible"],
        _ => vec!["ollama"],
    };

    for name in order {
        let backend = build_backend(name, config)?;
        if backend.is_available().await {
            return Ok(backend);
        }
    }

    Err(Error::NoBackendAvailable)
}
```

### 8.8 Configuration

AI is enabled via the config file (`config.llm.enabled: true`) or the Settings screen `[s]` toggle. Both `learnlocal` (Home screen) and `learnlocal start` respect this setting — no CLI flag needed.

```bash
learnlocal ai-status                                      # Check backends
```

---

## 9. Implementation Phases

### Phase 1: Core Runtime (MVP)

**Goal:** Single course works end-to-end. No LLM, no fancy editor.

**Deliverables:**
- [ ] Course loader (parse course.yaml, lesson.yaml, exercise.yaml)
- [ ] Course validator (`learnlocal validate`)
- [ ] Markdown renderer (terminal subset)
- [ ] Step-based execution engine with basic sandbox (timeout + tmpdir)
- [ ] Progress tracker with raw signal recording
- [ ] $EDITOR integration
- [ ] TUI: lesson view, exercise display, submit, hints, navigate
- [ ] One complete course: `cpp-fundamentals` (5 lessons, ~20 exercises)
- [ ] stdin support for exercises
- [ ] Multi-file exercise support

**Commands working:**
```bash
learnlocal list
learnlocal start cpp-fundamentals
learnlocal progress cpp-fundamentals
learnlocal reset cpp-fundamentals
learnlocal validate courses/cpp-fundamentals
```

---

### Phase 2: UX Polish

**Goal:** Feels good to use.

**Deliverables:**
- [ ] Colored diff for wrong output (expected vs actual)
- [ ] Success/failure animations
- [ ] Keyboard shortcuts help overlay
- [ ] `--lesson` flag to jump to specific lesson
- [ ] Configurable $LEARNLOCAL_EDITOR
- [ ] Course dependency graph visualization in `progress` command
- [ ] Firejail/bwrap sandbox integration (tiered)

---

### Phase 3: LLM Integration

**Goal:** Optional AI tutoring works.

**Deliverables:**
- [ ] LLM context view assembly from state layer
- [ ] Ollama backend
- [ ] Remote backend
- [ ] OpenAI-compatible backend
- [ ] AI config toggle, quick actions, chat panel
- [ ] Backend auto-selection with fallback
- [ ] Streaming responses in chat panel
- [ ] Graceful degradation if backend unavailable

---

### Phase 4: Additional Courses, Environment Engine & Community

**Goal:** Prove language-agnostic design, enable complex exercise types, enable community.

**Deliverables:**
- [x] `python-fundamentals` course (8 lessons, 54 exercises)
- [x] `js-fundamentals` course (8 lessons, 56 exercises)
- [x] `ai-fundamentals-python` course (8 lessons, 56 exercises — pure stdlib)
- [x] `linux-fundamentals` course (8 lessons, 55 exercises — platform: linux)
- [x] Environment engine v3 (filesystem setup, services, ports, state assertions, teardown warnings)
- [x] Platform blocking (course-level OS restriction)
- [x] Progressive reveal (H2-based content sections, Space to advance)
- [x] AI chat during lesson reading
- [x] Lesson sandbox (freeform coding playground)
- [x] UX discoverability (How To page, rotating tips, help overlay, quickstart banner)
- [x] Home QOL (two-panel layout, lesson list, Stats screen)
- [x] Watch mode (file watching with auto-recompile/test)
- [ ] Course contribution guide
- [ ] Course template (`learnlocal init-course`)
- [ ] Exercise test harness for CI

---

### Phase 5: Distribution

**Goal:** Easy to install.

**Deliverables:**
- [ ] Pre-built binaries (Linux x64, ARM64, macOS)
- [ ] GitHub releases CI workflow
- [ ] Install script
- [ ] AUR package
- [ ] Homebrew formula
- [ ] Separate `learnlocal-courses` repo for community courses

---

## 10. Testing Strategy

### 10.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_course_loader_parses_valid_course() { ... }

    #[test]
    fn test_course_loader_rejects_cycle() { ... }

    #[test]
    fn test_placeholder_substitution() { ... }

    #[test]
    fn test_output_validation_trims_whitespace() { ... }

    #[test]
    fn test_regex_validation() { ... }

    #[test]
    fn test_progress_serialization_roundtrip() { ... }

    #[test]
    fn test_semver_progress_key() { ... }
}
```

### 10.2 Integration Tests

```rust
#[test]
fn test_full_exercise_execution() {
    let course = load_course("courses/cpp-fundamentals").unwrap();
    let exercise = get_exercise(&course, "variables", "01-declare").unwrap();

    // Use the solution as user code
    let result = execute_exercise(&course, &exercise, &exercise.solution_files());
    assert!(result.is_success());
}
```

### 10.3 Course Validation as CI

```yaml
# .github/workflows/validate-courses.yml
name: Validate Courses
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release
      - run: |
          for course in courses/*/; do
            ./target/release/learnlocal validate "$course"
          done
```

---

## 11. Resolved Decisions

Previously open questions, now settled:

| Question | Decision |
|----------|----------|
| Build inline editor or defer to $EDITOR? | **$EDITOR-first**, minimal line-based fallback |
| Course versioning with partial progress? | **Semver**: patch/minor carry progress, major prompts user |
| Sandbox code execution? | **Yes**: tiered (basic always, firejail/bwrap if available) |
| Multi-file exercises? | **Yes**: `files:` array with `editable` flag per file |
| Interactive exercises (stdin)? | **Yes**: `input:` field piped to process |
| LLM coupling? | **Decoupled** (feature-gated), but state layer collects full context signals from day 1 |
| Who computes "struggling"? | **The LLM**, from raw signals. Core stores facts, not interpretations. |

## 12. Remaining Open Questions

1. **Windows support priority:** Linux-first is clear. macOS is straightforward (same POSIX model). Windows needs different sandbox strategy and $EDITOR defaults. Defer to Phase 5?
2. **Course pack distribution:** Git submodules? Separate download command? Bundled with binary? Affects Phase 4-5.
3. **Accessibility:** Screen reader support in ratatui TUI? Worth investigating early.

---

*End of specification v2.*
