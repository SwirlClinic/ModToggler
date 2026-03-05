---
phase: 02-core-mod-loop
plan: 03
subsystem: api
tags: [tauri-commands, react-query, ipc, specta, hooks]

requires:
  - phase: 02-01
    provides: "Import service (extract_zip_to_staging, partition_files, has_recognized_mod_files)"
  - phase: 02-02
    provides: "Toggle service (toggle_mod, toggle_sub_mod, delete_mod) and DB queries (insert_mod, list_mods_for_game, check_conflicts, SubModRecord, ConflictInfo)"
provides:
  - "Tauri IPC commands for import_mod, list_mods, list_mod_files, list_sub_mods_cmd, toggle_mod_cmd, toggle_sub_mod_cmd, delete_mod_cmd, check_conflicts_cmd"
  - "React Query hooks: useMods, useModFiles, useSubMods, useCheckConflicts, useImportMod, useToggleMod, useToggleSubMod, useDeleteMod"
  - "Updated TypeScript bindings with ImportResult, SubModRecord, ConflictInfo types"
affects: [02-04, 02-05, 03-ui]

tech-stack:
  added: []
  patterns: ["spawn_blocking for sync zip extraction in async Tauri command", "mutation hooks with multi-query cache invalidation"]

key-files:
  created:
    - src-tauri/src/commands/mods.rs
    - src/hooks/useMods.ts
  modified:
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src/bindings.ts

key-decisions:
  - "import_mod creates mod-specific staging subdir using slug from mod name"
  - "Sub-mod file entries stored with full Option_*/relative_path to match disk layout"

patterns-established:
  - "Mod command pattern: thin wrappers delegating to service/query layer"
  - "Mutation hooks invalidate all affected query keys (mods, sub-mods, conflicts, mod-files)"

requirements-completed: [IMPORT-01, TOGGLE-01, TOGGLE-02, TOGGLE-03, TOGGLE-05, TOGGLE-07, CONFLICT-01, CONFLICT-02, CONFLICT-03]

duration: 3min
completed: 2026-03-05
---

# Phase 2 Plan 3: IPC Commands + React Hooks Summary

**Tauri IPC commands wiring import/toggle services to frontend via 8 registered commands and 8 React Query hooks with typed bindings**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-05T16:12:46Z
- **Completed:** 2026-03-05T16:15:23Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- All 8 mod-related Tauri commands created and registered (import, list, toggle, delete, conflicts)
- import_mod command orchestrates zip extraction via spawn_blocking, DB record creation, sub-mod partitioning
- React Query hooks for all operations with cache invalidation and toast notifications
- TypeScript bindings regenerated with ImportResult, SubModRecord, ConflictInfo types

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Tauri commands for import, toggle, delete, list, and conflicts** - `ee1d370` (feat)
2. **Task 2: Create React Query hooks for all mod operations** - `d9395e1` (feat)

## Files Created/Modified
- `src-tauri/src/commands/mods.rs` - All mod-related Tauri commands (import, list, toggle, delete, conflicts)
- `src-tauri/src/commands/mod.rs` - Added `pub mod mods` module declaration
- `src-tauri/src/lib.rs` - Registered 8 new commands in collect_commands!
- `src/bindings.ts` - Auto-regenerated with new command functions and types
- `src/hooks/useMods.ts` - React Query hooks for all mod operations

## Decisions Made
- import_mod creates a mod-specific staging subdirectory using a slug derived from the mod name (same slug_from_name approach as games.rs)
- Sub-mod file entries stored with full `Option_folder/relative_path` format to match the on-disk layout from extraction

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused Path import**
- **Found during:** Task 1 (Tauri commands)
- **Issue:** Compiler warning for unused `Path` import in mods.rs (only `PathBuf` needed)
- **Fix:** Removed `Path` from the import statement
- **Files modified:** src-tauri/src/commands/mods.rs
- **Verification:** Clean cargo build with no warnings
- **Committed in:** ee1d370 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial cleanup, no scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full IPC layer complete: frontend can call all mod operations via typed hooks
- Ready for UI implementation (mod list, import dialog, toggle controls)
- Conflict detection accessible for pre-toggle warnings

---
*Phase: 02-core-mod-loop*
*Completed: 2026-03-05*
