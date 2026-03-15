# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-03-15

### Fixed

- **Unicode crash in inline editor and shell input** — cursor tracking now uses char indices instead of byte offsets, preventing panics on multi-byte characters (accented chars, CJK, emoji)
- **Timer thread killing wrong PID after exercise completes** — timeout threads now check a cancellation flag instead of sleeping the full duration and blindly sending kill signals
- **Unicode panic in assertion description truncation** — byte slicing replaced with char-aware truncation
- **Panic on `current_exercise().unwrap()` in execution paths** — replaced with proper error propagation for malformed courses or corrupted state
- **Path traversal vulnerability in sandbox `write_file`** — resolved paths are now verified to stay within the sandbox directory
- **Silent progress save failures** — disk-full and permission errors are now logged instead of silently discarded
- **Shell mode entry failure silently ignored** — errors now log and fall back to exercise prompt state
- **LLM thread leak on course re-entry** — old LLM thread is shut down before spawning a new one when switching courses
- **Ctrl+C and SIGTERM not saving editor drafts** — both exit paths now call `save_draft_to_disk()` before quitting
- **Content line count overflow** — `u16` cast now saturates at `u16::MAX` instead of silently wrapping
- **End key over-scrolling past content** — scroll offset now accounts for viewport height
- **CSV export lacking field escaping** — fields containing commas, quotes, or newlines are now properly quoted
- **`find_lesson_dir` matching non-numeric prefixes** — prefix is now validated as digits before matching
- **`collect_env_commands` missing flat exercise files** — falls back to `find_exercise_file()` for `NN-id.yaml` format

### Changed

- Updated README with problem/solution framing, numbered quick start, prerequisites table, keyboard shortcuts, troubleshooting guide, and course authoring section
- Applied `cargo fmt` to fix formatting diffs caught by CI lint (loader.rs, sandbox.rs, course_app.rs, inline_editor.rs)
- Added explicit `permissions` block to security audit CI job to fix Dependabot PR check failures
- Added pre-push checklist to CLAUDE.md (fmt, clippy) to prevent future CI lint failures

## [0.1.0] - 2026-03-14

### Added

- **Core runtime** — step-based execution engine supporting any language via YAML-configured build/run steps
- **TUI** — ratatui-based terminal interface with markdown-rendered lessons, inline code editor, and progress tracking
- **Shell mode** — interactive terminal for command-line exercises (Linux, Git courses)
- **10 courses, 500+ exercises** — C++, Python, JavaScript, Rust, Go, AI, Linux, SQL, Git, Production Incidents
- **Sandboxed execution** — timeout + tmpdir (basic), firejail, bubblewrap (tiered)
- **Built-in SQLite** — embedded database for SQL courses, no external setup needed
- **Optional LLM integration** — Ollama backend for AI hints and chat, feature-gated behind `--features llm`
- **Progress persistence** — JSON-based progress tracking keyed by course ID and major version
- **Environment engine** — filesystem setup, background services, dynamic ports, state assertions, teardown
- **Course validation** — `learnlocal validate` with structural checks and solution execution
- **CLI commands** — `list`, `start`, `progress`, `reset`, `validate`, `doctor`, `init`, `export`, `completions`, `man`
- **Shell completions** — bash, zsh, fish, PowerShell via `clap_complete`
- **Man page generation** — via `clap_mangen`
- **Signal handling** — clean terminal restoration on SIGTERM/SIGINT/SIGHUP
- **Structured logging** — `log` + `env_logger` behind `--verbose`
- **Config warnings** — yellow message on malformed YAML instead of silent fallback
- **Progress backup** — `progress.json.bak` created before destructive reset
- **Crash reports** — `learnlocal-crash.log` with system info and backtrace on panic
- **High-contrast theme** — `ThemePreset` enum with 10 semantic color fields, switchable in Settings
- **Terminal size guard** — graceful error if terminal < 80x24
- **Mouse click support** — home, settings, and progress screens via `LayoutCache`
- **Responsive layout** — single-panel mode < 100 cols, capped panel widths >= 160 cols
- **Welcome tour** — 9-slide interactive introduction to the TUI
- **CI/CD** — GitHub Actions for tests, clippy, fmt, MSRV, course validation, release builds, dependabot, cargo audit
- **Differentiated exit codes** — 0 success, 1 error, 2 validation fail, 3 missing tool

[0.1.1]: https://github.com/thehighnotes/learnlocal/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/thehighnotes/learnlocal/releases/tag/v0.1.0
