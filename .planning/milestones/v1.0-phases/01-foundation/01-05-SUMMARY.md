---
phase: 01-foundation
plan: "05"
subsystem: ui
tags: [react, tanstack-query, integrity-scan, dark-theme, sqlite-migrations]

# Dependency graph
requires:
  - phase: 01-foundation/04
    provides: "App shell, GameSelector, SettingsPanel, EmptyModView components"
  - phase: 01-foundation/03
    provides: "Tauri commands including run_integrity_scan, typed bindings"
provides:
  - "useIntegrityScan hook for startup integrity scanning"
  - "IntegrityAlert banner component for surfacing scan issues"
  - "Dark theme activated on app shell"
  - "SQLite migrations running on sqlx pool at startup"
affects: [02-mod-management, reliability, toggle-operations]

# Tech tracking
tech-stack:
  added: []
  patterns: ["useQuery with staleTime:Infinity for one-shot startup scan", "raw_sql migrations on sqlx pool"]

key-files:
  created:
    - src/hooks/useIntegrityScan.ts
    - src/components/IntegrityAlert.tsx
    - src/components/IntegrityAlert.test.tsx
  modified:
    - index.html
    - src-tauri/src/lib.rs

key-decisions:
  - "Run SQLite migrations directly on sqlx pool via raw_sql in setup block (tauri_plugin_sql migrations only run on JS-facing connection)"
  - "Set dark class on html element statically (dark-only app, no theme toggle needed for v1)"

patterns-established:
  - "Startup scan pattern: useQuery with staleTime:Infinity for one-shot queries"
  - "Alert banner pattern: dismissed state + conditional render based on scan result"

requirements-completed: [RELIAB-01, RELIAB-02, RELIAB-03]

# Metrics
duration: 8min
completed: 2026-03-05
---

# Phase 01 Plan 05: Integrity Scan UI and Phase 1 Verification Summary

**useIntegrityScan hook and IntegrityAlert banner component with dark theme fix and sqlx migration bootstrap**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-05T03:34:12Z
- **Completed:** 2026-03-05T03:42:00Z
- **Tasks:** 2 (1 auto + 1 checkpoint)
- **Files modified:** 5

## Accomplishments
- useIntegrityScan hook calls runIntegrityScan on mount via TanStack Query, stores result
- IntegrityAlert renders nothing on clean scan, yellow banner with issue descriptions when problems found
- Dismiss button hides alert from view
- Fixed dark theme not activating (missing class="dark" on html element)
- Fixed "no table games" error by running migrations directly on sqlx pool at startup
- 4 IntegrityAlert tests covering clean scan, journals, missing mods, and dismiss

## Task Commits

Each task was committed atomically:

1. **Task 1: useIntegrityScan hook and IntegrityAlert component** - `13dfd00` (feat)
2. **Task 2 fix: Dark theme and database migration on startup** - `8191dcf` (fix)

**Plan metadata:** (pending)

## Files Created/Modified
- `src/hooks/useIntegrityScan.ts` - Hook that calls runIntegrityScan on mount, returns scan result
- `src/components/IntegrityAlert.tsx` - Banner component showing integrity issues with dismiss
- `src/components/IntegrityAlert.test.tsx` - 4 Vitest tests for IntegrityAlert
- `index.html` - Added class="dark" to html element for dark theme
- `src-tauri/src/lib.rs` - Run SQLite migrations on sqlx pool in setup block

## Decisions Made
- Run SQLite migrations directly on sqlx pool via raw_sql in setup block. The tauri_plugin_sql migrations only run on the JS-facing connection, but Rust commands use a separate sqlx pool that needs its own migration execution.
- Set dark class on html element statically. This is a dark-only app for v1; no theme toggle is needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Dark theme not rendering**
- **Found during:** Checkpoint verification (human-reported)
- **Issue:** The html element lacked class="dark", so the CSS dark theme variables were never activated
- **Fix:** Added class="dark" to the html tag in index.html
- **Files modified:** index.html
- **Verification:** Rebuilds cleanly, dark theme variables now apply
- **Committed in:** 8191dcf

**2. [Rule 1 - Bug] Database tables not created for Rust commands**
- **Found during:** Checkpoint verification (human-reported "no table games" error)
- **Issue:** tauri_plugin_sql runs migrations on its own JS-facing connection, but the sqlx pool used by Rust commands connects to the same DB file without running migrations. Tables did not exist when Rust commands executed.
- **Fix:** Added migration execution loop using sqlx::raw_sql in the setup block, running the same DDL from db::migrations on the sqlx pool
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** cargo check passes, all 29 Rust tests pass
- **Committed in:** 8191dcf

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes essential for basic app functionality. No scope creep.

### User Feedback (Deferred)

**Mod structure field feedback:** User requested that the structure type be dynamic -- default to "manual tagging" with presets such as PAK/UCAS/UTOC groups. This is a design change beyond Phase 1 scope. Recorded here for future phases (likely Phase 2 mod management) to address.

## Issues Encountered
- Dark theme and database migration issues were caught during human verification checkpoint and fixed inline.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full Phase 1 foundation complete: Tauri app with game CRUD, integrity scanning, dark UI
- Ready for Phase 2 (mod management): mod import, toggle enable/disable, file operations
- Deferred item: mod structure field redesign (manual tagging + presets) for future phase

## Self-Check: PASSED

All 5 files verified present. Both commit hashes (13dfd00, 8191dcf) found in git log.

---
*Phase: 01-foundation*
*Completed: 2026-03-05*
