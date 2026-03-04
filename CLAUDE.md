# LearnLocal — Project Instructions

## What This Is

Offline terminal-based programming tutorial framework. Like `vimtutor` for any language.
Spec: `docs/SPECIFICATION.md`.

## Architecture Decisions (Non-Negotiable)

- **Rust** for the runtime. Course authors write YAML + Markdown, not Rust.
- **$EDITOR-first** for code editing. Inline TUI editor as convenience (`[e]`), external $EDITOR via `[E]` (Shift+e).
- **LLM is feature-gated.** Behind `--features llm` at compile time, `config.llm.enabled` at runtime. Core binary has zero async/HTTP dependencies without this feature.
- **State layer stores raw signals, not interpretations.** Record attempt counts, time spent, hints revealed, compile/run results. Never compute "struggling" or "mastery" in the core — that's the LLM's job.
- **Execution uses a step list**, not a fixed compile/run pair. See `course.yaml` `language.steps` in the spec.
- **Sandboxing is tiered:** basic (timeout + tmpdir) always; firejail/bwrap if detected.

## Project Structure

```
src/
  main.rs           # Entry point, CLI parsing
  course/           # YAML loading, course/lesson/exercise types, validation
  exec/             # Step-based execution engine, sandbox, environment engine, tool checking
  ui/               # ratatui TUI, markdown renderer, $EDITOR integration
  state/            # Progress (persisted JSON) + session signals (in-memory)
  llm/              # Optional: context view, backends (behind feature flag)
courses/            # Course packs (YAML + MD)
```

## Conventions

- Course format schemas are defined in the spec (Section 3). Don't deviate without updating the spec.
- Progress file lives at `~/.local/share/learnlocal/progress.json`. Config at `~/.config/learnlocal/config.yaml`.
- Respect `NO_COLOR` env var.
- All execution timeouts default to 10s. Course can override via `limits.timeout_seconds`.
- Multi-file exercises use `files:` array. Single-file use `starter:`. Never both.
- Exercise `input:` field is piped to stdin. Omit if exercise doesn't need stdin.
- `environment:` block in exercise YAML defines filesystem setup, setup/teardown commands, background services, env vars, dynamic ports, and state assertions.
- Environment engine lifecycle: sandbox → filesystem setup → setup commands → start services → student files → language steps → teardown → kill services → validate.
- Student code gets loopback networking automatically when exercise defines services.
- `{dir}` placeholder is substituted in env file content and service args. Teardown gets full `{dir}/{main}/{output}/{files}` substitution.
- Service readiness supports `ready_stream: stdout|stderr|both` (default: "both"). Services can capture stdout/stderr to sandbox files via `capture_stdout`/`capture_stderr`.
- Dynamic ports: `ports: N` allocates ephemeral ports, injected as `LEARNLOCAL_PORT_0..N` env vars.
- `course.yaml` can set `platform: linux|macos|windows` to restrict which OS can run the course.

## Testing

- `learnlocal validate <course-dir>` must work in Phase 1. Course authors depend on it.
- Every exercise solution in `courses/` must pass its own validation. Integration tests verify this.
- Unit tests for: course loading, placeholder substitution, output validation, progress serialization, semver progress keying.
- Unit tests for: environment setup, port allocation, service readiness (stdout/stderr/both/timeout), placeholder substitution in env content, validator checks for env fields.

## Common Gotchas

- The `{dir}`, `{main}`, `{output}`, `{files}` placeholders in execution steps need consistent substitution. `{main}` is the first `editable: true` file (or `main_file:` override).
- Java exercises need the class name in the run step args — this is a course-author concern, not a runtime concern. The runtime just substitutes placeholders.
- Progress is keyed to `{course_id}@{major_version}`. Don't key to full semver or progress breaks on patch updates.
- The LLM context view (Section 8.2 of spec) is a read-only assembly from state. It never writes back. One-way dependency.
- Environment setup/teardown commands use `{dir}` only for setup (student files don't exist yet), but full placeholders for teardown.
- `ready_stream` defaults to "both" — most real servers log to stderr, not stdout.
- Teardown failures are warnings (yellow text), not errors — they don't block the student's result.
- Service `capture_stdout`/`capture_stderr` paths must be sandbox-relative (no absolute, no `..`).

## Phases

Building in phases:
1. **Phase 1 (MVP):** Course loading, validation, execution, progress, TUI, $EDITOR, one C++ course -- **DONE**
2. **Phase 2:** UX polish, colored diffs, inline editor, run/submit separation, diagnostics -- **DONE**
3. **Phase 3:** LLM integration (Ollama, streaming, AI chat, context assembly) -- **DONE**
4. **Phase 4:** More courses (Python, JS, AI, Linux Fundamentals), progressive reveal, AI chat during lesson reading, environment engine v3, platform blocking -- **IN PROGRESS**
5. **Phase 5:** Distribution, packaging, community tooling

## Public Release Workflow

Work through the checklist in **Sprints**. Each sprint bundles related items for one session.

**Process per sprint:**
1. Claude proposes which items to bundle
2. High-level brainstorm together on design decisions
3. Document agreed design in `docs/design/sprint-N-<topic>.md`
4. Enter plan mode → full implementation plan
5. Execute, verify, mark items `[x]`

**Sprint Map:**
- **Sprint 1 — Foundation:** #1-3, #9-13 (legal + repo hygiene — unblocks everything)
- **Sprint 2 — Front Door:** #4, #6-8, #27-28 (README, badges, Cargo.toml release profile, toolchain pin)
- **Sprint 3 — CI/CD:** #14-20 (GitHub Actions: tests, clippy, fmt, MSRV, audit, dependabot)
- **Sprint 4 — CLI Polish:** #29-37 (completions, --verbose, doctor, init, export, exit codes, man pages)
- **Sprint 5 — Robustness:** #43-47, #30→44 tie-in (signal handling, logging, config warnings, crash reports, progress backup)
- **Sprint 6 — Distribution:** #18, #21-26 (release builds, binaries, crates.io, install script, package managers)
- **Sprint 7 — TUI & Accessibility:** #38-42 (high-contrast, terminal guard, mouse, responsive, theme config)
- **Sprint 8 — Testing & Quality:** #48-53 (integration tests, fuzzing, benchmarks, snapshots, Go validation, Windows CI)
- **Sprint 9 — Documentation:** #54-57 (course authoring guide, ARCHITECTURE link, rustdoc, FAQ)
- **Sprint 10 — Community:** #58-65 (CONTRIBUTING, COC, SECURITY, CHANGELOG, issue/PR templates, discussions)
- **Sprint 11 — Content:** #5, #66-69 (asciinema demo, quality bar docs, course template, catalog, --courses docs)
- **Sprint 12 — Future-Proofing:** #70-75 (plugins, update checker, a11y statement, i18n prep, analytics)

**Current sprint:** Sprint 6 — Distribution

## Public Release Checklist

### Legal & Identity
- [x] 1. LICENSE file (code license — MIT OR Apache-2.0, dual-licensed)
- [x] 2. Cargo.toml metadata: `authors`, `license`, `repository`, `homepage`, `keywords`, `categories`
- [x] 3. Course content license clarification (LICENSE-COURSES: CC-BY-4.0 for courses/ directory)

### First Impressions
- [x] 4. README.md — pitch, screenshots (ui1.png/ui2.png exist), install instructions, feature list, course catalog
- [ ] 5. Asciinema/GIF demo recording of completing an exercise
- [x] 6. Badge row — license badge (CI/crate/MSRV badges deferred to Sprint 3/6)
- [x] 7. "Why this exists" positioning — offline, zero-config, no browser, no cloud, no sign-up
- [x] 8. Comparison table vs Exercism, Codecademy, Rustlings, vimtutor

### Repo Hygiene
- [x] 9. `.gitignore` expansion — `*.swp`, `*.swo`, `.DS_Store`, `.vscode/`, `.idea/`, `nohup.out`, `*.log`
- [x] 10. Remove committed `nohup.out`
- [x] 11. Removed internal docs from root — `FEATURE_IDEAS.md`, `FUTURE_PLANS.md`, `FUTURE_COURSES.md`, `ENV_ENGINE_EVOLUTION.md`, `learnlocal-spec.md` (v1)
- [x] 12. Moved `learnlocal-spec-v2.md` → `docs/SPECIFICATION.md`
- [x] 13. Verify `Cargo.lock` has no stale entries

### CI/CD & Automation
- [x] 14. GitHub Actions: test matrix — `cargo test` + `cargo test --features llm` on Linux/macOS (x86_64, aarch64)
- [x] 15. GitHub Actions: course validation — `learnlocal validate` on all shipped courses
- [x] 16. GitHub Actions: `cargo clippy` + `cargo fmt --check`
- [x] 17. GitHub Actions: MSRV check
- [x] 18. GitHub Actions: release builds — cross-compile binaries on tag push
- [x] 19. Dependabot or Renovate for dependency update PRs
- [x] 20. `cargo audit` in CI for known CVEs

### Distribution & Installation
- [ ] 21. Pre-built binaries on GitHub Releases (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
- [ ] 22. `cargo install learnlocal` — publish to crates.io
- [ ] 23. Install script — `curl -sSf ... | sh` one-liner
- [ ] 24. Homebrew formula
- [ ] 25. AUR package
- [ ] 26. Nix flake
- [x] 27. `[profile.release] strip = true` in Cargo.toml to reduce binary size
- [x] 28. `rust-toolchain.toml` — pin toolchain for contributors

### CLI Polish
- [x] 29. Shell completions via `clap_complete` (bash, zsh, fish, PowerShell)
- [x] 30. `--verbose` / `--debug` flag for troubleshooting exercise failures
- [x] 31. Usage examples in `--help` subcommands
- [x] 32. `learnlocal doctor` command — check $EDITOR, sandbox tools, Ollama, terminal size, disk space
- [x] 33. `learnlocal init <course-name>` — scaffold course directory for authors
- [x] 34. `learnlocal export` — export progress to shareable format
- [x] 35. Colored `validate` output (green/red/yellow)
- [x] 36. Differentiated exit codes (0=success, 1=error, 2=validation fail, 3=missing tool)
- [x] 37. Man page generation via `clap_mangen`

### TUI & Accessibility
- [ ] 38. High-contrast theme option
- [ ] 39. Terminal size guard — graceful message if < 80x24
- [ ] 40. Optional mouse support in menus
- [ ] 41. Responsive layout testing at 80x24, 120x40, 200x60
- [ ] 42. Theme customization in `config.yaml`

### Robustness & Safety
- [x] 43. Signal handling via `signal-hook` for SIGTERM/SIGINT/SIGHUP — clean terminal restoration
- [x] 44. Structured logging — `log` + `env_logger` behind `--verbose`
- [x] 45. Config parse warning — show yellow message on malformed YAML instead of silent fallback
- [x] 46. Progress backup — `progress.json.bak` before destructive reset (CLI + TUI)
- [x] 47. Crash report — write `learnlocal-crash.log` with system info + backtrace on panic

### Testing & Quality
- [ ] 48. Integration tests — end-to-end: load course → run exercise → verify progress written
- [ ] 49. Fuzz targets via `cargo-fuzz` — YAML parsing, placeholder substitution, output validation
- [ ] 50. Benchmark suite via `criterion` — course loading, validation, TUI frame rate
- [ ] 51. TUI snapshot tests via `insta` crate
- [ ] 52. Validate Go course (install Go in CI or remove from default set)
- [ ] 53. Windows CI — verify core compiles and tests pass

### Documentation
- [ ] 54. Course authoring guide — standalone "How to create a LearnLocal course" with examples
- [ ] 55. Link `ARCHITECTURE.md` from CONTRIBUTING.md for contributors
- [ ] 56. Rustdoc — `cargo doc` with module-level docs
- [ ] 57. FAQ — offline? own editor? add a course? LLM required?

### Community & Governance
- [ ] 58. CONTRIBUTING.md — courses vs code, PR guidelines, coding conventions, test requirements
- [ ] 59. CODE_OF_CONDUCT.md (Contributor Covenant)
- [ ] 60. SECURITY.md — disclosure process, sandboxing guarantees/limitations
- [ ] 61. CHANGELOG.md — keep-a-changelog format from v0.1.0
- [ ] 62. GitHub issue templates — bug report (with system info), feature request, course request
- [ ] 63. GitHub PR template — checklist: tests pass, clippy clean, validation passes
- [ ] 64. Enable GitHub Discussions for community course sharing
- [ ] 65. "Good first issue" labels for new contributors

### Content & Courses
- [ ] 66. Course quality bar documentation — what makes a course shippable
- [ ] 67. Starter course template in `courses/template/`
- [ ] 68. Machine-readable course index/catalog
- [ ] 69. Document `--courses` flag for third-party course directories

### Future-Proofing (Post-Launch)
- [ ] 70. Plugin/extension system for custom validators and step types
- [ ] 71. `learnlocal --check-updates` via GitHub API
- [ ] 72. Accessibility statement — document current a11y features and roadmap
- [ ] 73. i18n preparation — extract UI strings to central location
- [ ] 74. Self-update capability
- [ ] 75. Anonymous usage analytics opt-in (document absence for now)
