# LearnLocal

**Offline terminal-based programming tutorials. Like vimtutor, for any language.**

[![CI](https://github.com/thehighnotes/learnlocal/actions/workflows/ci.yml/badge.svg)](https://github.com/thehighnotes/learnlocal/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

<p align="center">
  <img src="ui1.png" width="48%" alt="LearnLocal TUI — lesson view" />
  <img src="ui2.png" width="48%" alt="LearnLocal TUI — exercise view" />
</p>

## Why This Exists

Most programming tutorials require a browser, an account, and an internet connection. LearnLocal requires none of that.

- **Offline** — no internet, no cloud, no sign-up
- **Zero config** — install, run, learn
- **Terminal native** — no browser, no GUI, works over SSH
- **Your editor** — uses `$EDITOR`, not a locked-in web IDE
- **Privacy** — your code stays on your machine

## Features

- 10 courses, 500+ exercises covering systems to AI
- Step-based execution engine supporting any language
- Interactive TUI with markdown-rendered lessons
- Shell mode for command-line and sysadmin exercises
- Sandboxed execution (timeout + tmpdir, firejail/bwrap when available)
- Built-in SQLite for SQL courses — no external database needed
- Optional AI hints via Ollama (feature-gated, zero async deps without it)
- Progress tracking across sessions
- Course validation tool for authors

## Course Catalog

| Course | Exercises | Topics |
|--------|-----------|--------|
| C++ Fundamentals | 58 | Variables, control flow, functions, arrays, pointers, structs |
| Python Fundamentals | 57 | Variables, control flow, functions, data structures, strings |
| JavaScript (Node.js) | 58 | Variables, control flow, functions, arrays, objects, modern syntax |
| Rust Fundamentals | 57 | Ownership, types, pattern matching, error handling |
| Go Fundamentals | 58 | Syntax, concurrency, standard library |
| AI Fundamentals (Python) | 57 | Vectors, neural networks, training, attention |
| Linux Fundamentals | 55 | Filesystem, permissions, pipes, text processing |
| SQL (SQLite) | 55 | Queries, schemas, data manipulation |
| Git Time Travel | 42 | Rescue missions, conflicts, lost commits |
| Production Incident Simulator | 42 | Logs, debugging, real sysadmin scenarios |

## Installation

Build from source:

```sh
git clone https://github.com/thehighnotes/learnlocal.git
cd learnlocal
cargo install --path .
```

To include optional AI hints (requires [Ollama](https://ollama.com)):

```sh
cargo install --path . --features llm
```

## Quick Start

```sh
learnlocal                           # browse courses
learnlocal start cpp-fundamentals    # jump into a course
learnlocal validate my-course/       # validate your own course
```

## Comparison

| | LearnLocal | Exercism | Codecademy | Rustlings | vimtutor |
|---|---|---|---|---|---|
| Offline | Yes | No | No | Yes | Yes |
| Terminal-native | Yes | CLI submit | No | Yes | Yes |
| Multi-language | Yes (10) | Yes | Yes | Rust only | Vim only |
| Your `$EDITOR` | Yes | No | No | Yes | No |
| AI hints | Optional | No | Paid | No | No |
| Free | Yes | Yes | Freemium | Yes | Yes |
| Custom courses | Yes | No | No | No | No |
| No sign-up | Yes | No | No | Yes | Yes |

## License

Code is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

Course content in `courses/` is licensed under [CC-BY-4.0](LICENSE-COURSES).
