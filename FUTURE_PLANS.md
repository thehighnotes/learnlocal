# LearnLocal — Future Plans

Living document of planned changes. Ordered by dependency, not priority.

---

## 1. Editor Simplification

### Phase 1: Inline-First Editor (DONE)

The inline editor was refactored from a full-screen takeover (`AppState::InlineEditing`) to an in-place code box editor (Option D layout). Pressing `[e]` now makes the code box in the exercise prompt become interactive without losing context.

**What was done:**
- Removed `AppState::InlineEditing` variant entirely
- Added `editing: bool` flag to CourseApp — edit mode is a flag, not a state
- Code box renders editor lines with block cursor when `editing == true`
- Yellow border + `[editing]` label + `Ln N, Col N` status bar in bottom border
- Exercise title, prompt, type badge, hints all stay visible while editing
- Key routing: `[e]`/`Esc` exits edit mode, `Ctrl+S` saves, `Ctrl+Enter` save+run, `Ctrl+T` save+test
- Global scroll keys suppressed during editing (arrow keys go to editor)
- Auto-scroll keeps cursor visible via deferred `scroll_to_cursor` pattern
- Old `render()` method on InlineEditorState removed, `split_at_cursor()` made pub
- Sandbox editing follows same pattern

**Visual before (old):** `[e]` replaced entire exercise area with editor — lost all context.
**Visual after (new):**
```
┌─ Exercise: Variables ──────────────────────────────────┐
│  [WRITE]                                               │
│  Declare two variables and print them.                 │
│                                                        │
│  ┌─ main.go [editing] ─────────────────────────────┐   │
│  │  1  package main                                │   │
│  │  2  import "fmt"                                │   │
│  │  3  func main() {                              │   │
│  │  4      █                                       │   │
│  │  5  }                                           │   │
│  │ Ln 4, Col 7 ────────────────────────────────────│   │
│  └─────────────────────────────────────────────────┘   │
│                                                        │
│  Hints:                                                │
│  1. Use var or :=                                      │
└────────────────────────────────────────────────────────┘
```

### Phase 2: External Editor Cleanup (TODO)

Remove the external editor path (`[E]`) and `EditorType` enum. The inline editor is now the primary editing experience. Watch mode `[w]` stays for the external-editor + auto-rebuild workflow.

**Remove:**
- `EditorType` enum (auto/terminal/gui) from config
- `[E]` (shift-e) keybinding — `[e]` is the only edit key
- Editor type detection heuristics in `editor_detect.rs` — simplify to just "does this spawn a GUI window?" for watch mode
- `AppState::Editing` variant (external editor blocking state)

**Keep:**
- Watch mode `[w]` (uses external editor, orthogonal feature)
- GUI-vs-terminal detection (needed for watch mode to decide: suspend TUI or keep running)

**Add (optional, lower priority):**
- First-run editor picker for watch mode: scan PATH for known editors
- Editor args template for watch mode: `{file}`, `{line}` placeholders
- Per-editor defaults: `nvim → "+{line} {file}"`, `code → "--wait --goto {file}:{line}"`

### Files affected (Phase 2)
- `src/config.rs` — remove `EditorType`, optionally add `editor_args: Option<String>`
- `src/ui/editor_detect.rs` — simplify to GUI detection only
- `src/ui/course_app.rs` — remove `[E]` handler and `AppState::Editing`
- `src/ui/app.rs` — remove EditorType from settings
- `src/ui/screens.rs` — remove `EditorType` from `SettingsField` enum

---

## 2. Expanded Settings Menu

Current: 3 general + 3 AI = 6 settings. Target: 33 settings across 7 sections.

Settings screen gets section headers, scroll support, and grouped navigation.

### DISPLAY

| Setting | Type | Default | Notes |
|---|---|---|---|
| Theme | `< auto > dark > light >` | auto | Override terminal detection |
| Celebration style | `< full > brief > off >` | full | ASCII art celebrations |
| Show line numbers | `< on > off >` | on | In exercise prompt code display |
| Markdown width | `< narrow(60) > medium(80) > wide(100) >` | medium | Lesson content rendering |
| Diff style | `< side-by-side > inline > unified >` | inline | Output mismatch display |
| Color intensity | `< standard > muted > high-contrast >` | standard | Accessibility |

### EDITOR

| Setting | Type | Default | Notes |
|---|---|---|---|
| Editor (watch mode) | text / picker | `(detected)` | External editor for watch mode only |
| Editor args | text | `(auto per editor)` | `{file}`, `{line}` placeholders for watch mode |
| Auto-edit on start | `< off > on >` | off | Enter inline edit mode immediately on exercise load |
| Confirm reset | `< yes > no >` | yes | `[r]` reset confirmation |

### EXECUTION

| Setting | Type | Default | Notes |
|---|---|---|---|
| Timeout | `< 10s > 15s > 30s > 60s >` | 10s | Override course default |
| Max output lines | `< 50 > 100 > 200 > unlimited >` | 100 | Truncation for chatty programs |
| Sandbox level | `< auto > basic > contained >` | auto | Existing |
| Provisioning | `< auto > system-only >` | auto | Disable portable download prompts |
| Show compiler warnings | `< yes > no >` | yes | Filter warnings from output |
| Clear between runs | `< yes > no >` | yes | Keep previous output visible |

### WATCH MODE

| Setting | Type | Default | Notes |
|---|---|---|---|
| Debounce delay | `< 300ms > 500ms > 1s > 2s >` | 500ms | Vim tmpfiles vs VS Code batch saves |
| Auto-submit on pass | `< off > on >` | off | Submit automatically when output matches |
| Notify on result | `< off > bell > flash >` | off | Terminal bell or screen flash |

### HINTS & LEARNING

| Setting | Type | Default | Notes |
|---|---|---|---|
| Auto-reveal hints | `< off > after 3 > after 5 > after 10 >` | off | Show first hint after N failures |
| Show hint count | `< yes > no >` | yes | "3 hints available" badge |
| Show exercise type | `< yes > no >` | yes | [WRITE]/[FIX]/[PREDICT] badge |
| Confirm skip | `< yes > no >` | yes | `[s]` skip confirmation |
| Skip marker | `< yes > no >` | yes | Show "skipped" in progress view |

### PROGRESS & DATA

| Setting | Type | Default | Notes |
|---|---|---|---|
| Data directory | text | `~/.local/share/learnlocal/` | Portable installs, USB drives |
| Draft persistence | `< on > off >` | on | Save in-progress code between sessions |
| Course directories | text | `(bundled)` | Additional dirs to scan for courses |
| Toolchain cache | text | `(inside data dir)` | Where portable downloads live |

### AI (feature-gated)

| Setting | Type | Default | Notes |
|---|---|---|---|
| AI enabled | toggle | off | Existing |
| Ollama URL | text | `http://localhost:11434` | Existing |
| Model | picker | `qwen3:4b` | Existing |
| Context depth | `< minimal > standard > full >` | standard | Token budget for code context |
| Max response length | `< short(128) > medium(256) > long(512) >` | medium | Cap LLM verbosity |
| Auto-suggest | `< off > after 5 > after 10 >` | off | Proactive AI help after failures |
| System prompt style | `< tutor > concise > socratic >` | tutor | LLM personality |

### Implementation notes
- Config struct grows with `#[serde(default)]` on every new field — backwards compatible
- Settings screen needs vertical scrolling (33 items won't fit on one screen)
- Section headers in TUI: DISPLAY, EDITOR, EXECUTION, WATCH MODE, HINTS & LEARNING, PROGRESS & DATA, AI
- Toggle fields use `[Left/Right]` to cycle, text fields use `[Enter]` to edit
- All settings saved to `~/.config/learnlocal/config.yaml`

---

## 3. Portable Provisioning Completion

See `memory/provisioning.md` for detailed status. Summary of remaining work:

### 3a. Real SHA256 hashes
- Download each archive from registry URLs
- Compute `sha256sum`, replace placeholder strings in `src/exec/registry.rs`
- 8 entries (Python/Node/Go × linux-x86_64/linux-aarch64/darwin-x86_64/darwin-aarch64)

### 3b. TUI download modal
- New `AppState` variant or modal overlay in `src/ui/app.rs`
- Triggers between course selection and CourseApp launch
- Shows: language, version, download size
- Keys: `[Y]` download, `[n]` cancel, `[i]` show manual install instructions
- Progress indicator during download (curl outputs to stderr)
- On success: cache toolchain, enter course normally
- On cancel: return to home screen

### 3c. Validator behavior
- Option A: validator triggers download automatically (best for CI)
- Option B: validator skips execution for auto-provision courses when tool missing (reports "skipped, tool not found")
- Option C: require tool installed for validation (current behavior, simplest)
- Decision deferred — depends on how the tool is distributed

### 3d. Go course validation
- Blocked on either Go installed on system or download flow working
- 55 exercises written, YAML structure validates, solutions hand-traced by agents
- Needs execution validation once Go is available

---

## 4. Courses Still Possible

Beyond the 9 courses built (C++, Python, JS, AI, Linux, SQL, Rust, Go, env-engine-test):

| Course | Provision | Notes |
|---|---|---|
| TypeScript Fundamentals | auto (node) | Needs ts-node or tsc+node two-step |
| Git Fundamentals | system | Uses git CLI, environment engine for repo setup |
| Docker Fundamentals | system, platform: linux | Environment engine, needs Docker installed |
| Bash Scripting | system, platform: linux/macos | Shell scripts, environment engine for filesystem setup |
| HTML/CSS/JS (Web) | embedded? | Could use embedded web preview, or just file output validation |
| Data Structures (Python) | auto (python) | Algorithm-focused, builds on Python Fundamentals |
| Regex | embedded | Could validate in-process with Rust's regex crate |

---

## 5. Distribution (Phase 5 from spec)

Not started. Options:

- **Single binary**: `cargo build --release` (~15MB with rusqlite bundled)
- **Package managers**: Homebrew formula, AUR package, snap/flatpak
- **Courses as separate downloads**: `learnlocal install <course-url>` fetching course packs
- **Course authoring tools**: `learnlocal new-course`, `learnlocal new-lesson`, `learnlocal new-exercise` scaffolding commands

---

## Dependency Chain

```
Editor Phase 1 (inline-first) ─────────→ DONE
Editor Phase 2 (remove external) ──────→ can ship independently
Expanded Settings ─────────────────────→ depends on editor phase 2 (removes EditorType)
Provisioning Completion ───────────────→ unblocks Go validation + auto-download UX
  └→ TUI download modal ──────────────→ depends on expanded settings (Provisioning toggle)
Distribution ──────────────────────────→ depends on provisioning (portable binary story)
New Courses ───────────────────────────→ independent, can ship anytime
```
