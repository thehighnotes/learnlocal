# LearnLocal — Feature Ideas

Brainstormed 2026-02-16. Organized by readiness tier based on existing infrastructure.

---

## Tier 1: Ready Now (infrastructure already exists)

### Hole-Driven Exercises
- **What it enables:** Exercises with `___` blanks in starter code. Students fill holes; validator checks compilation + test output. Like a crossword puzzle for code. Especially devastating for Rust where the type system is essentially a puzzle game.
- **What's blocking it:** Nothing. This is a course authoring pattern, not an engine change. Zero engine modifications needed. Write a course with holes tomorrow.

### Compiler Construction Chain (Challenge Chains)
- **What it enables:** Multi-exercise problems where each exercise builds on the last and student code carries forward. Exercise 1: tokenizer. Exercise 2: parser. Exercise 3: AST. By lesson 8 the student has a working interpreter they built. Not toy exercises — a real artifact at the end.
- **What's blocking it:** Nothing. Persistent sandbox files (`src/state/sandbox.rs`) already carry student code forward between exercises. This is a course design choice, not a feature.

### Production Incident Simulator
- **What it enables:** Terminal goes red. Logs scroll. A service is down. Clock is ticking. The student has a sandboxed broken system and must fix it live. Not "write a function" — triage, diagnose, fix under pressure. `learnlocal incident` turns the terminal into a war room.
- **What's blocking it:** Barely anything. The environment engine already spins up background services, allocates ports, sets up broken filesystem state, and runs assertions. A "fix this broken server" exercise is just a creative use of `environment:` blocks with `services:`, `env_files:`, and `setup:` commands. The hardest part is making the terminal go red (UI polish).

---

## Tier 2: Near-Ready (one new exercise type or mode)

### Two-Player Protocol Exercises
- **What it enables:** Write both sides of a network protocol. A client AND a server, in two files, that talk to each other over a socket. The sandbox runs both. Both sides' logs interleave. Teaches networking by making the student build both ends and watch them handshake in real time.
- **What's blocking it:** Multi-file exercises + background services + port allocation already exist. Student writes `server.py` and `client.py`, env engine starts the server, runs the client against it, validates output. The plumbing is there. Needs a course that uses it this way.

### Git Time Travel Exercises
- **What it enables:** The exercise IS a git repo in a broken state. Conflicting merges. Detached HEADs. Rebases gone wrong. Fix it with actual git commands. "Here's a repo where someone force-pushed over your work. Recover the lost commits." This doesn't exist anywhere as a hands-on learning tool.
- **What's blocking it:** `setup:` commands can already create an entire git repo in a broken state. The gap: exercises currently expect "edit a file and run steps." Needs a `type: command` exercise where the student's input is shell commands, not a code file. That is a real but bounded engine addition — the single biggest unlock for multiple features.

### Your Own Archaeology
- **What it enables:** After finishing a course, the system diffs the student's first exercise solution against their last. Shows style evolution. Variable naming improving. Fewer unnecessary variables. More idiomatic patterns. A mirror that shows how much the student grew. Automatically generated, deeply personal.
- **What's blocking it:** Progress JSON already stores every attempt with timestamps. The diff engine exists. This is a new TUI screen that reads existing data and diffs first solution against last. Pure frontend work over data that already exists.

### Teach-Back Mode
- **What it enables:** After completing a concept, the system asks the student to explain it. In writing. In the terminal. The local LLM evaluates the explanation for accuracy and completeness. "You said recursion 'calls itself' but didn't mention the base case — try again." Forces the deepest form of learning: teaching.
- **What's blocking it:** LLM integration exists. Inline editor exists. A `type: explain` exercise where the "code" is prose and the LLM validates accuracy is a new exercise type, but the moving parts are all wired up.

### Debug Exercises
- **What it enables:** A `type: debug` exercise where the student gets broken code and an error message; the job is to find and fix the bug. Trains the single most important skill beginners lack — reading error messages. Nobody treats this as a first-class exercise type.
- **What's blocking it:** The diff engine already shows what changed. Needs a new exercise type where validation compares the student's fix against the expected correction, or just checks that tests pass. Bounded work.

### Test-Writing Exercises
- **What it enables:** `type: test` — here's a function with hidden bugs. Write tests that catch them all. The validator runs the student's tests against both the buggy version AND a correct version. Tests must pass on correct code and fail on buggy code. Completely inverts the learning model and teaches the skill that separates junior from senior developers.
- **What's blocking it:** Needs a new exercise type and a validation mode that runs student tests against two versions of code. The execution engine can handle this but needs orchestration for the dual-run approach.

---

## Tier 3: Medium Work (foundation solid, needs orchestration)

### Adversarial AI
- **What it enables:** The student writes code. The local LLM actively tries to break it. Fuzzes inputs, finds edge cases, throws garbage at the function. The student doesn't pass until the AI gives up trying to crash them. "Your `parse_email()` survived 847 attacks." That's a screenshot people post.
- **What's blocking it:** LLM generates inputs, sandbox runs code, compare against expected behavior. The pieces exist individually. The new part is the attack loop: LLM sees code, generates evil input, sandbox runs it, reports back, LLM tries again. Maybe 3-5 cycles. That's a new execution mode but not a new universe.

### Sabotage Mode (Code Review Exercises)
- **What it enables:** The AI writes code. The student is the senior reviewer. Find the bugs before it "ships." Starts with obvious issues and gets progressively sneakier — race conditions, off-by-ones hidden in correct-looking logic, subtle security holes. Trains code review as a muscle. Also works as `type: review` where the local LLM evaluates whether the student caught the key issues.
- **What's blocking it:** LLM can generate buggy code. Needs a `type: review` exercise where the student marks lines as buggy rather than writing code. New UI pattern — the inline editor in read-mostly mode with line annotations. Medium lift.

### Escape Room Progression
- **What it enables:** Non-linear exercise unlocking. Solving exercise A gives a key (a value, a hash, a port number) needed to unlock exercise D. But exercise D also needs something from exercise C. The student solves a dependency graph, not a list. The course itself is a puzzle.
- **What's blocking it:** Needs the progress/state system to track "keys" (output from one exercise unlocks another). The course format would need `unlock_requires:` fields. Bounded schema change + state tracking addition.

### AI-Generated Infinite Practice
- **What it enables:** The local LLM has access to signal data — attempt counts, failures, which exercises took forever. It generates new exercises targeting weak spots. "You struggled with pointer arithmetic in exercises 4.3 and 4.5 — here are 3 more, progressively harder." Infinite personalized practice, running entirely on the student's machine. Duolingo's adaptive model with zero data leaving the laptop.
- **What's blocking it:** LLM needs to generate valid exercise YAML, the runtime needs to load and execute dynamically-generated exercises, and validation needs to work on AI-created problems. The infrastructure exists but the generation-to-execution pipeline is new.

### Cross-Language Rosetta Mode
- **What it enables:** Same concept, side-by-side across languages. "Implement a hash map" in Python, then C++, then Rust. A first-class "compare" view showing solutions side by side in a split terminal. Polyglot developers would lose their minds.
- **What's blocking it:** 7+ languages exist as courses. Needs a cross-course linking mechanism and a split-terminal comparison view. Both are new but bounded.

### Kata Mode
- **What it enables:** Timed repetition. Completed an exercise? Do it again, faster. Track solve times. Show a personal-best graph in the terminal. `learnlocal kata` drops the student into random completed exercises with a countdown timer. What makes typing tutors addictive — not learning, mastery. The feeling of a problem that once took 20 minutes now taking 90 seconds.
- **What's blocking it:** Progress data exists. Timer exists. Needs a new mode that selects random completed exercises, times the student, and tracks personal bests. Medium frontend + state work.

### Regex Golf
- **What it enables:** "Match ALL of these strings. Match NONE of those." Fewest characters wins. A live terminal regex debugger highlights matches as the student types. Personal best tracking. Shareable challenges.
- **What's blocking it:** Needs a new exercise type with live regex evaluation and a visual match display. A new validator type for regex correctness, plus the real-time feedback loop.

### Code Dictation
- **What it enables:** The system describes a function in rapid plain English. The student types it live. "A function that takes a vector of integers, filters out the negatives, squares the rest, and returns the sum." Clock running. Tests run on submit. Speed + correctness.
- **What's blocking it:** Needs a timed prompt-display mode and the existing test validation. The pieces exist but need a new presentation mode.

### Exercise Packs as Gists
- **What it enables:** A single `.yaml` file pasted into a GitHub Gist. `learnlocal play <gist-url>` fetches and runs it. One exercise, self-contained. People share exercises on Reddit like puzzles. "Can you solve this in under 5 minutes?" Turns every programmer into a potential content creator with zero friction.
- **What's blocking it:** Needs a fetch-and-load path for single-exercise YAML files from URLs. The exercise runtime exists; the distribution/fetch layer is new.

---

## Tier 4: Significant New Infrastructure

### Memory Corruption Visualizer
- **What it enables:** For C++ and Rust. The student's code has a buffer overflow. The terminal renders memory as colored blocks in real time — and the student watches their data get overwritten. Stack smashing visualized with box-drawing characters. Segfaults become visible, not mysterious.
- **What's blocking it:** Needs valgrind/AddressSanitizer integration + a TUI memory renderer. A real project — both the tooling integration and the visualization are substantial.

### System Call X-Ray
- **What it enables:** Code runs under strace. The exercise renders syscalls as a timeline. "Your file reader made 4,000 read() calls. The reference solution made 12. Why? Fix it." Teaches what code actually does at the OS level.
- **What's blocking it:** strace integration + syscall timeline renderer. Real project — needs OS-level tracing wired into the sandbox and a visualization layer.

### ASCII Flamegraph Profiling
- **What it enables:** The student's solution works but it's slow. The terminal renders a flamegraph in ASCII art. Find the hot path. Optimize. Rerun. Watch the flamegraph shrink. Performance optimization as a visual, iterative game.
- **What's blocking it:** Profiler integration + flamegraph TUI renderer. Real project — needs perf/instruments integration and a custom visualization.

### Terminal Code Visualization
- **What it enables:** ASCII/TUI rendering of what's actually happening when code runs. Memory layout boxes for C++/Rust. A call stack growing and shrinking. Variable bindings lighting up as they change. Python Tutor but in the terminal, offline, rendered with box-drawing characters.
- **What's blocking it:** Technically hard — needs execution tracing, state capture, and a real-time TUI renderer. But visually stunning in a gif, which is what matters for virality.

---

## Implemented

### `type: command` Exercises [DONE]
- Student writes shell commands, validated by state assertions. Works in any course regardless of language.
- Enables: git exercises, sysadmin scenarios, mixed-language courses, file management challenges.
- Status: **Shipped.** Git Time Travel and Incident Simulator courses built on top of it.

### Code Golf Mode [DONE]
- `golf: true` on any exercise. Shows character count vs "par" (reference solution length) after success.
- "[GOLF]" badge on exercise prompt. Green "At or under par!" for beating the reference. Red "+N" for over par.
- Status: **Shipped.** Sprinkled across courses on exercises where brevity is interesting.

---

## Tier 1.5: Small TUI Features (high fun, bounded work)

### Terminal Achievements
- **What it enables:** Pop-up badges for milestones. "First Blood" (first exercise), "Streak: 10" (no failures in a row), "Speed Demon" (under 60s), "Night Owl" (coding at 3am), "Par Buster" (beat golf par), "Completionist" (finish a course). Zero functional value. Maximum dopamine.
- **What's blocking it:** Progress data exists. Need a persistent achievements store + a small overlay renderer. Bounded.

### Boss Battles
- **What it enables:** Last exercise of each course (or lesson) is a massive multi-part challenge combining everything taught. Different celebration art. "BOSS DEFEATED." visual. Makes finishing a lesson feel like beating a game level.
- **What's blocking it:** Nothing — this is a course authoring pattern. A `boss: true` flag on exercises could trigger special celebration art. The infrastructure is literally adding one boolean and a different ASCII art path.

### Daily Challenge
- **What it enables:** `learnlocal daily` — deterministic pseudo-random exercise based on the date. Same exercise for everyone worldwide on the same day. People compare approaches. Creates a daily ritual.
- **What's blocking it:** Needs a selection algorithm (hash date → pick exercise from completed courses) and a small CLI subcommand. No server needed — the "same for everyone" comes from deterministic hashing.

### Reverse Engineering / Output-First Exercises
- **What it enables:** Given expected output + test cases, figure out the hidden function. Like Wordle but for code. "This function returns 1, 1, 2, 3, 5, 8... what is it?" The prompt shows only inputs and outputs. Student writes the implementation.
- **What's blocking it:** Nothing — this is a `type: write` exercise where the prompt gives input/output pairs and the starter is an empty function. Pure course authoring.

---

## The Biggest Single Unlock

`type: command` is now shipped. The environment engine is more powerful than the current courses demand. Services, ports, filesystem setup, teardown, state assertions, multi-file, persistent sandbox — that's an incident simulator, a protocol lab, and a systems playground hiding inside a tutorial framework.

---

## The Thread

These are experiences, not features. Nobody posts "I used a platform with adaptive difficulty." People post:
- "An AI tried to break my code 847 times and couldn't"
- "Look at my memory corruption visualized in my terminal"
- "I just watched myself become a better programmer across 55 exercises"

The philosophy IS the feature. Everything Reddit complains about in learning platforms — accounts, subscriptions, cloud dependencies, telemetry — LearnLocal just doesn't do any of it.
