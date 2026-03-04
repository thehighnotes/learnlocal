# Sprint 1 — Foundation (Legal + Repo Hygiene)

## Items: #1, #2, #3, #9, #10, #11, #12, #13

## Decisions

### License Strategy (Dual)
- **Code:** Dual MIT + Apache-2.0
  - `LICENSE-MIT` — MIT license text, copyright "Mark Wind"
  - `LICENSE-APACHE` — Apache 2.0 license text, copyright "Mark Wind"
  - Cargo.toml: `license = "MIT OR Apache-2.0"`
- **Course content:** CC-BY-4.0
  - `LICENSE-COURSES` — CC-BY-4.0 full text
  - Courses already declare `license: CC-BY-4.0` in course.yaml — this formalizes it at repo level

### Cargo.toml Metadata
- `authors = ["Mark Wind"]`
- `license = "MIT OR Apache-2.0"`
- `repository = "https://github.com/thehighnotes/learnlocal"`
- `homepage = "https://github.com/thehighnotes/learnlocal"`
- `keywords = ["tutorial", "education", "terminal", "offline", "programming"]`
- `categories = ["command-line-utilities", "development-tools"]`
- `rust-version = "<TBD — test actual MSRV>"`
- `readme = "README.md"`

### .gitignore Expansion
Add standard entries:
```
# Build
/target

# Editor
*.swp
*.swo
*~
.vscode/
.idea/
*.iml

# OS
.DS_Store
Thumbs.db

# Runtime
nohup.out
*.log

# Rust
**/*.rs.bk
```

### File Operations
- `git rm nohup.out`
- Delete: `FEATURE_IDEAS.md`, `FUTURE_PLANS.md`, `FUTURE_COURSES.md`, `ENV_ENGINE_EVOLUTION.md`
- Delete: `learnlocal-spec.md` (v1, if exists)
- Move: `learnlocal-spec-v2.md` → `docs/SPECIFICATION.md`
- Update any code references to the spec path if they exist

### Cargo.lock
- Run `cargo update --dry-run` to check for stale entries
- No action unless something is obviously wrong
