# LearnLocal — Project Instructions

## What This Is

Offline terminal-based programming tutorial framework. Like `vimtutor` for any language.
Spec: `learnlocal-spec-v2.md` (v1 is archived, don't reference it for decisions).

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
