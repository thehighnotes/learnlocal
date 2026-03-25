# Phase 6 — Community Platform

## Strategy: Self-Hosted Server

Community platform running on a VPS at `https://learnlocal.aiquest.info`. Rust (axum) API server with SQLite database. GitHub OAuth for authentication. Course packages stored on disk, served directly.

> **Note**: The original design proposed a three-tier evolution (static GitHub → Cloudflare Worker → full platform). We skipped straight to a self-hosted Rust server which is simpler, more capable, and costs nothing extra (VPS was already available).

---

## Evolution Roadmap

### Tier 1 — Static Registry (launch, $0)

```
GitHub repo: thehighnotes/learnlocal-registry
├── registry.json          ← course index
└── README.md

Course packages stored as GitHub Release assets on author repos.
```

- Browse/search: client fetches `registry.json`, filters locally
- Download counts: GitHub API on release assets (free)
- Publish: author packages course → creates GitHub Release → submits PR to registry repo
- Review/quality gate: PR-based — maintainers review before merging to index
- Offline: `registry.json` cached locally in `~/.local/share/learnlocal/registry-cache.json`

### Tier 2 — Cloudflare Worker API ($0, free tier: 10M req/month)

```
community.learnlocal.dev/api/v1/
├── GET  /courses                     → registry + ratings merged
├── GET  /courses/:id                 → course details + stats
├── GET  /ratings/:course_id          → { stars, count }
├── POST /ratings/:course_id          → submit rating (GitHub OAuth)
├── GET  /reviews/:course_id          → text reviews
├── POST /reviews/:course_id          → submit review
└── GET  /stats/:course_id            → { downloads, completions }
```

Storage: Cloudflare D1 (free SQLite at edge)

```sql
CREATE TABLE ratings (
    course_id TEXT,
    github_user TEXT,
    stars INTEGER CHECK(stars BETWEEN 1 AND 5),
    created_at TEXT,
    UNIQUE(course_id, github_user)
);

CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    course_id TEXT,
    github_user TEXT,
    body TEXT,
    created_at TEXT
);
```

Auth: GitHub OAuth (authors/raters already have accounts).

### Tier 3 — Full Platform (when needed)

- Rust API (axum) on Fly.io / Railway
- PostgreSQL (Neon free tier)
- Org accounts, private courses, learning paths
- Payments via Stripe (paid premium courses)
- Same REST contract — client code unchanged

---

## Client Configuration

```yaml
# ~/.config/learnlocal/config.yaml
community:
  # Tier 1: static GitHub file
  registry_url: "https://raw.githubusercontent.com/thehighnotes/learnlocal-registry/main/registry.json"

  # Tier 2: swap to Worker API (same JSON shape)
  # registry_url: "https://community.learnlocal.dev/api/v1/courses"
```

---

## Registry Format (registry.json)

```json
{
  "version": 1,
  "updated_at": "2026-03-22T00:00:00Z",
  "courses": [
    {
      "id": "cpp-fundamentals",
      "name": "C++ Fundamentals",
      "version": "2.0.0",
      "author": "LearnLocal Community",
      "author_github": "thehighnotes",
      "description": "Learn C++ from scratch — variables, functions, pointers, structs, and memory management.",
      "language_id": "cpp",
      "language_display": "C++",
      "license": "CC-BY-4.0",
      "lessons": 8,
      "exercises": 58,
      "has_stages": false,
      "platform": null,
      "provision": "system",
      "tags": ["beginner", "cpp", "fundamentals"],
      "estimated_hours": 4,
      "download_url": "https://github.com/thehighnotes/learnlocal/releases/download/courses/cpp-fundamentals-2.0.0.tar.gz",
      "checksum": "sha256:abc123def456...",
      "published_at": "2026-03-22T00:00:00Z",
      "min_learnlocal_version": "0.2.0"
    }
  ]
}
```

---

## Course Package Format

```
course-cpp-fundamentals-2.0.0.tar.gz
├── manifest.json              ← package metadata (mirrors registry entry)
├── course.yaml
├── lessons/
│   ├── 01-hello-world/
│   │   ├── lesson.yaml
│   │   ├── content.md
│   │   └── exercises/
│   │       ├── 01-check-compiler.yaml
│   │       └── ...
│   └── ...
└── .learnlocal-studio.json    ← audit trail (optional, stripped on publish)
```

**manifest.json** (generated at package time):

```json
{
  "package_version": 1,
  "course_id": "cpp-fundamentals",
  "name": "C++ Fundamentals",
  "version": "2.0.0",
  "author": "LearnLocal Community",
  "license": "CC-BY-4.0",
  "language_id": "cpp",
  "lessons": 8,
  "exercises": 58,
  "checksum": "sha256:abc123...",
  "created_at": "2026-03-22T00:00:00Z",
  "learnlocal_min_version": "0.2.0"
}
```

---

## Publish Flow (from Studio)

```
Author clicks Publish in Course Designer
    │
    ├── Pre-flight checks:
    │   ├── All exercises pass validation?
    │   ├── All solutions run successfully?
    │   ├── Course has description, author, license?
    │   └── Show pre-flight report
    │
    ├── Package creation:
    │   ├── Generate manifest.json from course metadata
    │   ├── Create tar.gz of course directory (excluding .learnlocal-studio.json)
    │   └── Compute SHA-256 checksum
    │
    ├── Distribution (Tier 1):
    │   ├── Create GitHub Release on author's repo (via gh CLI)
    │   ├── Upload tar.gz as release asset
    │   ├── Generate registry entry JSON
    │   └── Open PR on learnlocal-registry repo (or copy to clipboard)
    │
    └── Post-publish:
        └── Show success + instructions for PR submission
```

---

## Browse Flow (TUI)

```
Student runs `learnlocal browse` or presses [b] on Home screen
    │
    ├── Fetch registry.json (or API endpoint)
    │   ├── Online: download + cache locally
    │   └── Offline: use cached version, show "Last updated: 2d ago"
    │
    ├── Display: Screen::Browse
    │   ├── Search bar (filter by name/language/tag)
    │   ├── Sort: popular / newest / alphabetical
    │   ├── Course cards: name, author, language, lessons, exercises, downloads
    │   └── [Enter] details, [d] download, [/] search, [Esc] back
    │
    ├── Course detail view:
    │   ├── Full description, lesson list, author, license
    │   ├── Download count (Tier 1) / stars + reviews (Tier 2)
    │   └── [d] Download & Install
    │
    └── Download flow:
        ├── Download tar.gz from download_url
        ├── Verify SHA-256 checksum
        ├── Extract to courses/ directory
        ├── Run toolcheck for required tools
        └── Show success: "Start with: learnlocal start {id}"
```

---

## Implementation Plan

### Client-side (what we build in Phase 6)

**New files:**
- `src/community/mod.rs` — module root
- `src/community/registry.rs` — fetch, cache, parse registry.json
- `src/community/package.rs` — create tar.gz, manifest.json, checksum
- `src/community/download.rs` — download, verify, extract course packages
- `src/ui/browse.rs` — TUI browse screen (Screen::Browse)

**Modified files:**
- `src/cli.rs` — add `Browse` command
- `src/main.rs` — wire `cmd_browse`
- `src/ui/screens.rs` — add `Screen::Browse`
- `src/ui/app.rs` — browse screen rendering + input handling
- `src/config.rs` — add `community.registry_url` config field
- `src/author/api.rs` — add `/api/publish` endpoint (package + registry entry generation)
- `src/web/app.js` — publish button in Studio

### Server-side (deferred to Tier 2)

- Cloudflare Worker for ratings/reviews API
- GitHub OAuth integration
- D1 database setup

---

## Feature Matrix by Tier

| Feature | Tier 1 (GitHub) | Tier 2 (Worker) | Tier 3 (Full) |
|---------|----------------|-----------------|----------------|
| Browse catalog | ✓ (static JSON) | ✓ (API) | ✓ |
| Download courses | ✓ (release assets) | ✓ | ✓ |
| Download counts | ✓ (GitHub API) | ✓ (tracked) | ✓ |
| Publish from Studio | ✓ (gh CLI + PR) | ✓ (API upload) | ✓ |
| Star ratings | ✗ | ✓ | ✓ |
| Text reviews | ✗ | ✓ | ✓ |
| Author profiles | ✗ (GitHub profile link) | ✓ | ✓ |
| Search/filter | ✓ (client-side) | ✓ (server-side) | ✓ |
| Offline browse | ✓ (cached registry) | ✓ | ✓ |
| Private courses | ✗ | ✗ | ✓ |
| Paid courses | ✗ | ✗ | ✓ |
| Learning paths | ✗ | ✗ | ✓ |
| Course dependencies | ✗ | ✓ | ✓ |
| Org accounts | ✗ | ✗ | ✓ |
| Analytics dashboard | ✗ | Basic | Full |

---

## Key Design Decisions

1. **Same REST contract across all tiers** — client code built once, backend swappable via config URL
2. **GitHub as Tier 1 backend** — zero cost, global CDN, built-in auth (gh CLI), familiar PR workflow
3. **Offline-first** — cached registry works without network after first fetch
4. **Decentralized packages** — courses live in author's own repos, not centralized storage
5. **PR-based quality gate** — maintainers review courses before they appear in the index
6. **Checksum verification** — SHA-256 on all downloads, fail on mismatch
7. **Audit trail preserved** — `.learnlocal-studio.json` tracks authorship but is stripped from published packages
