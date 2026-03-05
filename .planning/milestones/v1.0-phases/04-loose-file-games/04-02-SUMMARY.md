---
phase: 04-loose-file-games
plan: 02
subsystem: commands, hooks, bindings
tags: [tauri, specta, react-query, loose-file, ipc]

# Dependency graph
requires:
  - phase: 04-loose-file-games
    provides: migration v8, loose query functions, toggle branching, copy_files_to_staging
provides:
  - "import_loose_files, import_loose_zip, add_files_to_mod, remove_file_from_mod Tauri commands"
  - "LooseFileInput type in bindings.ts"
  - "useImportLooseFiles, useImportLooseZip, useAddFilesToMod, useRemoveFileFromMod React Query hooks"
  - "Updated ModRecord (mod_type) and FileEntry (destination_path) types in bindings"
affects: [04-03]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Loose-file commands reuse copy_files_to_staging for consistent staging", "remove_file_from_mod checks enabled state to find file in game dir vs staging"]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/mods.rs
    - src-tauri/src/lib.rs
    - src/bindings.ts
    - src/hooks/useMods.ts

key-decisions:
  - "Manual bindings update to match Rust types (mod_type on ModRecord, destination_path on FileEntry) since tauri-specta export runs at app startup not build time"
  - "import_loose_zip uses std::env::temp_dir for zip extraction (tempfile crate is dev-dependency only)"
  - "remove_file_from_mod queries file_entry directly via sqlx (no dedicated get_file_entry query function needed)"

patterns-established:
  - "Loose-file commands follow same staging pattern as structured: slug-named subdir under game staging_dir"
  - "All four loose hooks follow established unwrap/toast/invalidation pattern from useMods.ts"

requirements-completed: [LOOSE-01, LOOSE-02, LOOSE-03, LOOSE-04]

# Metrics
duration: 5min
completed: 2026-03-05
---

# Phase 4 Plan 2: Loose-File Commands & Hooks Summary

**Four Tauri IPC commands for loose-file import/management with typed bindings and React Query mutation hooks**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-05T20:35:52Z
- **Completed:** 2026-03-05T20:40:31Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Four new Tauri commands: import_loose_files, import_loose_zip, add_files_to_mod, remove_file_from_mod
- TypeScript bindings updated with LooseFileInput type and all new command functions, plus mod_type/destination_path fields
- Four React Query hooks following established unwrap/toast/invalidation pattern
- All 80 Rust tests pass, TypeScript compiles clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Tauri commands for loose-file operations + bindings regeneration** - `1098cd2` (feat)
2. **Task 2: React Query hooks for loose-file operations** - `08fcce7` (feat)

## Files Created/Modified
- `src-tauri/src/commands/mods.rs` - LooseFileInput type + 4 new commands, slug_from_name made pub(crate)
- `src-tauri/src/lib.rs` - Registered 4 new commands in collect_commands! macro
- `src/bindings.ts` - New command functions, LooseFileInput type, updated ModRecord/FileEntry types
- `src/hooks/useMods.ts` - useImportLooseFiles, useImportLooseZip, useAddFilesToMod, useRemoveFileFromMod hooks

## Decisions Made
- Manual bindings update since tauri-specta export runs at app startup (not cargo build). Updated ModRecord to include mod_type, FileEntry to include destination_path.
- Used std::env::temp_dir for import_loose_zip temp extraction dir (tempfile crate only available as dev-dependency)
- remove_file_from_mod queries file_entry directly with sqlx::query_as rather than adding a dedicated query function

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed pool type inference in remove_file_from_mod**
- **Found during:** Task 1 (remove_file_from_mod command)
- **Issue:** sqlx::query_as with tauri::State<SqlitePool> needed explicit pool dereference for type inference
- **Fix:** Added `let pool_ref: &SqlitePool = &pool;` before the query call
- **Files modified:** src-tauri/src/commands/mods.rs
- **Verification:** cargo build succeeds
- **Committed in:** 1098cd2

**2. [Rule 3 - Blocking] Replaced tempfile crate usage with std::env::temp_dir**
- **Found during:** Task 1 (import_loose_zip command)
- **Issue:** tempfile crate only in dev-dependencies, not available in production builds
- **Fix:** Used std::env::temp_dir() with process ID for unique temp directory
- **Files modified:** src-tauri/src/commands/mods.rs
- **Verification:** cargo build succeeds
- **Committed in:** 1098cd2

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary for compilation. No scope change.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Tauri commands and React Query hooks ready for Plan 03 (frontend UI)
- Bindings fully typed for LooseFileInput, ModRecord.mod_type, FileEntry.destination_path
- 80 Rust tests green, TypeScript compiles clean

---
*Phase: 04-loose-file-games*
*Completed: 2026-03-05*
