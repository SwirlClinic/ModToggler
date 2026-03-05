---
phase: 01-foundation
plan: "03"
subsystem: commands
tags: [tauri, specta, sqlx, sqlite, commands, ipc, bindings]

# Dependency graph
requires:
  - phase: 01-foundation-01
    provides: "Tauri v2 project skeleton with stub modules for commands and queries"
  - phase: 01-foundation-02
    provides: "AppError enum, file_ops service (same_volume, create_staging_dir), journal service (FilePair, IncompleteJournalEntry)"
provides:
  - "Five Tauri IPC commands: add_game, remove_game, edit_game, list_games, run_integrity_scan"
  - "DB query layer with sqlx: insert/list/update/delete games, list mods, list file_entries, scan journals"
  - "Record types: GameRecord, ModRecord, FileEntry, IntegrityScanResult with specta::Type"
  - "Typed TypeScript bindings (bindings.ts) for all 5 commands"
  - "SqlitePool managed state for Rust-side DB access"
affects: [01-04, 01-05, 02-toggle]

# Tech tracking
tech-stack:
  added: [sqlx 0.8, tempfile 3]
  patterns: [sqlx::SqlitePool as Tauri managed state for Rust commands, BigIntExportBehavior::Number for specta i64 export, scan_existing_mods for .pak/.ucas/.utoc detection]

key-files:
  created:
    - src-tauri/src/commands/games.rs
    - src-tauri/src/commands/integrity.rs
  modified:
    - src-tauri/src/db/queries.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/services/journal.rs
    - src-tauri/Cargo.toml
    - src/bindings.ts

key-decisions:
  - "Used sqlx::SqlitePool directly instead of tauri-plugin-sql for Rust commands -- tauri-plugin-sql is JS-facing only with pub(crate) methods"
  - "Configured BigIntExportBehavior::Number for specta i64 export since SQLite IDs are within safe JS number range"
  - "Added specta::Type derive to FilePair and IncompleteJournalEntry for IntegrityScanResult bindings"
  - "Used CARGO_MANIFEST_DIR for absolute bindings.ts export path"

patterns-established:
  - "Tauri commands take tauri::State<'_, SqlitePool> for DB access"
  - "sqlx::FromRow manual impl for bool fields stored as i64 in SQLite"
  - "AddGameResult carries cross_drive_warning and has_existing_mods metadata"
  - "Integrity scan returns empty results (not errors) on empty DB (PITFALL-5 guard)"

requirements-completed: [GAME-01, GAME-02, GAME-03, GAME-04, TOGGLE-04, TOGGLE-06, RELIAB-01]

# Metrics
duration: 9min
completed: 2026-03-04
---

# Phase 1 Plan 03: Tauri Commands Summary

**Five Tauri IPC commands (game CRUD + integrity scan) with sqlx query layer, specta bindings, and cross-drive/existing-mod detection**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-05T01:38:16Z
- **Completed:** 2026-03-05T01:47:34Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Five Tauri commands with specta annotations wired into collect_commands! and typed bindings
- DB query layer using sqlx directly (insert/list/update/delete games, list mods, list file entries, scan journals)
- AddGameResult carries cross_drive_warning and has_existing_mods for immediate UI feedback
- Integrity scan handles empty DB gracefully (no false positives on first launch)
- bindings.ts regenerated with all 5 typed commands and full type definitions

## Task Commits

Each task was committed atomically:

1. **Task 1: DB query functions and shared record types** - `4bf2955` (feat)
2. **Task 2: Game management commands and integrity scan command** - `91d58bb` (feat)

## Files Created/Modified
- `src-tauri/src/db/queries.rs` - GameRecord, ModRecord, FileEntry, IntegrityScanResult types + all SQL query functions via sqlx
- `src-tauri/src/commands/games.rs` - add_game, remove_game, edit_game, list_games Tauri commands
- `src-tauri/src/commands/integrity.rs` - run_integrity_scan Tauri command with PITFALL-5 guard
- `src-tauri/src/commands/mod.rs` - Re-exports games and integrity submodules
- `src-tauri/src/lib.rs` - 5 commands in collect_commands!, SqlitePool setup hook, BigInt config
- `src-tauri/src/services/journal.rs` - Added specta::Type derive to FilePair and IncompleteJournalEntry
- `src-tauri/Cargo.toml` - Added sqlx and tempfile dependencies
- `src/bindings.ts` - Auto-generated TypeScript bindings for all 5 commands

## Decisions Made
- Used sqlx::SqlitePool directly for Rust-side DB access instead of tauri-plugin-sql (which has pub(crate) methods only, designed for JS-side queries via IPC)
- Configured BigIntExportBehavior::Number for specta TypeScript export since i64 SQLite IDs stay within safe JS number range
- Added specta::Type derive to FilePair and IncompleteJournalEntry (required by IntegrityScanResult type hierarchy)
- Used CARGO_MANIFEST_DIR env for deterministic bindings.ts path resolution

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tauri-plugin-sql has no public Rust API for commands**
- **Found during:** Task 1 (DB query functions)
- **Issue:** Plan assumed `tauri_plugin_sql::Database` with public `.select()` and `.execute()` methods. Actual API has `DbPool` with `pub(crate)` methods only -- designed for JS-side IPC, not Rust commands.
- **Fix:** Added sqlx 0.8 as direct dependency, using sqlx::SqlitePool managed as Tauri state. DB queries use sqlx::query/query_as directly. tauri-plugin-sql retained for migrations and JS access.
- **Files modified:** src-tauri/Cargo.toml, src-tauri/src/db/queries.rs, src-tauri/src/lib.rs
- **Verification:** All 29 tests pass, cargo build succeeds
- **Committed in:** 4bf2955, 91d58bb

**2. [Rule 1 - Bug] specta BigInt export error on i64 fields**
- **Found during:** Task 2 (bindings generation)
- **Issue:** specta-typescript defaults to BigIntExportBehavior::Fail for i64 types, causing panic on export
- **Fix:** Configured Typescript::default().bigint(BigIntExportBehavior::Number) since SQLite rowids are safe for JS number
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** bindings.ts generated successfully with `number` type for all ID fields
- **Committed in:** 91d58bb

**3. [Rule 3 - Blocking] FilePair and IncompleteJournalEntry missing specta::Type**
- **Found during:** Task 1 (IntegrityScanResult type definition)
- **Issue:** IntegrityScanResult derives specta::Type but contains Vec<IncompleteJournalEntry> which contains Vec<FilePair> -- both need specta::Type
- **Fix:** Added specta::Type derive to both structs in services/journal.rs
- **Files modified:** src-tauri/src/services/journal.rs
- **Verification:** Compilation succeeds, bindings.ts includes FilePair and IncompleteJournalEntry types
- **Committed in:** 4bf2955

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All fixes necessary for compilation and correct bindings generation. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 5 Tauri commands ready for frontend consumption in Plans 04-05
- Typed TypeScript bindings available at src/bindings.ts
- SqlitePool wiring pattern established for future commands (Phase 2 toggle)
- 29 total tests passing across all modules

---
*Phase: 01-foundation*
*Completed: 2026-03-04*
