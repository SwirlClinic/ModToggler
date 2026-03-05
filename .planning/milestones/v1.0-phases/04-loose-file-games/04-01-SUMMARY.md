---
phase: 04-loose-file-games
plan: 01
subsystem: database, services
tags: [sqlite, migration, toggle, loose-file, import]

# Dependency graph
requires:
  - phase: 02-core-mod-loop
    provides: toggle service, file_entries table, journal infrastructure
provides:
  - "Migration v8: destination_path on file_entries, mod_type on mods"
  - "insert_mod_with_type, insert_file_entry_with_destination, delete_file_entry, add_file_entries_to_mod queries"
  - "build_loose_file_pairs for per-file destination path toggle"
  - "toggle_mod branching on mod_type (loose vs structured)"
  - "copy_files_to_staging with collision handling"
  - "check_conflicts CTE supporting both structured and loose mods"
affects: [04-02, 04-03]

# Tech tracking
tech-stack:
  added: []
  patterns: ["mod_type column on mods table for per-mod type tracking", "CTE-based conflict detection with effective_path", "build_loose_file_pairs for per-file destination paths"]

key-files:
  created: []
  modified:
    - src-tauri/src/db/migrations.rs
    - src-tauri/src/db/queries.rs
    - src-tauri/src/services/toggle.rs
    - src-tauri/src/services/import.rs

key-decisions:
  - "mod_type column on mods table (not inferred from file_entries) for explicit per-mod type tracking"
  - "check_conflicts uses CTE with effective_path: structured=relative_path, loose=destination_path/relative_path"
  - "copy_files_to_staging uses numeric suffix (_1, _2) for filename collisions"
  - "insert_mod delegates to insert_mod_with_type with 'structured' default for backward compat"

patterns-established:
  - "Loose mod path computation: staging_base/filename -> game_root/destination_path/filename"
  - "mod_type branching in toggle_mod and delete_mod for dual path semantics"

requirements-completed: [LOOSE-01, LOOSE-03, LOOSE-04]

# Metrics
duration: 5min
completed: 2026-03-05
---

# Phase 4 Plan 1: Loose-File Backend Summary

**Migration v8 with destination_path/mod_type columns, loose toggle logic with per-file destination paths, and copy_files_to_staging with collision handling**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-05T20:27:55Z
- **Completed:** 2026-03-05T20:33:18Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Migration v8 adds destination_path to file_entries and mod_type to mods with CHECK constraint
- ModRecord and FileEntry structs extended; all SELECT queries updated; new query functions for loose-file CRUD
- check_conflicts updated with CTE computing effective_path for both structured and loose mods
- build_loose_file_pairs computes correct staging/game paths using per-file destination_path
- toggle_mod branches on mod_type; loose path auto-creates destination directories on enable
- delete_mod handles loose file game-side path computation
- copy_files_to_staging copies files with numeric suffix collision handling
- All 80 Rust tests pass (37 DB + 30 service + 13 other)

## Task Commits

Each task was committed atomically:

1. **Task 1: DB migration v8 + extended record types + query functions** - `d44e71c` (feat)
2. **Task 2: Toggle service extension + loose import helper** - `b7fb7d4` (feat)

## Files Created/Modified
- `src-tauri/src/db/migrations.rs` - Migration v8 adding destination_path and mod_type columns
- `src-tauri/src/db/queries.rs` - Extended ModRecord/FileEntry, new query functions, updated check_conflicts
- `src-tauri/src/services/toggle.rs` - build_loose_file_pairs, mod_type branching in toggle_mod/delete_mod
- `src-tauri/src/services/import.rs` - copy_files_to_staging with collision handling

## Decisions Made
- mod_type as explicit column on mods (not inferred from file_entries) for reliable per-mod type tracking
- check_conflicts uses a CTE with effective_path computation: structured mods use relative_path, loose mods use destination_path/relative_path
- copy_files_to_staging uses _N numeric suffix pattern for filename collisions
- insert_mod preserved as backward-compatible wrapper around insert_mod_with_type

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow checker issue in copy_files_to_staging**
- **Found during:** Task 2 (copy_files_to_staging implementation)
- **Issue:** HashMap entry borrow conflicted with immutable contains_key check in collision loop
- **Fix:** Switched from HashMap<String, u32> to HashSet<String> tracking, using separate counter variable
- **Files modified:** src-tauri/src/services/import.rs
- **Verification:** cargo test passes, collision test verifies correct behavior
- **Committed in:** b7fb7d4

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor implementation fix for Rust borrow rules. No scope change.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Backend data layer and service logic complete for loose-file mods
- Ready for Plan 02 (commands + frontend) to build on these query functions and toggle logic
- All existing structured mod behavior verified unchanged (80 tests green)

---
*Phase: 04-loose-file-games*
*Completed: 2026-03-05*
