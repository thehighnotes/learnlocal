# LearnLocal: Offline Programming Tutorial Framework

**Version:** 0.1 Draft  
**Date:** February 7, 2026  
**Author:** Mark Wind  
**Purpose:** Specification for Claude Code implementation

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
│  - Input capture                                            │
│  - Progress display                                         │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    Core Runtime                             │
│  - Course loader/parser                                     │
│  - Lesson sequencer                                         │
│  - Progress tracker                                         │
│  - Validation orchestrator                                  │
└──────────┬──────────────────┬───────────────────┬───────────┘
           │                  │                   │
┌──────────▼──────┐ ┌─────────▼────────┐ ┌───────▼──────────┐
│  Course Packs   │ │ Language Backends│ │  LLM Backend     │
│  (YAML + MD)    │ │ (shell out)      │ │  (optional)      │
└─────────────────┘ └──────────────────┘ └──────────────────┘
```

### 2.2 Directory Structure

```
learnlocal/
├── src/                          # Runtime source code
│   ├── main.rs                   # Entry point (Rust recommended)
│   ├── course/
│   │   ├── mod.rs
│   │   ├── loader.rs             # YAML parsing
│   │   └── validator.rs          # Exercise validation
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── terminal.rs           # TUI rendering
│   │   └── markdown.rs           # MD subset renderer
│   ├── progress/
│   │   ├── mod.rs
│   │   └── tracker.rs            # JSON state management
│   └── llm/
│       ├── mod.rs
│       └── ollama.rs             # Optional LLM integration
│
├── courses/                      # Course packs (can be separate repos)
│   ├── cpp-fundamentals/
│   ├── python-fundamentals/
│   └── rust-fundamentals/
│
├── Cargo.toml                    # Rust dependencies
└── README.md
```

### 2.3 Technology Choice: Rust

**Rationale:**
- Single static binary (no runtime dependencies)
- Excellent terminal UI crates (ratatui, crossterm)
- Fast startup time
- Cross-platform (Linux, macOS, Windows)
- Memory safe

**Key dependencies:**
```toml
[dependencies]
ratatui = "0.25"          # Terminal UI
crossterm = "0.27"        # Terminal control
serde = "1.0"             # Serialization
serde_yaml = "0.9"        # YAML parsing
serde_json = "1.0"        # Progress storage
pulldown-cmark = "0.9"    # Markdown parsing
tokio = "1.0"             # Async (for LLM calls)
reqwest = "0.11"          # HTTP (for Ollama API)
dirs = "5.0"              # XDG directories
```

---

## 3. Course Format Specification

### 3.1 Course Structure

```
courses/
└── cpp-fundamentals/
    ├── course.yaml               # Course metadata
    ├── lessons/
    │   ├── 01-variables/
    │   │   ├── lesson.yaml       # Lesson config
    │   │   ├── content.md        # Explanation text
    │   │   ├── exercises/
    │   │   │   ├── 01-declare.yaml
    │   │   │   ├── 02-assign.yaml
    │   │   │   └── 03-types.yaml
    │   │   └── assets/           # Optional diagrams (ASCII)
    │   │       └── memory.txt
    │   ├── 02-pointers/
    │   └── ...
    └── templates/
        └── main.cpp              # Boilerplate for exercises
```

### 3.2 course.yaml Schema

```yaml
# Course metadata
name: "C++ Fundamentals"
version: "1.0.0"
description: "Learn C++ from the ground up"
author: "LearnLocal Community"
license: "CC-BY-4.0"

# Language configuration
language:
  id: cpp
  display_name: "C++"
  
  # How to compile
  compile:
    command: "g++"
    args: ["-std=c++17", "-Wall", "-Wextra", "-o", "{output}", "{input}"]
    # {input} = source file, {output} = binary name
  
  # How to run (after compilation)
  run:
    command: "./{output}"
    args: []
  
  # File extension
  extension: ".cpp"
  
  # Optional: interpreter-only languages skip compile
  # interpreted: true
  # run:
  #   command: "python3"
  #   args: ["{input}"]

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

# Optional: estimated time per lesson
estimated_minutes_per_lesson: 30
```

### 3.3 lesson.yaml Schema

```yaml
# Lesson metadata
id: variables
title: "Variables and Types"
description: "Learn how to declare and use variables in C++"
estimated_minutes: 25

# Content file (markdown)
content: content.md

# Exercises in order
exercises:
  - 01-declare
  - 02-assign
  - 03-types
  - 04-constants

# Optional: concepts introduced (for dependency tracking)
teaches:
  - variable-declaration
  - primitive-types
  - initialization

# Optional: recap at end
recap: |
  You learned:
  - Variables store data with a specific type
  - int, float, double, char, bool are primitive types
  - Variables must be declared before use
```

### 3.4 Exercise YAML Schema

```yaml
# exercises/01-declare.yaml

id: declare
title: "Declare a Variable"
type: "write"  # write | fix | multiple-choice | fill-blank

# The prompt shown to user
prompt: |
  Declare an integer variable named `age` with the value `25`.

# Starting code (optional)
starter: |
  #include <iostream>
  
  int main() {
      // Your code here
      
      std::cout << age << std::endl;
      return 0;
  }

# Validation method
validation:
  method: "output"  # output | compile-only | regex | custom
  
  # For method: output
  expected_output: "25"
  
  # For method: regex (matches against output)
  # pattern: "^25$"
  
  # For method: custom (runs a script)
  # script: "validate.sh"

# Hints (revealed progressively)
hints:
  - "Variables in C++ need a type before the name"
  - "The syntax is: type name = value;"
  - "For integers, the type is `int`"

# Solution (shown if user gives up)
solution: |
  #include <iostream>
  
  int main() {
      int age = 25;
      
      std::cout << age << std::endl;
      return 0;
  }

# Explanation after success
explanation: |
  `int age = 25;` does three things:
  1. Declares a variable named `age`
  2. Specifies its type as `int` (integer)
  3. Initializes it with the value `25`
```

### 3.5 Exercise Types

| Type | Description | Validation |
|------|-------------|------------|
| `write` | Write code from scratch | Output or regex match |
| `fix` | Fix buggy code | Output match |
| `fill-blank` | Complete partial code | Output match |
| `multiple-choice` | Select correct answer | Exact match |
| `predict` | Predict output of code | Exact match |

### 3.6 content.md Format

Standard markdown with some constraints:

```markdown
# Variables and Types

In C++, every variable has a **type** that determines:
- How much memory it occupies
- What values it can hold
- What operations are valid

## Primitive Types

| Type | Size | Range | Example |
|------|------|-------|---------|
| `int` | 4 bytes | -2B to +2B | `42` |
| `float` | 4 bytes | ±3.4e38 | `3.14f` |
| `double` | 8 bytes | ±1.7e308 | `3.14159` |
| `char` | 1 byte | -128 to 127 | `'A'` |
| `bool` | 1 byte | true/false | `true` |

## Declaration Syntax

```cpp
type name = value;

// Examples:
int count = 0;
float temperature = 98.6f;
char grade = 'A';
bool passed = true;
```

## Memory Visualization

```
┌──────────────────────────────────────┐
│ Memory                               │
├──────────┬──────────┬───────────────┤
│ Address  │ Variable │ Value         │
├──────────┼──────────┼───────────────┤
│ 0x1000   │ count    │ 0             │
│ 0x1004   │ temp     │ 98.6          │
│ 0x1008   │ grade    │ 'A' (65)      │
└──────────┴──────────┴───────────────┘
```

> **Note:** The actual memory addresses will vary each time you run.
```

**Supported markdown subset:**
- Headings (# ## ###)
- Bold, italic, code spans
- Code blocks with language hint
- Tables
- Blockquotes
- Ordered/unordered lists
- Horizontal rules

**Not supported (for terminal simplicity):**
- Images (use ASCII art instead)
- Links (display URL inline)
- HTML

---

## 4. Runtime Behavior

### 4.1 CLI Interface

```bash
# List available courses
$ learnlocal list
Available courses:
  cpp-fundamentals    C++ Fundamentals (12 lessons)
  python-fundamentals Python Fundamentals (10 lessons)

# Start or resume a course
$ learnlocal start cpp-fundamentals

# Check progress
$ learnlocal progress cpp-fundamentals
C++ Fundamentals: 3/12 lessons complete (25%)
  ✓ Variables and Types
  ✓ Operators and Expressions
  ✓ Control Flow
  → Pointers and References (in progress: 2/4 exercises)
  ○ Functions
  ○ Classes and Objects
  ...

# Reset progress
$ learnlocal reset cpp-fundamentals

# Jump to specific lesson
$ learnlocal start cpp-fundamentals --lesson pointers

# Enable AI hints (requires Ollama running)
$ learnlocal start cpp-fundamentals --ai

# Specify custom courses directory
$ learnlocal --courses ~/my-courses list
```

### 4.2 State Machine

```
                    ┌─────────────┐
                    │   START     │
                    └──────┬──────┘
                           │
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
              │     │ AWAIT INPUT │                 │
              │     └──────┬──────┘                 │
              │            │                        │
              │     ┌──────▼──────┐                 │
              │     │  VALIDATE   │                 │
              │     └──────┬──────┘                 │
              │            │                        │
              │    ┌───────┴───────┐                │
              │    │               │                │
         ┌────▼────▼──┐      ┌─────▼─────┐         │
         │   FAIL     │      │  SUCCESS  │         │
         │ show hint  │      │ next ex?  │         │
         └────────────┘      └─────┬─────┘         │
                                   │               │
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

### 4.3 Validation Flow

```rust
// Pseudocode
fn validate_exercise(exercise: &Exercise, user_code: &str) -> ValidationResult {
    // 1. Write user code to temp file
    let temp_file = write_temp_file(user_code, &course.language.extension);
    
    // 2. Compile (if not interpreted)
    if let Some(compile) = &course.language.compile {
        let result = run_command(
            &compile.command,
            &substitute_args(&compile.args, &temp_file, &output_file)
        );
        
        if !result.success {
            return ValidationResult::CompileError(result.stderr);
        }
    }
    
    // 3. Run
    let run_result = run_command(
        &course.language.run.command,
        &substitute_args(&course.language.run.args, &temp_file, &output_file)
    );
    
    // 4. Check output
    match &exercise.validation.method {
        Method::Output => {
            if run_result.stdout.trim() == exercise.validation.expected_output.trim() {
                ValidationResult::Success
            } else {
                ValidationResult::WrongOutput {
                    expected: exercise.validation.expected_output.clone(),
                    actual: run_result.stdout,
                }
            }
        },
        Method::Regex => {
            let re = Regex::new(&exercise.validation.pattern)?;
            if re.is_match(&run_result.stdout) {
                ValidationResult::Success
            } else {
                ValidationResult::WrongOutput { ... }
            }
        },
        Method::Custom => {
            run_validation_script(&exercise.validation.script, &temp_file)
        }
    }
}
```

### 4.4 Progress Storage

Location: `~/.local/share/learnlocal/progress.json`

```json
{
  "version": 1,
  "courses": {
    "cpp-fundamentals": {
      "started_at": "2026-02-07T10:30:00Z",
      "last_activity": "2026-02-07T14:22:00Z",
      "lessons": {
        "variables": {
          "status": "completed",
          "completed_at": "2026-02-07T11:00:00Z",
          "exercises": {
            "01-declare": {"status": "completed", "attempts": 1},
            "02-assign": {"status": "completed", "attempts": 2},
            "03-types": {"status": "completed", "attempts": 1},
            "04-constants": {"status": "completed", "attempts": 3}
          }
        },
        "pointers": {
          "status": "in_progress",
          "exercises": {
            "01-address": {"status": "completed", "attempts": 1},
            "02-dereference": {"status": "in_progress", "attempts": 4}
          }
        }
      }
    }
  }
}
```

---

## 5. Terminal UI

### 5.1 Layout

```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal │ C++ Fundamentals │ Lesson 4/12 │ Exercise 2/4  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ## Pointers and References                                 │
│                                                             │
│  A pointer stores the memory address of another variable.   │
│                                                             │
│  ```cpp                                                     │
│  int x = 42;                                                │
│  int* ptr = &x;  // ptr holds address of x                  │
│  ```                                                        │
│                                                             │
│  The `&` operator gets the address, `*` dereferences it.    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Exercise: Dereference a Pointer                            │
│                                                             │
│  Given a pointer `ptr` to an integer, print the value it    │
│  points to.                                                 │
│                                                             │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ #include <iostream>                                    │ │
│  │                                                        │ │
│  │ int main() {                                           │ │
│  │     int value = 100;                                   │ │
│  │     int* ptr = &value;                                 │ │
│  │                                                        │ │
│  │     // Print the value ptr points to                   │ │
│  │     std::cout << _ << std::endl;                       │ │
│  │                                                        │ │
│  │     return 0;                                          │ │
│  │ }                                                      │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [Enter] Submit  [h] Hint (2 left)  [s] Skip  [a] AI Help    │
│ [←/→] Navigate  [q] Quit                                    │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Code Editor

Minimal inline editor for exercise code:

**Features:**
- Syntax highlighting (basic: keywords, strings, comments)
- Line numbers
- Cursor navigation (arrow keys)
- Basic editing (insert, delete, backspace)
- Home/End, PgUp/PgDn

**Not needed (keep simple):**
- Multiple cursors
- Find/replace
- Auto-indent
- Autocomplete

**Alternative:** Allow `$EDITOR` to open external editor:
```bash
# User can configure
export LEARNLOCAL_EDITOR=vim

# Then 'e' key opens current exercise in vim
# On save+exit, learnlocal picks up the changes
```

### 5.3 Colors and Themes

```rust
// Minimal color scheme
struct Theme {
    heading: Color,      // Cyan
    code: Color,         // White
    keyword: Color,      // Yellow
    string: Color,       // Green
    comment: Color,      // Gray
    error: Color,        // Red
    success: Color,      // Green
    prompt: Color,       // Blue
    muted: Color,        // DarkGray
}

// Support NO_COLOR environment variable
if std::env::var("NO_COLOR").is_ok() {
    // Disable all colors
}
```

---

## 6. LLM Integration (Optional)

### 6.1 Architecture

```
┌─────────────────────┐
│   LearnLocal UI     │
└──────────┬──────────┘
           │ User query + full context
┌──────────▼──────────┐
│   LLM Backend       │
│   (abstraction)     │
└──────────┬──────────┘
           │
     ┌─────┴─────┬────────────────┐
     │           │                │
┌────▼────┐ ┌────▼─────┐ ┌────────▼────────┐
│ Ollama  │ │ Remote   │ │ OpenAI-compat   │
│ (local) │ │ (AGX)    │ │ (any endpoint)  │
└────┬────┘ └────┬─────┘ └────────┬────────┘
     │           │                │
┌────▼────┐ ┌────▼─────┐ ┌────────▼────────┐
│ qwen3:4b│ │ Qwen/TRT │ │ Claude/GPT/etc  │
│ llama3  │ │ via API  │ │                 │
└─────────┘ └──────────┘ └─────────────────┘
```

### 6.2 Configuration

Location: `~/.config/learnlocal/config.yaml`

```yaml
# LLM Backend Configuration
llm:
  # Which backend to use: ollama | remote | openai-compatible | disabled
  backend: ollama
  
  # Ollama settings (default)
  ollama:
    url: "http://localhost:11434"
    model: "qwen3:4b"
    # Fallback models if primary unavailable
    fallback_models: ["llama3:8b", "mistral:7b"]
  
  # Remote endpoint (e.g., your AGX Orin via WireGuard)
  remote:
    url: "http://10.0.0.50:8080"  # AGX on WireGuard
    # Or via VPS tunnel
    # url: "https://your-vps.com/llm"
    api_key: "${LEARNLOCAL_REMOTE_KEY}"  # Optional, from env
    model: "qwen3-4b-trt"
    timeout_seconds: 30
  
  # OpenAI-compatible API (Claude, GPT, etc.)
  openai_compatible:
    url: "https://api.anthropic.com/v1"
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-sonnet-4-20250514"
  
  # Behavior settings
  settings:
    # Max tokens for response
    max_tokens: 500
    # Temperature (0 = deterministic, 1 = creative)
    temperature: 0.3
    # Include full lesson content in context
    include_lesson_content: true
    # Include previous exercises in lesson
    include_previous_exercises: true
    # Include user's attempt history for current exercise
    include_attempt_history: true
    # Max previous attempts to include
    max_history_attempts: 3

# Keybinding for AI chat
keybindings:
  ai_chat: "a"        # Open AI chat panel
  ai_hint: "h"        # Quick hint (uses canned prompt)
  ai_explain: "e"     # Explain compiler error
```

### 6.3 Backend Abstraction

```rust
// src/llm/mod.rs

pub trait LlmBackend: Send + Sync {
    /// Check if backend is available
    async fn is_available(&self) -> bool;
    
    /// Send a message with full context, get response
    async fn chat(&self, context: &LlmContext, message: &str) -> Result<String>;
    
    /// Stream response token by token (for real-time display)
    async fn chat_stream(
        &self, 
        context: &LlmContext, 
        message: &str,
        callback: impl Fn(&str) + Send
    ) -> Result<()>;
    
    /// Get backend name for display
    fn name(&self) -> &str;
}

pub enum Backend {
    Ollama(OllamaBackend),
    Remote(RemoteBackend),
    OpenAiCompatible(OpenAiBackend),
}

impl Backend {
    pub fn from_config(config: &LlmConfig) -> Result<Self> {
        match config.backend.as_str() {
            "ollama" => Ok(Backend::Ollama(OllamaBackend::new(&config.ollama)?)),
            "remote" => Ok(Backend::Remote(RemoteBackend::new(&config.remote)?)),
            "openai-compatible" => Ok(Backend::OpenAiCompatible(OpenAiBackend::new(&config.openai_compatible)?)),
            "disabled" => Err(Error::LlmDisabled),
            _ => Err(Error::UnknownBackend(config.backend.clone())),
        }
    }
}
```

### 6.4 Full Context Construction

The key differentiator: AI gets **full context** of where the student is.

```rust
// src/llm/context.rs

/// Complete context passed to LLM for every interaction
#[derive(Debug, Clone, Serialize)]
pub struct LlmContext {
    // === Course Context ===
    pub course_name: String,           // "C++ Fundamentals"
    pub language: String,              // "C++"
    
    // === Lesson Context ===
    pub lesson_title: String,          // "Pointers and References"
    pub lesson_content: String,        // Full markdown content
    pub lesson_position: String,       // "Lesson 4 of 12"
    pub concepts_taught: Vec<String>,  // ["pointers", "references", "address-of"]
    
    // === Exercise Context ===
    pub exercise_title: String,        // "Dereference a Pointer"
    pub exercise_prompt: String,       // The task description
    pub exercise_type: String,         // "write" | "fix" | "fill-blank"
    pub starter_code: String,          // Original template
    
    // === Student State ===
    pub current_code: String,          // What they've written now
    pub attempt_number: u32,           // Current attempt (1, 2, 3...)
    pub attempt_history: Vec<Attempt>, // Previous attempts this exercise
    pub hints_revealed: Vec<String>,   // Hints already shown
    pub time_spent_seconds: u64,       // Time on this exercise
    
    // === Compilation/Runtime State ===
    pub last_compile_output: Option<CompileResult>,
    pub last_run_output: Option<RunResult>,
    
    // === Previous Exercises (same lesson) ===
    pub completed_exercises: Vec<CompletedExercise>,
    
    // === Progress Context ===
    pub lessons_completed: u32,        // How far in course
    pub total_lessons: u32,
    pub struggles: Vec<String>,        // Concepts student struggled with
}

#[derive(Debug, Clone, Serialize)]
pub struct Attempt {
    pub code: String,
    pub compile_result: Option<CompileResult>,
    pub run_result: Option<RunResult>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub expected_output: String,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletedExercise {
    pub title: String,
    pub attempts_needed: u32,
    pub final_code: String,
}
```

### 6.5 Context Rendering for LLM

```rust
// src/llm/prompt.rs

impl LlmContext {
    /// Render full context as system prompt
    pub fn to_system_prompt(&self) -> String {
        format!(r#"
You are a friendly programming tutor helping a student learn {language}.

## Current Position
- Course: {course_name}
- {lesson_position}
- Exercise: "{exercise_title}" (attempt #{attempt_number})

## Lesson Content
The student just read this material:

{lesson_content}

## Exercise Details
Type: {exercise_type}
Task: {exercise_prompt}

Starter code:
```{language_lower}
{starter_code}
```

## Student's Current Code
```{language_lower}
{current_code}
```

## Compilation/Run Results
{compile_run_summary}

## Attempt History
{attempt_history_summary}

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
            language = self.language,
            course_name = self.course_name,
            lesson_position = self.lesson_position,
            exercise_title = self.exercise_title,
            attempt_number = self.attempt_number,
            lesson_content = self.truncate_lesson_content(2000),
            exercise_type = self.exercise_type,
            exercise_prompt = self.exercise_prompt,
            language_lower = self.language.to_lowercase(),
            starter_code = self.starter_code,
            current_code = self.current_code,
            compile_run_summary = self.format_compile_run(),
            attempt_history_summary = self.format_attempt_history(),
            hints_summary = self.format_hints(),
        )
    }
    
    fn format_compile_run(&self) -> String {
        let mut out = String::new();
        
        if let Some(compile) = &self.last_compile_output {
            if compile.success {
                out.push_str("Compilation: ✓ Success\n");
            } else {
                out.push_str(&format!(
                    "Compilation: ✗ Failed\n```\n{}\n```\n",
                    compile.stderr.trim()
                ));
            }
        }
        
        if let Some(run) = &self.last_run_output {
            out.push_str(&format!(
                "Output: {}\nExpected: {}\nMatch: {}\n",
                run.stdout.trim(),
                run.expected_output.trim(),
                if run.matched { "✓" } else { "✗" }
            ));
        }
        
        if out.is_empty() {
            out.push_str("(not yet compiled/run)");
        }
        
        out
    }
    
    fn format_attempt_history(&self) -> String {
        if self.attempt_history.is_empty() {
            return "(first attempt)".to_string();
        }
        
        self.attempt_history
            .iter()
            .enumerate()
            .map(|(i, attempt)| {
                let status = match (&attempt.compile_result, &attempt.run_result) {
                    (Some(c), _) if !c.success => "compile error",
                    (_, Some(r)) if !r.matched => "wrong output",
                    (_, Some(r)) if r.matched => "passed",
                    _ => "unknown",
                };
                format!("Attempt {}: {}", i + 1, status)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

### 6.6 AI Chat Panel UI

```
┌─────────────────────────────────────────────────────────────┐
│ LearnLocal │ C++ Fundamentals │ Lesson 4/12 │ Exercise 2/4  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [Exercise content as before...]                            │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ AI Tutor (qwen3:4b @ localhost)                      [Esc]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ You: Why do I need the asterisk before ptr?                 │
│                                                             │
│ AI: Good question! In C++, the asterisk (*) has two         │
│ different meanings depending on context:                    │
│                                                             │
│ 1. In a declaration (`int* ptr`), it means "ptr is a        │
│    pointer to an int"                                       │
│                                                             │
│ 2. In an expression (`*ptr`), it means "get the value       │
│    that ptr points to" — this is called dereferencing       │
│                                                             │
│ In your exercise, you need to print the VALUE, not the      │
│ address. So you need to dereference. Look at line 8 in      │
│ your code — what's missing there?                           │
│                                                             │
│ ─────────────────────────────────────────────────────────── │
│ You: Oh! So *ptr gives me 100, but ptr gives the address?   │
│                                                             │
│ AI: Exactly! You've got it. Try it out.                     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ > _                                                         │
│                                                             │
│ [Enter] Send  [Esc] Close  [↑] History  [Ctrl+L] Clear      │
└─────────────────────────────────────────────────────────────┘
```

### 6.7 Remote Backend (AGX Orin)

For your specific setup — AGX accessible via WireGuard:

```rust
// src/llm/remote.rs

pub struct RemoteBackend {
    client: Client,
    config: RemoteConfig,
}

impl RemoteBackend {
    pub fn new(config: &RemoteConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        
        if let Some(key) = &config.api_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", key))?
            );
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()?;
        
        Ok(Self { client, config })
    }
}

impl LlmBackend for RemoteBackend {
    async fn chat(&self, context: &LlmContext, message: &str) -> Result<String> {
        // Supports multiple API formats
        let body = match self.config.api_format.as_str() {
            "ollama" => json!({
                "model": self.config.model,
                "prompt": format!("{}\n\nUser: {}", context.to_system_prompt(), message),
                "stream": false,
            }),
            "openai" => json!({
                "model": self.config.model,
                "messages": [
                    {"role": "system", "content": context.to_system_prompt()},
                    {"role": "user", "content": message}
                ],
            }),
            // Custom format for your AGX endpoint
            "custom" => json!({
                "system": context.to_system_prompt(),
                "user": message,
                "model": self.config.model,
                "max_tokens": 500,
            }),
            _ => return Err(Error::UnknownApiFormat),
        };
        
        let response = self.client
            .post(&self.config.url)
            .json(&body)
            .send()
            .await?;
        
        // Parse response based on format
        // ...
    }
    
    async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/health", self.config.url))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .is_ok()
    }
    
    fn name(&self) -> &str {
        "Remote (AGX)"
    }
}
```

### 6.8 AGX Endpoint Specification

If you want to run a simple API on your AGX for LearnLocal to call:

```python
# agx_tutor_api.py — minimal Flask endpoint on AGX

from flask import Flask, request, jsonify
import ollama  # or your TRT inference code

app = Flask(__name__)

@app.route('/health', methods=['GET'])
def health():
    return jsonify({"status": "ok"})

@app.route('/chat', methods=['POST'])
def chat():
    data = request.json
    
    system_prompt = data.get('system', '')
    user_message = data.get('user', '')
    model = data.get('model', 'qwen3:4b')
    max_tokens = data.get('max_tokens', 500)
    
    # Using Ollama on AGX
    response = ollama.chat(
        model=model,
        messages=[
            {'role': 'system', 'content': system_prompt},
            {'role': 'user', 'content': user_message}
        ],
        options={'num_predict': max_tokens}
    )
    
    return jsonify({
        "response": response['message']['content'],
        "model": model,
        "backend": "agx-ollama"
    })

if __name__ == '__main__':
    # Bind to WireGuard interface
    app.run(host='10.0.0.50', port=8080)
```

Or with your TensorRT runtime once it's ready:

```python
@app.route('/chat', methods=['POST'])
def chat_trt():
    data = request.json
    
    # Your TRT inference
    from prometheus_runtime import TRTInference
    
    model = TRTInference("qwen3-4b.engine")
    response = model.generate(
        system=data['system'],
        user=data['user'],
        max_tokens=data.get('max_tokens', 500)
    )
    
    return jsonify({
        "response": response,
        "model": "qwen3-4b-trt",
        "backend": "agx-tensorrt"
    })
```

### 6.9 Automatic Backend Selection

```rust
// src/llm/selector.rs

pub async fn select_best_backend(config: &LlmConfig) -> Result<Box<dyn LlmBackend>> {
    // Priority order based on config
    let backends: Vec<(&str, Box<dyn LlmBackend>)> = vec![
        ("remote", Box::new(RemoteBackend::new(&config.remote)?)),
        ("ollama", Box::new(OllamaBackend::new(&config.ollama)?)),
    ];
    
    for (name, backend) in backends {
        if backend.is_available().await {
            println!("AI backend: {} ✓", backend.name());
            return Ok(backend);
        } else {
            println!("AI backend: {} unavailable, trying next...", name);
        }
    }
    
    Err(Error::NoBackendAvailable)
}
```

### 6.10 CLI Flags

```bash
# Use specific backend
$ learnlocal start cpp-fundamentals --ai --backend ollama
$ learnlocal start cpp-fundamentals --ai --backend remote

# Override model
$ learnlocal start cpp-fundamentals --ai --model qwen3:8b

# Override endpoint (one-off, doesn't save to config)
$ learnlocal start cpp-fundamentals --ai --endpoint http://192.168.1.100:8080

# Disable AI even if configured
$ learnlocal start cpp-fundamentals --no-ai

# Check AI status
$ learnlocal ai-status
Checking AI backends...
  ollama (localhost:11434): ✓ available (qwen3:4b)
  remote (10.0.0.50:8080):  ✓ available (qwen3-4b-trt)
  
Active backend: remote (preferred in config)
```

### 6.11 Quick Actions vs Chat

Two modes of AI interaction:

**Quick Actions (single keypress):**
| Key | Action | Canned Prompt |
|-----|--------|---------------|
| `h` | Hint | "Give me a small hint without revealing the answer" |
| `e` | Explain Error | "Explain this compiler error in simple terms: {error}" |
| `c` | Check Approach | "Is my approach correct? Don't give the answer, just say if I'm on track" |
| `w` | Why Wrong | "My output is {actual} but expected {expected}. Why?" |

**Chat Mode (`a` key):**
- Opens chat panel
- Full conversation history within session
- Free-form questions
- Context automatically included

```rust
// Quick action example
fn handle_quick_hint(context: &LlmContext, backend: &dyn LlmBackend) -> Result<String> {
    let prompt = "Give me a small hint for this exercise. \
                  Don't reveal the answer, just point me in the right direction. \
                  Keep it under 50 words.";
    
    backend.chat(context, prompt).await
}

fn handle_explain_error(context: &LlmContext, backend: &dyn LlmBackend) -> Result<String> {
    let error = context.last_compile_output
        .as_ref()
        .map(|c| c.stderr.clone())
        .unwrap_or_default();
    
    let prompt = format!(
        "Explain this compiler error in simple terms for a beginner:\n\n{}\n\n\
         What's wrong and what should I look for?",
        error
    );
    
    backend.chat(context, &prompt).await
}

---

## 7. Implementation Phases

### Phase 1: Core Runtime (MVP)

**Goal:** Single course, basic validation, no LLM

**Deliverables:**
- [ ] Course loader (parse course.yaml, lesson.yaml)
- [ ] Markdown renderer (terminal subset)
- [ ] Exercise runner (compile + run + check output)
- [ ] Progress tracker (JSON file)
- [ ] Basic TUI (lesson view, exercise input, submit)
- [ ] One complete course: `cpp-fundamentals` (5 lessons)

**Commands:**
```bash
learnlocal list
learnlocal start cpp-fundamentals
learnlocal progress cpp-fundamentals
```

**Timeline:** 2-3 weeks

---

### Phase 2: Editor & UX Polish

**Goal:** Better code editing, visual feedback

**Deliverables:**
- [ ] Inline code editor with syntax highlighting
- [ ] External editor support ($EDITOR)
- [ ] Colored diff for wrong output
- [ ] Animated success/failure feedback
- [ ] Keyboard shortcuts help overlay
- [ ] `--lesson` flag to jump to specific lesson

**Timeline:** 1-2 weeks

---

### Phase 3: LLM Integration

**Goal:** Optional AI hints via Ollama

**Deliverables:**
- [ ] Ollama backend module
- [ ] `--ai` flag to enable
- [ ] [a] key for "ask AI"
- [ ] Context construction from current exercise state
- [ ] Response rendering in UI
- [ ] Graceful fallback if Ollama unavailable

**Timeline:** 1 week

---

### Phase 4: Additional Courses

**Goal:** Prove language-agnostic design

**Deliverables:**
- [ ] `python-fundamentals` course
- [ ] `rust-fundamentals` course (meta!)
- [ ] Course contribution guide (CONTRIBUTING.md)
- [ ] Course validation tool (`learnlocal validate courses/my-course`)

**Timeline:** 2 weeks per course

---

### Phase 5: Distribution

**Goal:** Easy installation

**Deliverables:**
- [ ] Pre-built binaries (Linux x64, ARM64, macOS, Windows)
- [ ] GitHub releases workflow
- [ ] Install script (`curl -fsSL ... | sh`)
- [ ] AUR package (Arch Linux)
- [ ] Homebrew formula (macOS)

**Timeline:** 1 week

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_course_loader() {
        let course = load_course("test_data/sample_course").unwrap();
        assert_eq!(course.name, "Sample Course");
        assert_eq!(course.lessons.len(), 2);
    }
    
    #[test]
    fn test_output_validation() {
        let result = validate_output("hello world\n", "hello world");
        assert!(result.is_success());
    }
    
    #[test]
    fn test_output_validation_fails() {
        let result = validate_output("hello world\n", "goodbye world");
        assert!(result.is_wrong_output());
    }
}
```

### 8.2 Integration Tests

```rust
#[test]
fn test_full_exercise_flow() {
    let course = load_course("courses/cpp-fundamentals").unwrap();
    let lesson = course.get_lesson("variables").unwrap();
    let exercise = lesson.get_exercise("01-declare").unwrap();
    
    let user_code = "int age = 25;";
    let result = run_exercise(&course.language, &exercise, user_code);
    
    assert!(result.is_success());
}
```

### 8.3 Course Validation

```bash
# Tool to validate course structure
$ learnlocal validate courses/cpp-fundamentals

Validating cpp-fundamentals...
  ✓ course.yaml is valid
  ✓ All lessons have content.md
  ✓ All exercises have valid schemas
  ✓ All solutions compile
  ✓ All solutions pass validation
  ✓ No dependency cycles in lessons

Course is valid!
```

---

## 9. Future Considerations

### 9.1 Potential Features (Post-1.0)

- **Challenges:** Timed exercises, competitive mode
- **Certificates:** Completion badges (local, printable)
- **Spaced repetition:** Review exercises from past lessons
- **Custom courses:** Course creation wizard
- **Sync:** Optional cloud backup of progress
- **Multiplayer:** Local network peer learning

### 9.2 Course Marketplace

Potential for community course sharing:
```bash
# Future
$ learnlocal search "rust"
Available courses:
  rust-fundamentals (official)       ★★★★★ (4.8)
  rust-web-dev (community)           ★★★★☆ (4.2)
  rust-embedded (community)          ★★★★☆ (4.5)

$ learnlocal install rust-web-dev
```

### 9.3 IDE Integration

VS Code / Neovim extensions that:
- Show lesson content in sidebar
- Run exercises from editor
- Display progress inline

---

## 10. Open Questions

1. **Editor choice:** Build minimal editor or always defer to `$EDITOR`?
2. **Course versioning:** How to handle course updates when user has partial progress?
3. **Sandbox:** Should we sandbox code execution for safety?
4. **Multi-file exercises:** How to handle projects that need multiple files?
5. **Interactive exercises:** Support for exercises that need stdin input?

---

## Appendix A: Example Course Snippet

### courses/cpp-fundamentals/course.yaml

```yaml
name: "C++ Fundamentals"
version: "1.0.0"
description: "Learn C++ from the ground up. No prior experience needed."
author: "LearnLocal Community"
license: "CC-BY-4.0"

language:
  id: cpp
  display_name: "C++"
  compile:
    command: "g++"
    args: ["-std=c++17", "-Wall", "-Wextra", "-o", "{output}", "{input}"]
  run:
    command: "./{output}"
    args: []
  extension: ".cpp"

lessons:
  - id: variables
    title: "Variables and Types"
  - id: operators
    title: "Operators and Expressions"
    requires: [variables]
  - id: control-flow
    title: "Control Flow"
    requires: [operators]
  - id: pointers
    title: "Pointers and References"
    requires: [variables]
  - id: functions
    title: "Functions"
    requires: [control-flow]
```

### courses/cpp-fundamentals/lessons/01-variables/content.md

```markdown
# Variables and Types

Every program needs to store data. In C++, we use **variables** — named containers that hold values.

## Why Types Matter

Unlike some languages, C++ requires you to specify the **type** of data a variable holds:

```cpp
int age = 25;        // Integer (whole number)
double price = 9.99; // Decimal number
char grade = 'A';    // Single character
bool passed = true;  // True or false
```

The type determines:
- How much memory is used
- What operations are valid
- What values are allowed

## Declaring Variables

The syntax is always: `type name = value;`

```cpp
int count = 0;        // Declare and initialize
int total;            // Declare only (dangerous!)
total = 100;          // Assign later
```

> ⚠️ **Warning:** Always initialize your variables. Uninitialized variables contain garbage data.

## Naming Rules

Variable names in C++:
- Must start with a letter or underscore
- Can contain letters, digits, underscores
- Are case-sensitive (`age` ≠ `Age`)
- Cannot be reserved words (`int`, `return`, etc.)

```cpp
int playerScore;     // ✓ Good (camelCase)
int player_score;    // ✓ Good (snake_case)
int 2fast;           // ✗ Starts with digit
int my-var;          // ✗ Contains hyphen
```

Now let's practice!
```

### courses/cpp-fundamentals/lessons/01-variables/exercises/01-declare.yaml

```yaml
id: declare
title: "Declare an Integer"
type: write

prompt: |
  Declare an integer variable named `score` with the value `100`.
  The program should print the score.

starter: |
  #include <iostream>
  
  int main() {
      // Declare score here
      
      std::cout << score << std::endl;
      return 0;
  }

validation:
  method: output
  expected_output: "100"

hints:
  - "An integer type in C++ is `int`"
  - "The syntax is: type name = value;"
  - "You need: int score = 100;"

solution: |
  #include <iostream>
  
  int main() {
      int score = 100;
      
      std::cout << score << std::endl;
      return 0;
  }

explanation: |
  `int score = 100;` declares an integer variable and initializes it.
  
  Breaking it down:
  - `int` — the type (integer)
  - `score` — the variable name
  - `=` — assignment operator
  - `100` — the initial value
  - `;` — statement terminator (required!)
```

---

*End of specification.*
