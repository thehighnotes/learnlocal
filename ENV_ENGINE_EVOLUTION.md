# Environment Engine Evolution

## Implementation Status

| Phase | Description | Status |
|---|---|---|
| **Phase A** | Bug fixes (teardown, permissions, path validation, UTF-8) | **DONE** |
| **Phase B** | Workspace view | **DONE** |
| **Phase C** | Assertion checklist | **DONE** |
| **Phase D** | Linux course conversion (~35 exercises) | Planned |
| **Phase E** | SQL Fundamentals course | Planned |

---

## Current State Assessment

### What Exists (Capability Layer — Complete)

The engine runs **every exercise** through `run_lifecycle()` in `runner.rs`. It's already the universal execution path. The 7-phase pipeline works:

1. Sandbox creation (tmpdir + firejail/bwrap)
2. Filesystem setup (dirs, files, symlinks, permissions, env vars, ports, cwd)
3. Setup commands (sequential, `{dir}` placeholder, any binary)
4. Background services (loopback networking, ready pattern/delay, pipe draining)
5. Student files + language steps (loopback when services defined)
6. Teardown commands (full placeholder substitution, warnings not errors)
7. Service cleanup (ServiceGuard RAII drop)

11 state assertion types: FileExists, DirExists, FileNotExists, DirNotExists, FileContains, FileMatches, FileEquals, Permissions, Symlink, FileCount, DirEmpty.

Setup commands are generic — `sqlite3`, `git`, `python3`, anything that's on `$PATH`. The engine doesn't care what you run. Dynamic ports allocate ephemeral ports and inject `LEARNLOCAL_PORT_0..N`. Services get loopback networking. Teardown failures surface as yellow warnings. All env spec fields are `#[serde(default)]` — backward compatible.

### What Was Missing (Now Fixed — Phases A-C)

| Gap | Fix | Phase |
|-----|-----|-------|
| Students can't see what the engine set up | **Workspace view** — bordered panel shows files (with content preview), dirs, symlinks, services, ports, setup steps | B |
| Assertions shown only after failure | **Assertion checklist** — pre-run `○` expectations, post-run `✔`/`✘` results in ExercisePrompt, ResultSuccess, ResultFail views | C |
| Teardown skipped on step failure | **Unconditional teardown** — pipeline captures errors without early returns, runs teardown, attaches warnings to error. All LifecycleError variants carry teardown_warnings | A |
| Assertion paths not validated | **Path safety** — `check_env_path()` on all assertion paths in validator + `validate_path_safety()` runtime guard in `check_assertion()` | A |
| No assertion path safety in validator | **Validator checks** — assertion paths, symlink targets, FileMatches regex patterns all validated at build time | A |
| Silent permission failure | **Error on invalid octal** — `u32::from_str_radix` failure now returns `LearnLocalError::Execution` instead of silent skip | A |
| UTF-8 truncation panic | **Safe truncation** — `truncate_output()` finds last valid char boundary before byte limit | A |

### Current Course Engine Usage

| Course | Exercises | Engine Features Used | Env Engine Opportunity |
|--------|-----------|---------------------|----------------------|
| C++ Fundamentals | 55 | None (pure stdout) | Future: file I/O lessons |
| Python Fundamentals | 54 | None (pure stdout) | Future: file I/O, csv, sqlite lessons |
| JS Fundamentals | 56 | None (pure stdout) | Future: file I/O, HTTP, JSON lessons |
| AI Fundamentals | 56 | None (pure stdout) | Future: data file processing |
| Linux Fundamentals | 55 | None (self-contained scripts) | ~20 exercises convertible now |
| Env Engine Test | 6 | Full (files, services, ports, assertions) | Test course only |

---

## The Vision: All-in-Engine Courses

Every course — not just Linux and SQL — will eventually have exercises that need environment setup. A Python course doesn't stop at `print("hello")`. It grows into file I/O, databases, APIs. The engine isn't a niche feature for "complex" courses — it's core infrastructure that every course will use as it matures.

### What "All-in-Engine" Means

```
┌─────────────────────────────────────────────────────┐
│                  Student View Layer                  │
│  Workspace panel • Assertion checklist • Diff view   │  ← NEW: visibility
├─────────────────────────────────────────────────────┤
│                 Validation Layer                     │
│  Output match • Regex • State assertions • Combined  │  ← EXISTS: expanded
├─────────────────────────────────────────────────────┤
│                 Engine Core                          │
│  Files • Dirs • Services • Ports • Setup • Teardown  │  ← EXISTS: complete
├─────────────────────────────────────────────────────┤
│                 Sandbox Layer                        │
│  tmpdir • firejail/bwrap • timeout • network control │  ← EXISTS: complete
└─────────────────────────────────────────────────────┘
```

The bottom two layers exist. The top two need work. The engine has capability but lacks visibility and feedback.

---

## Feature 1: Workspace View — IMPLEMENTED

### What It Does

When an exercise defines an `environment:` block with visible content (files, dirs, symlinks, services, ports), the exercise view shows a **WORKSPACE** panel between the exercise prompt and the code box.

### Actual Rendering

```
  Exercise: Sort a data file

  [WRITE]

  Sort the contents of data.txt alphabetically.

  ┌─ WORKSPACE ──────────────────────────────────────┐
  │  📄 data.txt                                      │
  │    cherry                                          │
  │    apple                                           │
  │    banana                                          │
  │    date                                            │
  │                                                    │
  │  📁 output/                                        │
  └──────────────────────────────────────────────────┘

  ┌─ main.sh ────────────────────────────────────────┐
  │ 1  #!/bin/bash                                    │
  │ 2  # Your solution here                           │
  └──────────────────────────────────────────────────┘
```

With services:

```
  ┌─ WORKSPACE ──────────────────────────────────────┐
  │  🗄 app.db (seeded via setup)                     │
  │  📄 schema.sql                                    │
  │    CREATE TABLE users (id INTEGER, name TEXT);     │
  │    ...                                             │
  │  🔌 sqlite3 on app.db                              │
  └──────────────────────────────────────────────────┘
```

### Implementation (completed)

**Method:** `CourseApp::render_workspace_lines(env: &EnvironmentSpec, theme: &Theme) -> Vec<Line>` — static method, ~130 lines.

**Location:** `render_exercise_prompt()` in `course_app.rs`, after prompt text and before code box. Lines inserted into the scrollable content vector.

**What's shown:**
- Files (📄): filename + permissions if set + first 4 lines of content. Longer files show "... (N more lines)" in muted.
- Directories (📁): path with trailing `/`
- Symlinks (🔗): link → target
- Services (🔌): name + command
- Ports (🌐): "N dynamic port(s)" with pluralization
- Setup commands (⚙️): "N setup step(s)" count summary

**Styling:** Box border `theme.code_border` (DarkGray), labels `theme.keyword`, content `theme.body_text`, muted details `theme.muted`. WORKSPACE header in top border using box-drawing chars.

**Visibility rules:**
- Only rendered when `exercise.environment` is `Some` AND has visible content (any of: files, dirs, symlinks, services, ports > 0, setup commands)
- Empty `EnvironmentSpec` → nothing shown
- Pure compute exercises → nothing shown

**Scroll integration:** Lines added to existing `Vec<Line>`, handled by `render_scrollable()`. No new scroll state needed.

---

## Feature 2: Assertion Checklist — IMPLEMENTED

### What It Does

When an exercise uses `validation.method: state`, the exercise view shows a **checklist** of what will be validated. After running, the checklist updates with pass/fail per item.

### Actual Rendering — Before Run

```
  ┌─ EXPECTED ───────────────────────────────────────┐
  │  ○ output/report.txt exists                       │
  │  ○ output/report.txt contains "Total:"            │
  │  ○ output/ directory has 3 files                  │
  └──────────────────────────────────────────────────┘
```

### Actual Rendering — After Run (Mixed Results)

```
  ┌─ RESULTS ────────────────────────────────────────┐
  │  ✔ output/report.txt exists                       │
  │  ✘ output/report.txt contains "Total:"            │
  │    → file does not contain "Total:"               │
  │  ✔ output/ directory has 3 files                  │
  └──────────────────────────────────────────────────┘
```

### Implementation (completed)

**Three methods on CourseApp:**

1. `render_assertion_checklist_lines(assertions, theme)` — Pre-run view. `○` unchecked items in `theme.body_text`. EXPECTED header in bordered panel.
2. `render_assertion_results_lines(results, theme)` — Post-run view. `✔` in `theme.success` (green), `✘` in `theme.error` (red). RESULTS header turns green with ✔ when all pass. Failed items show detail text (e.g., `→ file does not contain "Total:"`) in `theme.muted`.
3. `assertion_description(assertion)` — Maps all 11 `StateAssertion` variants to natural language strings:

| Variant | Example Output |
|---|---|
| `FileExists("output.txt")` | `output.txt exists` |
| `DirExists("output")` | `output/ exists` |
| `FileNotExists("temp.txt")` | `temp.txt does not exist` |
| `DirNotExists("tmp")` | `tmp/ does not exist` |
| `FileContains({path: "log.txt", content: "Total:"})` | `log.txt contains "Total:"` |
| `FileMatches({path: "out.txt", pattern: "\\d+"})` | `out.txt matches /\d+/` |
| `FileEquals({path: "result.txt", content: "hello"})` | `result.txt equals "hello"` |
| `Permissions({path: "run.sh", mode: "755"})` | `run.sh has permissions 755` |
| `Symlink({path: "link", target: "file"})` | `link → file` |
| `FileCount({path: "output", count: 3})` | `output/ has 3 entries` |
| `DirEmpty("cache")` | `cache/ is empty` |

Content previews truncated at 30 chars with `...` suffix.

**State tracking:** `last_assertion_results: Option<Vec<AssertionResult>>` on `CourseApp`.
- Set in `process_submit_result()` for `StateAssertionFailed` results
- Cleared in `reset_session_for_current_exercise()` and before each new submission
- Renders in ExercisePrompt (post-run state persists until next exercise), ResultSuccess, and ResultFail

**Spoiler prevention:** Pre-run checklist hidden for `ExerciseType::Predict` (the expected state IS the answer). State assertions still show for non-Predict exercises.

---

## Feature 3: Bug Fixes — ALL FIXED

### Bug 1: Teardown Skipped on Failure — FIXED

**Problem:** Four early `return Ok(Err(...))` in `run_lifecycle()` exited before teardown. Teardown commands (DB dumps, log captures) were lost on failure.

**Fix:** Restructured `run_lifecycle()` with `phase_error: Option<LifecycleError>` pattern. Each phase sets `phase_error` and `break`s instead of returning early. Teardown always runs. All `LifecycleError` variants now carry `teardown_warnings: Vec<String>`. Both public API functions propagate warnings from error paths (previously returned `Vec::new()`).

### Bug 1b: Silent Permission Failure — FIXED

**Problem:** `if let Ok(mode) = u32::from_str_radix(mode_str, 8)` silently ignored invalid octal permissions like `"abc"`.

**Fix:** Changed to `u32::from_str_radix(...).map_err(...)? ` — returns `LearnLocalError::Execution` with message `"Invalid octal permission 'abc' for file 'script.sh'"`.

### Bug 2: Assertion Paths Not Validated — FIXED

**Problem:** `check_env_path()` validated env files/dirs/symlinks but NOT assertion paths. Could read outside sandbox via `../../../etc/passwd`.

**Fix (two layers):**
1. **Validator** (`validator.rs`): iterates all assertions, calls `check_env_path()` on every path (all 11 variants). Symlink assertions check both `path` and `target`. FileMatches assertions validate regex with `Regex::new()`.
2. **Runtime** (`environment.rs`): `assertion_path()` helper extracts path from any variant. `check_assertion()` calls `validate_path_safety()` before `sandbox_dir.join()`.

### Bug 3: UTF-8 Safe Output Truncation — FIXED

**Problem:** `truncate_output()` sliced at byte boundary, could split multi-byte UTF-8 chars.

**Fix:** Reverse scan `(0..=max_bytes).rev().find(|&i| s.is_char_boundary(i))` finds last valid char boundary.

---

## Course Conversion Strategy

### Guiding Principle

**Don't break what works. Convert when conversion adds genuine value.**

A conversion is justified when it:
1. Removes boilerplate that isn't teaching material
2. Makes the exercise intent clearer
3. Enables better feedback (workspace view, assertion checklist)

A conversion is NOT justified when it:
1. Removes code that IS the learning objective
2. Adds complexity without pedagogical benefit
3. Makes solutions non-standalone

### Linux Fundamentals: The Proof of Concept

The Linux course has three exercise categories:

**Category A: Given data, operate on it (~20 exercises)**

The student gets data files and must process them. Currently, the solution creates the data as boilerplate.

```yaml
# BEFORE: boilerplate mixed with task
solution: |
  #!/bin/bash
  TMPD=$(mktemp -d)
  printf "cherry\napple\nbanana\n" > "$TMPD/data.txt"   # ← setup boilerplate
  sort "$TMPD/data.txt"                                   # ← actual task
  rm -rf "$TMPD"                                          # ← cleanup boilerplate

# AFTER: environment handles setup, student focuses on task
environment:
  files:
    - path: data.txt
      content: "cherry\napple\nbanana\n"
solution: |
  #!/bin/bash
  sort data.txt
```

**Conversion verdict: YES.** The `mktemp`/`printf`/`rm -rf` is boilerplate. The workspace view shows the student what files exist. The solution becomes the pure skill being taught. Shorter, cleaner, more focused.

**Category B: Creation IS the task (~25 exercises)**

The student must create directories, files, permissions, symlinks. This IS the learning objective.

```yaml
# STAYS AS-IS: creation is the skill being taught
solution: |
  #!/bin/bash
  TMPD=$(mktemp -d)
  mkdir -p "$TMPD/project/src" "$TMPD/project/docs"
  chmod 755 "$TMPD/project/src"
```

But we CAN add state assertions to validate the result directly:

```yaml
# ENHANCED: state assertions replace stdout-based verification
environment:
  assertions:
    - dir_exists: project/src
    - dir_exists: project/docs
    - permissions:
        path: project/src
        mode: "755"
validation:
  method: state
```

Currently, these exercises force the student to run `find | sort` or `stat --format='%a'` to PROVE they did the right thing. State assertions check directly — no verification boilerplate needed.

**Conversion verdict: PARTIAL.** Keep the solution as-is (creation is the task). Add assertions as validation method. Remove the `stat`/`find` proof-printing from solutions and expected_output.

**Category C: System tools and pipes (~10 exercises)**

Text processing, process management, networking. These are pure compute (stdin→stdout) and don't need environment blocks.

**Conversion verdict: NO.** These work perfectly as-is.

### Pure Compute Courses (C++, Python, JS, AI): No Conversion Needed

These courses have zero exercises that need environment blocks today. The workspace view and assertion checklist simply won't appear for them. No changes needed, no quality lost.

**Future growth:** When these courses add file I/O / database / API lessons (Phase 5+), those new exercises will be authored with environment blocks from the start. The infrastructure will already be there.

### Summary: Conversion Scope

| Course | Exercises to Convert | Type of Change |
|--------|---------------------|----------------|
| Linux | ~20 | Add `environment.files`, remove mktemp/cleanup boilerplate |
| Linux | ~15 | Add `validation.assertions`, remove stat/find proof-printing |
| Linux | ~20 | No change (pure compute, creation-is-task) |
| C++/Python/JS/AI | 0 | No change |
| **Total** | ~35 of 276 | ~13% of all exercises |

---

## New Courses Enabled

With workspace view + assertion checklist, these courses become practical:

### SQL (SQLite) Fundamentals — Zero Install

```yaml
# course.yaml
language:
  name: sql
  steps:
    - name: run-query
      command: sqlite3
      args: ["{dir}/app.db", ".read {main}"]
      capture_output: true
      check_exit_code: true

# exercise.yaml
environment:
  setup:
    - name: seed-db
      command: sqlite3
      args: ["{dir}/app.db"]
      stdin: |
        CREATE TABLE products (name TEXT, price REAL, stock INTEGER);
        INSERT INTO products VALUES ('Widget', 9.99, 150);
        INSERT INTO products VALUES ('Gadget', 24.99, 75);
  teardown:
    - name: dump-state
      command: sqlite3
      args: ["{dir}/app.db", ".dump"]
      capture_to: "db_final_state.sql"
```

The student writes SQL queries. The workspace view shows the database schema and sample data. Teardown captures the final DB state for LLM diagnostics. No installation beyond `sqlite3` (pre-installed on most Linux/macOS).

### Git Fundamentals

```yaml
environment:
  setup:
    - name: init-repo
      command: git
      args: ["init", "{dir}/repo"]
    - name: create-files
      command: sh
      args: ["-c", "cd {dir}/repo && echo 'hello' > readme.txt && git add . && git commit -m 'initial'"]
  assertions:
    - file_exists: repo/.git/refs/heads/feature
validation:
  method: state
```

Student practices git commands. Assertions check repo state directly (branches exist, files committed, etc.). No stdout parsing needed.

### Go Web / HTTP API

```yaml
environment:
  ports: 1
  files:
    - path: server.go
      content: |
        package main
        import (...)
        func main() {
            http.HandleFunc("/api/status", func(w http.ResponseWriter, r *http.Request) {
                json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
            })
            http.ListenAndServe(":"+os.Getenv("LEARNLOCAL_PORT_0"), nil)
        }
  setup:
    - name: build-server
      command: go
      args: ["build", "-o", "{dir}/server", "{dir}/server.go"]
  services:
    - name: api
      command: "{dir}/server"
      ready_pattern: "listening"
      ready_stream: stderr
```

Student writes HTTP client code. Server is built and started by the engine. Port is dynamically allocated. Workspace view shows the API is running.

---

## Implementation Order

### Phase A: Bug Fixes — DONE

**Files changed:** `src/exec/runner.rs`, `src/exec/environment.rs`, `src/exec/sandbox.rs`, `src/course/validator.rs`

1. **Teardown on failure** — Restructured `run_lifecycle()` to use `phase_error: Option<LifecycleError>` pattern. Each phase (setup, services, student steps) sets `phase_error` and breaks instead of returning early. Teardown always runs. All `LifecycleError` variants now carry `teardown_warnings: Vec<String>`. Both public API functions (`run_exercise_with_sandbox`, `execute_exercise_with_sandbox`) propagate warnings from error paths.
2. **Silent permission failure** — `u32::from_str_radix` failure now returns `LearnLocalError::Execution` with clear message instead of silently skipping via `if let Ok(...)`.
3. **Assertion path validation** — Validator iterates all assertions, calls `check_env_path()` on every path (FileExists, DirExists, FileContains, FileMatches, Permissions, Symlink paths AND targets, FileCount). Also validates FileMatches regex patterns with `Regex::new()`. Runtime adds `assertion_path()` helper + `validate_path_safety()` guard in `check_assertion()`.
4. **UTF-8 safe truncation** — `truncate_output()` uses reverse scan `(0..=max_bytes).rev().find(|&i| s.is_char_boundary(i))` to find last valid char boundary.

### Phase B: Workspace View — DONE

**Files changed:** `src/ui/course_app.rs`

1. `render_workspace_lines(env, theme)` — static method, returns `Vec<Line>`. Shows files (📄 with first 4 content lines), dirs (📁), symlinks (🔗), services (🔌), ports (🌐), setup steps (⚙️). Bordered panel with WORKSPACE header.
2. Integrated into `render_exercise_prompt()` after prompt text, before code box. Only renders when exercise has environment with visible content.
3. Sandbox view skipped — lesson sandbox is lesson-level (no exercise environment blocks).

### Phase C: Assertion Checklist — DONE

**Files changed:** `src/ui/course_app.rs`

1. `assertion_description()` — maps all 11 `StateAssertion` variants to human-readable strings. Content previews truncated at 30 chars.
2. `render_assertion_checklist_lines()` — pre-run view with `○` unchecked items. EXPECTED header. Shown in ExercisePrompt for `validation.method: state` exercises (hidden for Predict type).
3. `render_assertion_results_lines()` — post-run view with `✔` (green) / `✘` (red) per assertion. RESULTS header (green with ✔ when all pass). Failed assertions show detail text in muted color.
4. `last_assertion_results: Option<Vec<AssertionResult>>` on CourseApp — set after graded submission in `process_submit_result()`, cleared in `reset_session_for_current_exercise()`.
5. Post-run results shown in ExercisePrompt (after returning from result view), ResultSuccess (after explanation), and ResultFail (via existing FailureDetail rendering).

### Phase D: Linux Course Conversion (planned)

1. Identify the ~20 Category A exercises (given data, operate on it)
2. Convert each: add `environment.files`, simplify solution, update expected_output
3. Identify the ~15 Category B exercises (creation + verification)
4. Enhance each: add `validation.assertions`, change method to `state`, remove proof-printing
5. Run `learnlocal validate` on entire course — ensure all 55 exercises pass
6. Manual testing of converted exercises with workspace view

### Phase E: SQL Fundamentals Course (~10-12 hours)

1. Design 8 lessons, ~55 exercises (matching quality of existing courses)
2. Every exercise uses `environment:` blocks (setup commands seed SQLite, teardown captures state)
3. Mix of output validation (query results) and state validation (table structure)
4. Workspace view shows schema, sample data
5. Validate and test all exercises

---

## What Stays the Same

These things do NOT change:

- **The `language.steps` execution model** — Steps still run student code. The engine adds context around them, doesn't replace them.
- **Output-based validation** — Still the primary validation method for pure compute exercises. State assertions are an additional method, not a replacement.
- **Course YAML format** — All new fields already exist and are `#[serde(default)]`. No schema changes.
- **Backward compatibility** — Exercises without `environment:` work identically.
- **The spec** — Section 3.9 already documents everything the engine can do. The workspace view and assertion checklist are UI features, not engine changes.

## What We're NOT Doing

- **Domain-specific YAML sugar** (`databases:`, `git:`, `packages:`) — Nice to have but not needed. Raw `setup` commands handle everything. Can add later per-domain as courses warrant it.
- **Embedded runtimes** (`rusqlite` compiled into the binary) — Interesting for zero-install SQL, but a significant dependency and binary size increase. Defer to Phase 5.
- **Package management** (`pip install`, `npm install`) — Conflicts with offline-first. Not practical.
- **Converting pure compute exercises** — C++/Python/JS/AI don't need env blocks today. Don't force them.

---

## Quality Evaluation

### What Gets Better

1. **Exercise focus** — Solutions contain ONLY the skill being taught. No mktemp, no rm -rf, no stat --format.
2. **Visual context** — Students see the workspace state before coding. Better than reading setup code mentally.
3. **Direct feedback** — "✔ permissions correct" is clearer than comparing stat output strings.
4. **Shorter solutions** — 1-3 lines of actual code vs 6-10 lines of setup + code + cleanup.
5. **Course authoring** — Declaring `files: [{path: data.txt, content: ...}]` is clearer than embedding `printf > file` in solutions.
6. **New course types enabled** — SQL, Git, APIs become practical.

### What Gets Worse

1. **Solution portability** — Env-engine solutions aren't standalone scripts. You can't copy-paste `sort data.txt` into a terminal without first creating `data.txt`. The old `mktemp + printf + sort + rm` works anywhere.
2. **Consistency within Linux course** — Some exercises will use workspace view (given data), others won't (creation tasks). Two exercise models in one course. Mitigated by the assertion checklist being consistent across both.
3. **Learning `mktemp` and cleanup patterns** — Students currently learn `TMPD=$(mktemp -d)` ... `rm -rf "$TMPD"` as a discipline. With env engine, they don't practice this. **Mitigation:** Dedicate 2-3 exercises specifically to temp dir management. Don't scatter it across every exercise as boilerplate.

### Net Assessment

For Linux course: **net positive** — the workspace view and assertion checklist provide better feedback than stdout string comparison. The loss of mktemp practice is mitigated by dedicated exercises.

For pure compute courses: **neutral** — no changes, no impact.

For new courses (SQL, Git, etc.): **strongly positive** — these courses literally cannot exist without the engine. The workspace view is essential for showing database state, repo state, etc.
