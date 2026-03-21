# Sprint 10 — Course Designer, Staged Exercises & Community Platform

## Vision

Three features that form a complete ecosystem:

1. **Interactive Course Designer** — A web-based GUI (`learnlocal author <course-dir>`) that lets anyone build courses visually. Drag-and-drop, form-based editing, live terminal preview, solution auto-testing. Zero YAML knowledge required.

2. **Staged Exercises** — Exercises with escalating validation stages. Student code carries forward (basic → edge cases → optimization). Teaches iterative development — the most important real-world skill.

3. **Community Platform** — A course registry where authors publish courses and students discover them. Upload from the Designer, browse from the TUI. Creates a self-sustaining content ecosystem.

```
Author creates course     →  Publishes to community  →  Student discovers in TUI
    (Web Designer)               (Platform API)            (Browse & Download)
         ↑                                                        │
         └────────────── Student becomes author ←─────────────────┘
```

---

## Feature 1: Staged Exercises

### Concept

Today every exercise is atomic: one prompt, one validation, pass or fail. Staged exercises add depth: same code file, escalating requirements.

```yaml
id: reverse-string
title: "Reverse a String"
type: write
prompt: "Write a function that reverses a string."

starter: |
  fn main() {
      // Reverse the string and print it
  }

# Base validation (non-staged fallback)
validation:
  method: output
  expected_output: "olleh"
hints:
  - "Use chars().rev().collect()"
solution: |
  fn main() { println!("{}", "hello".chars().rev().collect::<String>()); }

# Staged progression — each builds on the student's previous code
stages:
  - id: basic
    title: "Basic Solution"
    prompt: "Make it work for ASCII strings."
    validation:
      method: output
      expected_output: "olleh"
    hints:
      - "Use a loop or built-in reverse"
    solution: |
      fn main() { println!("{}", "hello".chars().rev().collect::<String>()); }
    explanation: "chars().rev().collect() reverses character-by-character."

  - id: unicode
    title: "Handle Unicode"
    prompt: "Now handle multi-byte characters correctly."
    validation:
      method: output
      expected_output: "🌍olleh"
    hints:
      - "chars() already handles Unicode in Rust"
    solution: |
      fn main() { println!("{}", "hello🌍".chars().rev().collect::<String>()); }
    explanation: "Rust's char type is Unicode-aware."

  - id: in-place
    title: "Do It In-Place"
    prompt: "Reverse without allocating a new String."
    validation:
      method: regex
      pattern: "unsafe|as_bytes_mut|swap"
    hints:
      - "You'll need unsafe or byte-level manipulation"
    solution: |
      fn main() {
          let mut s = String::from("hello");
          let bytes = unsafe { s.as_bytes_mut() };
          bytes.reverse();
          println!("{}", s);
      }
    explanation: "In-place reversal requires byte-level access."
```

### Schema Changes

**File: `src/course/types.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseStage {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub prompt: Option<String>,
    pub validation: Validation,
    #[serde(default)]
    pub hints: Vec<String>,
    pub solution: Option<String>,
    pub solution_files: Option<Vec<SolutionFile>>,
    pub explanation: Option<String>,
    #[serde(default)]
    pub additional_files: Vec<ExerciseFile>,
}

// Add to existing Exercise struct:
//   #[serde(default)]
//   pub stages: Vec<ExerciseStage>,
```

Design rules:
- `stages` defaults to empty vec — fully backward compatible via `#[serde(default)]`
- No `starter` in stages — student code carries forward from previous stage
- `additional_files` lets stages introduce new test/reference files
- Each stage has independent `validation`, `hints`, `solution`, `explanation`
- Base exercise fields remain for non-staged exercises and serve as Stage 1 defaults

### Progress Changes

**File: `src/state/types.rs`**

Add to `ExerciseProgress`:
- `current_stage: Option<usize>` — 0-based stage index, None for non-staged
- `completed_stages: Vec<String>` — stage IDs that have been passed

Add to `AttemptRecord`:
- `stage_id: Option<String>` — which stage this attempt was for

All new fields use `#[serde(default)]` for backward compatibility with existing progress.json files.

Completion rules:
- Non-staged: Completed when validation passes (unchanged)
- Staged: Completed only when ALL stages pass
- Individual stage completion tracked in `completed_stages`
- `current_stage` persists across sessions

### Session State Changes

**File: `src/state/signals.rs`**

Add to `SessionState`:
- `current_stage_idx: Option<usize>`
- `stage_hints_revealed: HashMap<usize, usize>` — per-stage hint tracking

Key behavior: `advance_stage()` preserves `current_code`, resets hints for the new stage, increments stage index. Code carries forward — this is the defining mechanic.

### Draft Persistence

**File: `src/state/sandbox.rs`**

Draft path for staged exercises:
```
~/.local/share/learnlocal/drafts/{course}@{major}/{lesson}/{exercise}/stage-{idx}/
```

- Stage completion: draft NOT cleared (code carries forward)
- Exercise completion (all stages): ALL stage drafts cleared
- Exercise entry: load draft from current stage (or starter if none)

### Execution Changes

**File: `src/exec/runner.rs`**

Add `ExecutionResult` variant:
```rust
StageComplete {
    stage_id: String,
    stage_idx: usize,
    is_final: bool,  // true = exercise done, false = more stages
}
```

Logic: if exercise has stages, use `current_stage_idx` to select which stage's `validation` to check. Return `StageComplete` on pass. Caller (CourseApp) handles advancement.

### UI Changes

**File: `src/ui/course_app.rs`**

- Stage indicator in exercise prompt: `Stage 1 of 3: Basic Solution`
- Completed stages shown as checkmarks: `✓ Basic  ✓ Unicode  → In-Place`
- Stage-specific hints displayed (not base hints)
- New `AppState::StageComplete` — intermediate celebration, "Next: [stage title]", Enter to continue
- Result fail shows which STAGE failed with stage-specific hints

### Validator Changes

**File: `src/course/validator.rs`**

- Each stage must have non-empty `hints`
- Each stage must have `solution` or `solution_files`
- Each stage must have valid `validation`
- Stage IDs must be unique within exercise
- `--run-solutions` validates each stage's solution independently

### Testing

- Deserialize staged exercise YAML (round-trip)
- Validate stage constraints
- Progress JSON backward compatibility
- Stage advancement preserves code
- Draft path generation with stage index

---

## Feature 2: Interactive Course Designer

### Architecture

```
learnlocal author <course-dir>
    │
    ├── Starts local HTTP server (axum) on localhost:PORT
    ├── Opens browser automatically (xdg-open / open)
    ├── Serves embedded web UI (HTML/CSS/JS baked into binary)
    ├── REST API for CRUD on courses/lessons/exercises
    ├── WebSocket for terminal preview (pty → xterm.js)
    └── All changes write directly to course directory on disk
```

Feature-gated: `--features author` (keeps core binary lean)

### Technology Stack

**Backend (Rust):**
- `axum` — HTTP server + WebSocket support
- `rust-embed` or `include_dir` — embed static web assets in binary
- `portable-pty` — spawn pseudo-terminal for preview
- Reuses existing: `course::loader`, `course::validator`, `exec::runner`, `exec::toolcheck`

**Frontend (Web, embedded in binary):**
- CodeMirror 6 — code editing with syntax highlighting (solution, starter, environment scripts)
- xterm.js — terminal emulator for live TUI preview
- SortableJS — drag-and-drop reordering
- marked.js — markdown preview for prompts/explanations
- Vanilla JS or Svelte — minimal framework, compiles to small bundle
- All assets embedded — no npm install at runtime

### CLI Integration

**File: `src/cli.rs`**

```rust
#[derive(Subcommand, Debug)]
pub enum Command {
    // ... existing ...

    /// Course authoring tools
    Author {
        #[command(subcommand)]
        subcommand: AuthorCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthorCommand {
    /// Open the interactive course designer (web UI)
    Design {
        /// Course directory path
        path: PathBuf,
        /// Port for the local server (default: auto)
        #[arg(long, default_value = "0")]
        port: u16,
        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },

    /// Run a specific exercise's solution and show output
    RunSolution {
        path: PathBuf,
        lesson: String,
        exercise: String,
        /// Auto-update expected_output in YAML
        #[arg(long)]
        update: bool,
    },

    /// Run ALL solutions, optionally update expected_output fields
    RunAllSolutions {
        path: PathBuf,
        #[arg(long)]
        update: bool,
    },
}
```

### UI Layout

Three-panel layout, all live:

#### Panel 1: Course Index (left sidebar)

```
📚 Python Fundamentals v1.1.0
   [Edit Course Settings]

▼ Lesson 1: Hello World
   ≡ 01-hello          ✓  write
   ≡ 02-fix-print      ✓  fix
   ≡ 03-multi-line     ✓  write
   ≡ 04-escape-chars   ✓  fill-blank
   [+ Add Exercise]

▼ Lesson 2: Variables
   ≡ 01-declare        ● ← editing
   ≡ 02-types          ✓  write
   ≡ 03-dynamic        ⚠  missing hints
   [+ Add Exercise]

▶ Lesson 3: Operators (collapsed)

[+ Add Lesson]
```

Features:
- **Tree view** — expandable lessons → exercises
- **Drag-and-drop reorder** — `≡` grip handles. Reorder exercises within a lesson, reorder lessons in course. Auto-updates file numbering prefixes
- **Validation badges** — ✓ green (passes), ⚠ yellow (warnings), ✗ red (errors)
- **Add/delete** — inline buttons. Delete has confirmation
- **Duplicate** — right-click context menu to copy exercise as template
- **Search/filter** — filter by name, type, or status
- **Lesson content editing** — click lesson title to edit lesson.yaml + content.md

#### Panel 2: Exercise Editor (main area)

```
━━ Exercise: declare ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ID: [declare              ]    Type: [write ▾]
Title: [Declare and Print a Variable                        ]

── Prompt ─────────────────────────── [Preview] [Markdown] ──
│ Declare a variable named `age` with value 25 and print    │
│ it using an f-string.                                      │
──────────────────────────────────────────────────────────────

── Solution ──────────────────────────── [▶ Run Solution] ──
│ age = 25                                                   │
│ print(f"I am {age} years old")                             │
──────────────────────────────────────────────────────────────
  Output: "I am 25 years old"  ✓ captured
  [Use as expected output]

── Starter Code ──────────── [Auto-strip from solution] ────
│ # Your code here                                           │
──────────────────────────────────────────────────────────────

── Validation ──────────────────────────────────────────────
  Method: [output ▾]
  Expected output: "I am 25 years old"  ✓ matches solution
  [Edit] [Test against solution]

── Hints ─────────────────── [+ Add Hint] ─── drag to reorder
  ≡ 1. Use the print() function                    [✏] [✗]
  ≡ 2. Use f-strings: f"text {variable}"           [✏] [✗]
  ≡ 3. print(f"I am {age} years old")              [✏] [✗]

── Explanation ──────────────────────────── [Preview] ──────
│ print() sends text to stdout. f-strings let you embed     │
│ variables directly in string literals using {name} syntax. │
──────────────────────────────────────────────────────────────

── Stages ──────────────────────────── [+ Add Stage] ──────
  (none — single-pass exercise)
  [Convert to staged exercise]

── Advanced ──────────────────────────── (collapsed) ──────
  ▶ Environment (setup, services, teardown)
  ▶ Multi-file configuration
  ▶ Stdin input
  ▶ Code golf

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[💾 Auto-saved]  [▶ Validate]  [👁 Toggle Preview]  [📤 Publish]
```

Key features:
- **Code editors** — CodeMirror with syntax highlighting matching the course language
- **Run Solution** — executes solution, captures stdout, shows inline. One click to set as expected_output
- **Auto-strip** — button that generates starter code by stripping the solution (removes function bodies, blanks key expressions). Author refines from there
- **Validation config** — method dropdown shows conditional fields:
  - `output` → expected output text
  - `regex` → pattern input with live test against solution output
  - `state` → visual assertion builder (add rows, pick type, fill fields)
  - `compile-only` → no additional fields
- **Hint management** — drag-and-drop reorder, inline edit, add/remove
- **Stage builder** — each stage as a collapsible card with its own prompt/validation/hints/solution/explanation. Drag to reorder stages
- **Environment builder** — collapsed by default. Visual builder for dirs, files, env vars, setup commands, services, teardown. For advanced authors only
- **Auto-save** — debounced writes to disk on every change. No save button needed

#### Stage Builder (expanded)

When "Convert to staged exercise" or "+ Add Stage" is clicked:

```
── Stages ─────────────────────────────── [+ Add Stage] ──

  ┌─ Stage 1: Basic Solution ─────────────── ≡ drag ── [✗]
  │  Prompt: [Make it work for ASCII strings.          ]
  │  Solution: [▶ Run]
  │  │ fn main() { println!("{}", "hello"...           │
  │  Output: "olleh" ✓
  │  Validation: output → "olleh" [Use captured output]
  │  Hints: 2 hints [Edit]
  │  Explanation: [Edit]
  └────────────────────────────────────────────────────────

  ┌─ Stage 2: Handle Unicode ─────────────── ≡ drag ── [✗]
  │  Prompt: [Now handle multi-byte characters.        ]
  │  Solution: [▶ Run]
  │  │ fn main() { println!("{}", "hello🌍"...         │
  │  Output: "🌍olleh" ✓
  │  Validation: output → "🌍olleh"
  │  Hints: 1 hint [Edit]
  │  Explanation: [Edit]
  └────────────────────────────────────────────────────────

  ┌─ Stage 3: In-Place ───────────────────── ≡ drag ── [✗]
  │  Prompt: [Reverse without allocating a new String. ]
  │  Solution: [▶ Run]
  │  Validation: regex → "unsafe|as_bytes_mut|swap"
  │  Hints: 1 hint [Edit]
  └────────────────────────────────────────────────────────

  [+ Add Stage]
```

#### State Assertion Builder (for `method: state`)

```
── Assertions ─────────────────────── [+ Add Assertion] ──

  ≡  [file_exists      ▾]  path: [output/report.txt    ]  [✗]
  ≡  [file_contains    ▾]  path: [output/report.txt    ]
                            content: [Total: 42          ]  [✗]
  ≡  [permissions      ▾]  path: [output/report.txt    ]
                            mode: [644                   ]  [✗]
  ≡  [file_count       ▾]  path: [output                ]
                            count: [2                    ]  [✗]
```

Visual row builder. Type dropdown. Conditional fields per type. Drag to reorder.

#### Panel 3: Terminal Preview (bottom or side panel)

```
┌─ Preview ──────────────────────────── [↕ Resize] [⟳ Refresh] ──┐
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                                                            │  │
│  │  [WRITE] Declare and Print a Variable                     │  │
│  │                                                            │  │
│  │  Declare a variable named 'age' with value 25 and print   │  │
│  │  it using an f-string.                                     │  │
│  │                                                            │  │
│  │  ┌─ main.py (editable) ─────────────────────────────────┐ │  │
│  │  │ 1│ # Your code here                                  │ │  │
│  │  │  │                                                    │ │  │
│  │  └──────────────────────────────────────────────────────┘ │  │
│  │                                                            │  │
│  │  [e] Edit  [Enter] Run  [t] Test  [h] Hint  [?] Help     │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Terminal: interactive — type to test the student experience     │
└──────────────────────────────────────────────────────────────────┘
```

Implementation:
- xterm.js in the browser connecting via WebSocket to a pty
- Backend spawns `learnlocal start <course> --lesson <id>` pointed at the live course directory
- Author can interact: type code, press Enter to run, press `[t]` to test
- Refresh button restarts the preview after editor changes
- Resizable via drag handle
- For staged exercises: stage selector to preview each stage independently

### Backend API

**REST Endpoints:**

```
GET    /api/course                          → course metadata + lesson/exercise tree
PUT    /api/course                          → update course.yaml fields

GET    /api/lessons                         → list lessons with exercises
POST   /api/lessons                         → create new lesson
PUT    /api/lessons/:id                     → update lesson metadata
DELETE /api/lessons/:id                     → delete lesson
PUT    /api/lessons/reorder                 → reorder lessons [{id, position}]

GET    /api/lessons/:lid/exercises          → list exercises
POST   /api/lessons/:lid/exercises          → create new exercise
GET    /api/lessons/:lid/exercises/:eid     → get exercise details (full YAML)
PUT    /api/lessons/:lid/exercises/:eid     → update exercise
DELETE /api/lessons/:lid/exercises/:eid     → delete exercise
PUT    /api/lessons/:lid/exercises/reorder  → reorder exercises [{id, position}]

POST   /api/run-solution                    → run solution code, return stdout
POST   /api/validate                        → validate entire course
POST   /api/validate/:lid/:eid             → validate single exercise

POST   /api/publish                         → package + upload to community platform

GET    /api/toolcheck                       → check required tools (doctor-lite)
```

**WebSocket Endpoints:**

```
WS /ws/preview?lesson=:lid&exercise=:eid   → terminal preview (pty bridge)
WS /ws/validate-live                        → live validation as files change
```

### File Organization

```
src/
  author/                    # New module (behind --features author)
    mod.rs                   # Module root
    server.rs                # axum server setup, routes, static asset serving
    api.rs                   # REST endpoint handlers
    preview.rs               # WebSocket pty bridge for terminal preview
    yaml_rw.rs               # Read/write/update YAML files on disk
    solution_runner.rs       # Run solution, capture output
    publish.rs               # Package + upload to community platform

  web/                       # Frontend assets (embedded in binary)
    index.html
    app.js (or app.svelte)   # Main application
    style.css
    components/
      course-index.js        # Tree view + drag-and-drop
      exercise-editor.js     # Form-based exercise editing
      stage-builder.js       # Stage management UI
      assertion-builder.js   # State assertion visual builder
      terminal-preview.js    # xterm.js integration
      markdown-editor.js     # Prompt/explanation editing
    vendor/
      codemirror/            # Code editor
      xterm/                 # Terminal emulator
      sortable/              # Drag-and-drop
      marked/                # Markdown rendering
```

---

## Feature 3: Community Platform

### Concept

A central registry where authors publish courses and students discover them. The platform itself is a future project — this design defines the integration points from both the Designer (publish) and the TUI (browse/download).

### Course Package Format

When an author publishes, the system creates a package:

```
course-package.tar.gz
├── manifest.json            # Package metadata (see below)
├── course.yaml
├── lessons/
│   └── ... (full course directory)
└── screenshots/             # Optional: auto-generated TUI screenshots
    ├── exercise-preview.png
    └── lesson-preview.png
```

**manifest.json:**
```json
{
  "package_version": 1,
  "course_id": "python-fundamentals",
  "name": "Python Fundamentals",
  "version": "1.1.0",
  "author": "LearnLocal Community",
  "license": "CC-BY-4.0",
  "description": "Learn Python from the ground up",
  "language_id": "python",
  "language_display": "Python",
  "lesson_count": 8,
  "exercise_count": 57,
  "exercise_types": {
    "write": 49,
    "fix": 4,
    "command": 4
  },
  "has_stages": false,
  "estimated_hours": 3.5,
  "platform": null,
  "provision": "auto",
  "tags": ["beginner", "python", "fundamentals"],
  "checksum": "sha256:abc123...",
  "created_at": "2026-03-21T19:00:00Z",
  "learnlocal_min_version": "0.5.0"
}
```

### Platform API Contract (Placeholder)

The actual platform is a future project. These endpoints define the contract that the Designer (publish) and TUI (browse) will use:

```
# Registry API (community.learnlocal.dev or similar)

POST   /api/v1/courses                    # Upload course package
  Auth: Bearer token
  Body: multipart/form-data with course-package.tar.gz
  Returns: { id, url, status: "pending_review" | "published" }

GET    /api/v1/courses                    # List/search courses
  Query: ?language=python&sort=popular&page=1
  Returns: { courses: [manifest...], total, page }

GET    /api/v1/courses/:id                # Course details
  Returns: { manifest, download_url, stats: { downloads, rating } }

GET    /api/v1/courses/:id/download       # Download course package
  Returns: course-package.tar.gz

GET    /api/v1/registry.json              # Full registry index (for offline caching)
  Returns: { version, courses: [{ id, name, version, language, checksum, download_url }] }
```

### Designer: Publish Flow

```
Author clicks [📤 Publish] in Course Designer
    │
    ├── Pre-flight checks:
    │   ├── All exercises pass validation? (must be ✓)
    │   ├── All solutions run successfully? (must be ✓)
    │   ├── Course has description, author, license? (must be ✓)
    │   └── Show pre-flight report with pass/fail
    │
    ├── Package creation:
    │   ├── Generate manifest.json from course metadata
    │   ├── Create tar.gz of course directory
    │   └── Compute SHA-256 checksum
    │
    ├── Auth (first time):
    │   ├── "Sign in to LearnLocal Community"
    │   ├── OAuth flow or API key input
    │   └── Store token in ~/.config/learnlocal/auth.yaml
    │
    ├── Upload:
    │   ├── POST to platform API
    │   ├── Progress bar during upload
    │   └── Show result: "Published!" or "Submitted for review"
    │
    └── Post-publish:
        ├── Show course URL on community platform
        └── Badge in Course Index: "📤 Published v1.1.0"
```

### TUI: Browse & Download Flow

**New command:**
```
learnlocal browse                # Opens community browser in TUI
```

**New screen in TUI (`Screen::Browse`):**

```
┌─ LearnLocal | Community Courses ─────────────────────────────────┐
│                                                                   │
│  Search: [_____________]    Filter: [All Languages ▾]  [Popular ▾]│
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  Python Fundamentals                        ★★★★☆  1.2k ↓ │   │
│  │  Learn Python from the ground up                           │   │
│  │  8 lessons · 57 exercises · ~3.5 hours · by Community      │   │
│  │  [python] [beginner] [fundamentals]                        │   │
│  ├────────────────────────────────────────────────────────────┤   │
│  │  Rust Ownership Deep Dive                   ★★★★★   847 ↓ │   │
│  │  Master Rust's ownership system with staged exercises      │   │
│  │  5 lessons · 35 exercises · ~4 hours · by rustacean42      │   │
│  │  [rust] [intermediate] [ownership] [staged]                │   │
│  ├────────────────────────────────────────────────────────────┤   │
│  │  SQL for Data Scientists                    ★★★★☆   632 ↓ │   │
│  │  Practical SQL with real-world datasets                    │   │
│  │  10 lessons · 80 exercises · ~6 hours · by datadev         │   │
│  │  [sql] [intermediate] [data-science]                       │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                   │
│  [Enter] Details  [d] Download  [↑↓] Navigate  [/] Search  [Esc] │
└───────────────────────────────────────────────────────────────────┘
```

**Course detail view (Enter on a course):**

```
┌─ Python Fundamentals v1.1.0 ─────────────────────────────────────┐
│                                                                   │
│  By: LearnLocal Community                                         │
│  License: CC-BY-4.0                                               │
│  Downloads: 1,247   Rating: ★★★★☆ (4.2/5)                        │
│  Platform: any   Requires: python3                                │
│                                                                   │
│  Learn Python from the ground up. 8 lessons covering variables,  │
│  control flow, functions, data structures, and string methods.    │
│                                                                   │
│  Lessons:                                                         │
│    1. Hello World & Getting Started (7 exercises)                 │
│    2. Variables and Types (7 exercises)                            │
│    3. Operators and Expressions (7 exercises)                     │
│    4. Control Flow (8 exercises)                                  │
│    5. Functions (7 exercises)                                     │
│    6. Lists and Tuples (7 exercises)                              │
│    7. Dictionaries (7 exercises)                                  │
│    8. String Methods (7 exercises)                                │
│                                                                   │
│  [d] Download & Install  [Esc] Back                               │
└───────────────────────────────────────────────────────────────────┘
```

**Download flow:**
```
Downloading Python Fundamentals v1.1.0...
  ████████████████████░░░░  78%  (2.1 MB / 2.7 MB)

Verifying checksum... ✓
Extracting to courses/python-fundamentals... ✓
Checking tools... ✓ python3 found

✓ Installed! Start with: learnlocal start python-fundamentals
  Or find it on the Home screen.

[Enter] Go to Home  [Esc] Browse more
```

**Offline registry caching:**
- On first browse, download `registry.json` and cache locally
- Show cached results when offline with "Last updated: 2 days ago"
- Refresh on next browse when online
- Fits the offline-first philosophy — browse works without network after first sync

---

## Implementation Phases

### Phase 1: Staged Exercises — Schema & Backend

No UI changes. Backend only.

Files to modify:
- `src/course/types.rs` — Add `ExerciseStage`, `stages` field
- `src/course/validator.rs` — Stage validation rules
- `src/state/types.rs` — Add stage tracking to progress
- `src/exec/runner.rs` — Add `StageComplete` variant, stage-aware validation
- `src/state/signals.rs` — Stage tracking in session state
- `src/state/sandbox.rs` — Stage-aware draft paths

Tests: schema round-trip, validator rules, backward compatibility

### Phase 2: Staged Exercises — UI

Files to modify:
- `src/ui/screens.rs` — Add `StageComplete` to `AppState`
- `src/ui/course_app.rs` — Stage indicator, advancement, hints, celebration

### Phase 3: Author CLI — run-solution tool

Files to create/modify:
- `src/cli.rs` — Add `Author` command group
- `src/main.rs` — Add `cmd_author()` dispatcher
- `src/author/mod.rs` — New module
- `src/author/solution_runner.rs` — Run solution, capture output, update YAML

### Phase 4: Course Designer — Backend

New files (behind `--features author`):
- `src/author/server.rs` — axum server setup, static assets
- `src/author/api.rs` — REST endpoint handlers
- `src/author/preview.rs` — WebSocket pty bridge
- `src/author/yaml_rw.rs` — YAML read/write/update operations
- `Cargo.toml` — Add `axum`, `tokio`, `rust-embed`, `portable-pty` under `[features] author`

### Phase 5: Course Designer — Frontend

New files (embedded in binary):
- `src/web/` — Complete frontend application
- CodeMirror, xterm.js, SortableJS, marked.js integration
- Course index, exercise editor, stage builder, assertion builder, terminal preview

### Phase 6: Community Platform — Client Integration

Files to create/modify:
- `src/author/publish.rs` — Package creation, upload
- `src/ui/screens.rs` — Add `Screen::Browse`
- `src/ui/app.rs` — Browse screen rendering, download flow
- `src/cli.rs` — Add `Browse` command
- `src/community/` — New module for registry client, package handling

Platform API: placeholder endpoints, actual server is a separate project

### Phase 7: Integration & Polish

- Designer supports creating staged exercises (stage builder UI)
- Publish validates staged exercises
- Browse shows `[staged]` badge for courses with staged exercises
- End-to-end: create course in Designer → publish → browse in TUI → download → learn

---

## Dependencies Between Phases

```
Phase 1 (Staged Schema)  ──→  Phase 2 (Staged UI)
                          ──→  Phase 7 (Designer creates stages)

Phase 3 (run-solution)   ──→  Phase 4 (Designer backend uses it)

Phase 4 (Designer BE)    ──→  Phase 5 (Designer FE)
                          ──→  Phase 6 (Publish from Designer)

Phase 6 (Community)      ──→  Phase 7 (Browse in TUI)
```

Phase 1 and Phase 3 are independent — can build in parallel.
Phase 4 and Phase 2 are independent — can build in parallel.

---

## Key Design Decisions

1. **Staged exercises are backward compatible** — `#[serde(default)]` everywhere. Zero impact on existing courses.

2. **Code carries forward between stages** — The defining mechanic. Drafts persist; only full exercise completion clears them.

3. **Designer is web-based, not TUI** — Rich interaction (drag-and-drop, CodeMirror, visual builders) requires a browser. TUI is for students, web UI is for authors.

4. **Preview shows the REAL TUI** — xterm.js + pty, not a web recreation. The preview IS the app. No divergence possible.

5. **Feature-gated** — `--features author` adds axum, tokio, web assets. Core binary stays lean and dependency-free.

6. **Auto-save to disk** — The Designer writes directly to the course directory. No intermediate format. The YAML on disk is always the source of truth.

7. **Community platform is API-first** — Client integration designed now, server built later. The API contract is stable; the implementation can change.

8. **No new dependencies for staged exercises** — Schema change only. Existing serde_yaml, existing runner, existing validator.

9. **Offline-first browse** — Registry cached locally. Works without network after first sync.

---

## QoL Polish Items (from earlier analysis, to bundle opportunistically)

These refined polish items can be implemented alongside the main features:

| ID | Feature | Approach |
|----|---------|----------|
| B1 | Auto-indent on Enter | Copy previous line's leading whitespace |
| B5 | Editor status line | Show file count only when multi-file |
| D3 | Lesson position | "Lesson 3 of 8 (Exercise 4/7)" breadcrumb |
| D4 | Nav at boundaries | Hide nav hint when at boundary (or wrap-around) |
| G1 | Shell output clarity | Bold/color the `$ ` prompt line |
| H2 | Key bar | Curate 3-4 keys per state, always end with `[?]` |
| C2 | Watch mode | `Last run: Xs ago` timestamp |

---

## Validation Checklist

Before marking sprint complete:

### Staged Exercises
- [ ] Staged exercise YAML deserializes and round-trips correctly
- [ ] Non-staged exercises work exactly as before (backward compat)
- [ ] Old progress.json files without stage fields load without error
- [ ] Stage advancement preserves student code
- [ ] Stage-specific hints display correctly
- [ ] Stage completion tracked in progress
- [ ] Exercise completion requires all stages
- [ ] `learnlocal validate` checks stage constraints
- [ ] `learnlocal validate --run-solutions` tests each stage's solution

### Course Designer
- [ ] `learnlocal author design <path>` starts server and opens browser
- [ ] Course index shows lesson/exercise tree
- [ ] Drag-and-drop reorder updates files on disk
- [ ] Exercise editor saves valid YAML
- [ ] Solution runner captures output correctly
- [ ] Terminal preview shows real TUI via xterm.js
- [ ] Stage builder creates valid staged exercises
- [ ] State assertion builder generates correct YAML
- [ ] New exercises added to lesson.yaml automatically

### Community Platform
- [ ] Course packaging creates valid tar.gz with manifest
- [ ] Publish flow validates before uploading
- [ ] Browse screen fetches and displays registry
- [ ] Download extracts to courses directory
- [ ] Offline registry cache works

### Code Quality
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo clippy --all-targets --features llm -- -D warnings` passes
- [ ] `cargo clippy --all-targets --features author -- -D warnings` passes
- [ ] All existing tests pass (265 without llm, 281 with llm)
- [ ] New unit tests for all new modules

---

## File Reference

### Existing files to modify:
- `src/cli.rs` — Add Author and Browse commands
- `src/main.rs` — Command dispatch
- `src/course/types.rs` — ExerciseStage struct, stages field
- `src/course/validator.rs` — Stage validation rules
- `src/state/types.rs` — Stage tracking in progress
- `src/state/signals.rs` — Stage tracking in session
- `src/state/sandbox.rs` — Stage-aware draft paths
- `src/exec/runner.rs` — StageComplete result variant
- `src/ui/screens.rs` — StageComplete state, Browse screen, Author screen
- `src/ui/app.rs` — Browse screen rendering, author entry point
- `src/ui/course_app.rs` — Stage indicator, advancement, celebration
- `Cargo.toml` — New feature flags and dependencies

### New files to create:
- `src/author/mod.rs` — Author module root
- `src/author/server.rs` — axum server, routing, static assets
- `src/author/api.rs` — REST API handlers
- `src/author/preview.rs` — WebSocket terminal preview
- `src/author/yaml_rw.rs` — YAML file operations
- `src/author/solution_runner.rs` — Run + capture solution output
- `src/author/publish.rs` — Package + upload to community
- `src/community/mod.rs` — Registry client, download, cache
- `src/web/` — Frontend application (embedded in binary)
