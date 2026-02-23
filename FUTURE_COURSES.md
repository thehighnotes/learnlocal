# Future Course Considerations

Beyond traditional programming languages, LearnLocal's "edit text → run command → validate output" loop supports any skill where a CLI tool can grade text-based input.

## Programming Languages (Planned)

- Go Fundamentals
- Rust Fundamentals
- SQL (SQLite) Fundamentals

**Already shipped:**
- C++ Fundamentals v2.0.0 (8 lessons, 55 exercises)
- Python Fundamentals v1.0.0 (8 lessons, 54 exercises)
- JS (Node.js) Fundamentals v1.0.0 (8 lessons, 56 exercises)
- AI Fundamentals (Python) v1.0.0 (8 lessons, 56 exercises) — pure stdlib, no pip
- Linux Fundamentals v1.0.0 (8 lessons, 55 exercises) — platform: linux

## Query & Data Languages

- **jq** — JSON processing from the command line (single binary, increasingly popular)
- **GraphQL** — schema + query writing (validated with a linter/runner)
- **XPath / CSS Selectors** — querying structured documents

## Text Processing & Pattern Matching

- **Regular Expressions** — write patterns, validate matches (python or grep as runner, huge audience)
- **sed & awk** — stream editing, text transformation pipelines
- **Text processing pipelines** — combining grep, sort, cut, tr, uniq, etc.

## DevOps & Configuration

- **Git** — validate repo state after running commands
- **Docker** — Dockerfiles validated by `docker build` or a linter
- **Makefiles / Build systems** — write build rules, validate with `make`
- **systemd units / nginx configs** — validated with `nginx -t`, `systemd-analyze verify`
- **Terraform / HCL** — validated with `terraform validate`

## Markup & Document Languages

- **HTML/CSS** — validated with a parser or headless tool
- **LaTeX** — compile to PDF, validate structure
- **Typst** — modern LaTeX alternative, compiles fast
- **Mermaid / Graphviz DOT** — diagram-as-code, validated by the compiler

## Music & Creative

- **LilyPond** — music notation, compiles to sheet music
- **ABC notation** — simpler music notation format
- **POV-Ray** — text-based 3D scene description

## Data Formats

- **YAML** — write valid YAML, validated by a parser
- **TOML** — same idea
- **JSON Schema** — write schemas, validate against test data
- **CSV wrangling** — with csvkit or similar

## Strongest Non-Language Candidates

Ranked by framework fit (low tool requirements, clear validation, broad audience):

1. **Regex Fundamentals** — python or grep as runner, no install beyond python
2. **Git Fundamentals** — validate repo state, universally needed
3. **sed & awk** — output matching is a natural fit for the framework (Linux Fundamentals already covers basic text processing — sed, awk, grep, sort, cut, etc. — but a dedicated deep-dive course could go further)
4. **jq** — single binary install, deterministic JSON output
5. **HTML/CSS Fundamentals** — validated with a parser, massive audience
6. **Makefiles** — pre-installed on most systems, underserved in tutorials

---

## Self-Contained Toolchain Provisioning

Instead of requiring students to pre-install tools, LearnLocal could provision them automatically.
Each course would declare a `toolchain.type` controlling how its tools are set up.

### Provisioning Tiers

```
Course defines in course.yaml:
  toolchain:
    type: embedded | managed | system
```

**Tier 1: Embedded (zero friction)**

The tool runs inside the LearnLocal binary itself. No external install, no setup, instant start.

| Course          | How                                                              |
|-----------------|------------------------------------------------------------------|
| SQL/SQLite      | `rusqlite` embeds the full SQLite engine. App *is* the database. |
| Regex           | Rust `regex` crate. Validate patterns in-process.                |
| jq-like / JSON  | `serde_json` or `jaq`. Parse and query JSON without a binary.   |

SQLite is the standout here — a student runs `learnlocal start sql-fundamentals` and they're
immediately writing queries against a working database. No install page, no setup, nothing.
The app can pre-seed tables per exercise.

**Tier 2: Managed (app downloads/provisions on first run)**

LearnLocal downloads a tool into `~/.local/share/learnlocal/toolchains/` and manages it.
Requires user consent on first run. No sudo, no system-wide changes.

| Course   | How                                                          |
|----------|--------------------------------------------------------------|
| Go       | Official tarball — single download, no installer drama       |
| jq       | Single static binary, drop in cache dir                      |
| Node.js  | Standalone binary (official builds publish these)            |
| Python   | Detect system Python → create venv in course workspace       |

**Tier 3: System (current behavior — detect and guide)**

Tools that can't be trivially downloaded. App detects presence, shows install guidance if missing.

| Course   | Why it stays here                                            |
|----------|--------------------------------------------------------------|
| C/C++    | gcc/clang can't be embedded or trivially downloaded          |
| Rust     | rustup works but it's a full toolchain                       |
| Docker   | Requires system-level daemon                                 |
| LaTeX    | Large distribution (texlive)                                 |

### What This Enables

- **Zero-to-learning in one command** for embedded courses
- **No "install X first" friction** for managed courses
- **Graceful degradation** — managed courses fall back to system tools if already installed
- Toolchain dir is self-contained and deletable (`learnlocal clean-tools`)
