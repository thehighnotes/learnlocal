# LearnLocal — Testing Guide

Manual testing guide for all community platform features. Run through these after code changes to verify end-to-end behavior.

---

## Prerequisites

```bash
# Build all feature variants
cargo build                      # Default (client)
cargo build --features server    # Server
cargo build --features llm       # AI features
cargo build --features author    # Course Designer

# Run automated tests (must all pass before manual testing)
cargo test
cargo test --features server
cargo test --features llm
```

---

## 1. TUI Core (Home Screen)

| Step | Action | Expected |
|------|--------|----------|
| 1.1 | `cargo run` | Home screen loads, courses listed grouped by language |
| 1.2 | Arrow keys | Navigate course list, right panel shows lessons |
| 1.3 | `[s]` | Settings screen opens |
| 1.4 | `[h]` | How To screen opens |
| 1.5 | `[w]` | Welcome Tour opens |
| 1.6 | `[t]` | Stats screen opens |
| 1.7 | `[p]` | Progress screen for selected course |
| 1.8 | `[b]` | **Browse screen opens** (community) |
| 1.9 | `Enter` on course | Course starts, first lesson content shown |
| 1.10 | `q` | Quit from any screen |

## 2. Course Flow

| Step | Action | Expected |
|------|--------|----------|
| 2.1 | Start any course | Lesson content renders as markdown |
| 2.2 | `Space` | Progressive reveal advances sections |
| 2.3 | `Enter` after lesson | First exercise prompt shown |
| 2.4 | `[e]` | Inline editor opens with starter code |
| 2.5 | `[E]` (Shift+e) | External $EDITOR opens |
| 2.6 | Write solution, `Enter` | Code executes, result shown (pass/fail) |
| 2.7 | `[h]` on exercise | Hints reveal progressively |
| 2.8 | Complete all exercises | Lesson recap with stats |
| 2.9 | Complete all lessons | Course completion celebration |

## 3. Staged Exercises

Test with a course that has stages (or create one with `stages:` in exercise YAML).

| Step | Action | Expected |
|------|--------|----------|
| 3.1 | Start a staged exercise | Stage indicator shows "Stage 1 of N" in breadcrumb |
| 3.2 | Pass stage 1 | StageComplete screen with animation, press Enter for next stage |
| 3.3 | Code carries forward | Your code from stage 1 is pre-loaded in stage 2 |
| 3.4 | Hints are stage-specific | `[h]` shows hints for current stage only |
| 3.5 | Complete all stages | Exercise marked complete, progress updated |
| 3.6 | Resume mid-stage | Quit and restart — resumes at current stage with draft saved |

## 4. Shell Mode (Command Exercises)

Test with Linux Fundamentals, Git Time Travel, or any `type: command` exercise.

| Step | Action | Expected |
|------|--------|----------|
| 4.1 | Start a command exercise | Interactive shell opens with prompt |
| 4.2 | Run commands | Shell executes commands, output displayed |
| 4.3 | `exit` or Ctrl+D | Shell exits, validation runs against environment state |
| 4.4 | Pass/fail feedback | Clear indication of what passed/failed |

## 5. SQL Exercises

Test with SQL (SQLite) Fundamentals course.

| Step | Action | Expected |
|------|--------|----------|
| 5.1 | Start SQL course | No external tools needed (embedded SQLite) |
| 5.2 | Write SQL query | Executes against built-in SQLite, output shown |
| 5.3 | `learnlocal doctor` | Shows SQLite as embedded, no install needed |

## 6. Browse (Community)

### 6a. CLI Browse

| Step | Action | Expected |
|------|--------|----------|
| 6a.1 | `cargo run -- browse` | Fetches registry, shows source (Remote/Cached/Offline) |
| 6a.2 | `cargo run -- browse --search python` | Filtered results |
| 6a.3 | With no network | Falls back to cache or shows "Offline" gracefully |

### 6b. TUI Browse

| Step | Action | Expected |
|------|--------|----------|
| 6b.1 | Press `[b]` from home | Browse screen renders with title bar, search bar, course list |
| 6b.2 | `j`/`k` or arrows | Navigate course list, detail panel updates |
| 6b.3 | `/` | Search bar activates, type to filter |
| 6b.4 | `Esc` in search | Exit search mode |
| 6b.5 | `s` | Sort cycles: A-Z → Newest → Exercises |
| 6b.6 | `c` | Clear search query |
| 6b.7 | Detail panel | Shows name, version, author, description, language, lessons, exercises, license, tags, platform, ratings, fork info |
| 6b.8 | Installed course | Green checkmark in list, "Installed" in detail |
| 6b.9 | `Esc` | Returns to home screen |

## 7. Install

| Step | Action | Expected |
|------|--------|----------|
| 7.1 | `cargo run -- install <course-id>` | Download progress shown step by step |
| 7.2 | Checksum verification | "Verifying checksum... ✓" in output |
| 7.3 | Course validation | "Validating course... ✓" in output |
| 7.4 | Success | Green "Installed" message with `learnlocal start <id>` hint |
| 7.5 | Already installed | Yellow warning, no re-download |
| 7.6 | Invalid course ID | Red error with "use learnlocal browse" hint |
| 7.7 | TUI: `[d]` on browse | Downloads selected course, status shown in key bar |
| 7.8 | After install | Course appears on home screen immediately |

## 8. Login / Logout

**Prerequisite**: GitHub OAuth App must be configured on the server with `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` environment variables.

| Step | Action | Expected |
|------|--------|----------|
| 8.1 | `cargo run -- login` | Prints verification URL + user code |
| 8.2 | Open URL in browser | GitHub device authorization page |
| 8.3 | Enter code | GitHub confirms authorization |
| 8.4 | CLI polls and succeeds | "Logged in as <username>" message |
| 8.5 | Check config | `~/.config/learnlocal/config.yaml` has `auth_token` under `community` |
| 8.6 | `cargo run -- logout` | Token removed, "Logged out" message |
| 8.7 | Login without server | Clean error message |

## 9. Rate & Review

**Prerequisite**: Logged in (`learnlocal login`), course exists on server.

| Step | Action | Expected |
|------|--------|----------|
| 9.1 | `cargo run -- rate <id> 5` | "Rated <id> with 5 stars" |
| 9.2 | `cargo run -- rate <id> 3` | Updates existing rating (upsert) |
| 9.3 | `cargo run -- rate <id> 0` | Error: "Stars must be between 1 and 5" |
| 9.4 | `cargo run -- rate <id> 6` | Error: "Stars must be between 1 and 5" |
| 9.5 | Rate without login | Error: "Not logged in" |
| 9.6 | `cargo run -- review <id> "Great course!"` | "Review submitted" |
| 9.7 | Review same course again | Error: "already reviewed" |
| 9.8 | Review > 2000 chars | Error: "between 1 and 2000 characters" |
| 9.9 | Browse after rating | Rating stars visible in detail panel |

## 10. Publish

**Prerequisite**: Logged in, valid course directory.

| Step | Action | Expected |
|------|--------|----------|
| 10.1 | `cargo run -- author publish courses/cpp-fundamentals --dry-run` | Pre-flight checks run, package created, "Dry run complete" |
| 10.2 | Pre-flight: missing description | Red ✗ on "Description" check |
| 10.3 | Pre-flight: validation fails | Red ✗ on "Validation" check with details |
| 10.4 | `cargo run -- author publish courses/cpp-fundamentals` | Full publish: checks → package → upload |
| 10.5 | Upload success | "Published! Course will appear after review." |
| 10.6 | Publish same version again | Error: "already exists" (PRIMARY KEY conflict) |
| 10.7 | Publish new version | Bump version in course.yaml, publish succeeds |
| 10.8 | Publish by different user | Error: "owned by <owner>" with fork guidance |
| 10.9 | Publish without login | Error: "Not logged in" |
| 10.10 | Package contents | Verify tar.gz excludes .git/, .learnlocal-studio.json, __pycache__/ |

## 11. Server (--features server)

| Step | Action | Expected |
|------|--------|----------|
| 11.1 | `cargo run --features server -- server --port 3001 --data-dir /tmp/ll-test/data --packages-dir /tmp/ll-test/pkg` | Server starts, prints listen address |
| 11.2 | `curl localhost:3001/health` | `{"status":"ok"}` |
| 11.3 | `curl localhost:3001/api/v1/courses` | JSON with `version`, `updated_at`, empty `courses` |
| 11.4 | Publish a course to it | Course appears in listing (after approval) |
| 11.5 | `curl localhost:3001/api/v1/courses/<id>` | Course detail with reviews |
| 11.6 | Download package | `curl localhost:3001/api/v1/packages/<file>.tar.gz` returns tarball |
| 11.7 | Kill and restart | Data persists (SQLite on disk) |

## 12. Provenance & Fork Chain

| Step | Action | Expected |
|------|--------|----------|
| 12.1 | Publish original course as User A | `owner_github` = User A |
| 12.2 | User B publishes new version of same ID | Error: "owned by User A" |
| 12.3 | User B publishes fork with different ID | Success, `forked_from` shows User A's course |
| 12.4 | Browse the fork in TUI | Detail panel shows "Forked from <original> v1.0.0 by User A" |
| 12.5 | User C forks User B's fork | Chain preserved: C → B → A |

## 13. Offline Behavior

| Step | Action | Expected |
|------|--------|----------|
| 13.1 | `learnlocal browse` (first run, online) | Registry fetched and cached |
| 13.2 | Disconnect network | — |
| 13.3 | `learnlocal browse` (offline) | Uses cache, shows "Cached (Xh ago)" |
| 13.4 | `learnlocal` (home screen) | Works fully offline with local courses |
| 13.5 | `learnlocal install` (offline) | Fails gracefully with network error |

## 14. Author CLI

| Step | Action | Expected |
|------|--------|----------|
| 14.1 | `cargo run -- author run-solution courses/cpp-fundamentals --lesson variables --exercise declare` | Runs solution, shows pass/fail |
| 14.2 | `cargo run -- author run-all-solutions courses/cpp-fundamentals` | Batch runs all solutions |
| 14.3 | `cargo run -- author run-all-solutions courses/cpp-fundamentals --update` | Auto-updates expected_output in YAML |
| 14.4 | `cargo run -- validate courses/cpp-fundamentals` | Structural validation passes |
| 14.5 | `cargo run -- validate courses/cpp-fundamentals --run-solutions` | Validates + runs all solutions |

## 15. Course Designer (--features author)

| Step | Action | Expected |
|------|--------|----------|
| 15.1 | `cargo run --features author -- author design` | Server starts, browser opens |
| 15.2 | Create new project | Welcome screen → create → scaffold created |
| 15.3 | Edit course metadata | Tab-based editor, auto-save |
| 15.4 | Add/reorder lessons | Drag-and-drop, hierarchy updates |
| 15.5 | Add/edit exercises | Form-based editing in tabs |
| 15.6 | Live preview | Terminal preview runs exercise in real time |
| 15.7 | AI chat | Connects to Ollama for authoring assistance |

## 16. Edge Cases & Regression

| Step | Action | Expected |
|------|--------|----------|
| 16.1 | Terminal < 80x24 | Graceful error message, doesn't crash |
| 16.2 | Ctrl+C during exercise | Draft saved, terminal restored |
| 16.3 | `learnlocal doctor` | All checks run, missing tools shown with install hints |
| 16.4 | Config parse error | Yellow warning, falls back to defaults |
| 16.5 | Progress backup on reset | `progress.json.bak` created before reset |
| 16.6 | `NO_COLOR=1 learnlocal list` | No ANSI colors in output |
| 16.7 | Course with `platform: linux` on macOS | Course shown as blocked, can't start |
