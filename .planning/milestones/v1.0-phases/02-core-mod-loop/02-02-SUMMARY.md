---
phase: 02-core-mod-loop
plan: 02
subsystem: database, services
tags: [sqlx, sqlite, toggle, journal, conflict-detection, sub-mods]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: journal service, file_ops, AppError, DB schema
  - phase: 02-core-mod-loop plan 01
    provides: sub_mods table migration, file_entries.sub_mod_id column
provides:
  - Mod CRUD queries (insert, get, list, update, delete)
  - Sub-mod queries (insert, get, list, update enabled/user_enabled)
  - File entry queries with sub_mod_id support
  - Conflict detection SQL query (overlapping relative_paths)
  - Journal async DB functions (begin_toggle, mark_file_done, complete_journal)
  - Toggle service (toggle_mod, toggle_sub_mod, delete_mod)
  - get_game query for fetching game by ID
affects: [02-core-mod-loop plan 03 (commands layer), 03-ui-and-ux (frontend hooks)]

# Tech tracking
tech-stack:
  added: []
  patterns: [journal-wrapped file moves, sub-mod user_enabled state preservation, in-memory SQLite test pool]

key-files:
  created: [src-tauri/src/services/toggle.rs]
  modified: [src-tauri/src/db/queries.rs, src-tauri/src/services/mod.rs]

key-decisions:
  - "Added get_game query (not in plan) as critical dependency for toggle service"
  - "Used platform-aware test assertions with normalize() for Windows backslash compatibility"
  - "Sub-mod files moved before parent files on disable, after parent files on enable"

patterns-established:
  - "test_pool() helper: in-memory SQLite with PRAGMA foreign_keys + all migrations for async DB tests"
  - "journal_move_files(): reusable journal-wrapped file move pattern for both mod and sub-mod toggles"
  - "build_file_pairs(): pure function for constructing FilePair lists from FileEntry + base paths"

requirements-completed: [TOGGLE-01, TOGGLE-02, TOGGLE-03, TOGGLE-05, TOGGLE-07, CONFLICT-01, CONFLICT-02]

# Metrics
duration: 6min
completed: 2026-03-05
---

# Phase 02 Plan 02: Toggle Service Summary

**Journal-wrapped toggle service with mod CRUD queries, sub-mod state preservation, and SQL-based conflict detection**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-05T16:04:44Z
- **Completed:** 2026-03-05T16:10:20Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Full mod lifecycle queries: insert, get, list (ordered by enabled+name), update enabled, delete with cascade
- Sub-mod queries tracking both effective (enabled) and user-intent (user_enabled) states separately
- Conflict detection via SQL join on file_entries.relative_path for same-game enabled mods
- Journal async DB functions (begin_toggle, mark_file_done read-modify-write, complete_journal)
- Toggle service with parent->sub-mod cascade: disable preserves user_enabled, enable restores user_enabled=true sub-mods
- Delete mod removes files from whichever location (staging or game dir) and cleans up empty directories
- 28 new tests (21 async DB tests + 7 toggle unit tests), all passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend DB queries** - `e86ef78` (feat)
2. **Task 2: Create toggle service** - `928d54c` (feat)

## Files Created/Modified
- `src-tauri/src/db/queries.rs` - Added SubModRecord, ConflictInfo types; 13 new query functions; 21 async tests
- `src-tauri/src/services/toggle.rs` - Toggle service: toggle_mod, toggle_sub_mod, delete_mod, build_file_pairs
- `src-tauri/src/services/mod.rs` - Added `pub mod toggle;`

## Decisions Made
- Added `get_game` query not in the original plan -- required by toggle service to look up game's mod_dir (Rule 2: missing critical functionality)
- Used `normalize()` helper in tests to convert backslashes to forward slashes for Windows compatibility
- Sub-mod files are moved before parent files on disable (prevents orphaned sub-mod files in game dir) and after parent files on enable
- Empty file pairs list short-circuits without creating a journal entry (avoids empty journal noise)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added get_game query**
- **Found during:** Task 1 (DB queries)
- **Issue:** Toggle service needs to look up game.mod_dir to build file paths, but no get_game query existed
- **Fix:** Added `get_game(pool, game_id)` query returning GameRecord or GameNotFound error
- **Files modified:** src-tauri/src/db/queries.rs
- **Verification:** Used by toggle service in Task 2; tested implicitly through all toggle DB tests
- **Committed in:** e86ef78 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed Windows path separator in test assertions**
- **Found during:** Task 2 (toggle service tests)
- **Issue:** `Path::join` on Windows produces backslashes, but tests compared against forward-slash strings
- **Fix:** Added `normalize()` test helper; kept OS-native paths in production code since `file_ops::move_file` takes `&Path`
- **Files modified:** src-tauri/src/services/toggle.rs
- **Verification:** All 7 toggle tests pass on Windows
- **Committed in:** 928d54c (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 bug)
**Impact on plan:** Both auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Toggle service ready for Tauri command layer (02-03 plan)
- All query functions exported and available for commands
- ConflictInfo type has specta::Type derive for TypeScript bindings generation

---
*Phase: 02-core-mod-loop*
*Completed: 2026-03-05*
