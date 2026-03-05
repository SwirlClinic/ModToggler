# Roadmap: ModToggler

## Overview

ModToggler ships in four phases. The foundation is built first: game configuration, the file operations infrastructure (atomic file moves, transaction journaling for crash recovery, startup integrity scan, cross-drive handling). The core mod loop ships second — import from zip, toggle on/off, conflict detection — everything that delivers the core product value. Profiles ship third as a power-user layer on top of a stable toggle. Loose-file game support ships last, as it's a secondary use case with higher complexity and no dependency on profiles.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Foundation** - Game configuration, staging setup, atomic file ops infrastructure, reliability primitives
- [ ] **Phase 2: Core Mod Loop** - Import from zip, toggle on/off, conflict detection
- [x] **Phase 3: Profiles** - Named per-game profiles, save/load mod configuration sets (completed 2026-03-05)
- [ ] **Phase 4: Loose-File Games** - Mishmash game support with manual file tagging

## Phase Details

### Phase 1: Foundation
**Goal**: Users can configure games and the app has the infrastructure to move files reliably
**Depends on**: Nothing (first phase)
**Requirements**: GAME-01, GAME-02, GAME-03, GAME-04, TOGGLE-04, TOGGLE-06, RELIAB-01, RELIAB-02, RELIAB-03
**Success Criteria** (what must be TRUE):
  1. User can add, edit, and remove games with a name and mod directory path
  2. User can select a game from a list and see its (empty) mod view
  3. App-managed staging directory (~/.modtoggler/games/[game]/staging/) is created on game add
  4. App recovers correctly after a mid-toggle crash using the transaction journal (no orphaned files)
  5. App reports clear errors when file operations fail due to permissions, missing files, or cross-drive moves
**Plans**: 5 plans

Plans:
- [x] 01-01-PLAN.md — Scaffold project, install dependencies, SQLite migrations, tauri-specta wiring, test infrastructure
- [x] 01-02-PLAN.md — AppError type hierarchy, file_ops service (atomic move + cross-drive fallback), journal service
- [x] 01-03-PLAN.md — Game management Tauri commands (add/remove/edit/list) + integrity scan command
- [x] 01-04-PLAN.md — React app shell, game selector, game CRUD forms, empty mod view, useGames hooks
- [x] 01-05-PLAN.md — IntegrityAlert component, useIntegrityScan hook, visual end-to-end checkpoint

### Phase 2: Core Mod Loop
**Goal**: Users can import mods and toggle them on/off with conflict warnings
**Depends on**: Phase 1
**Requirements**: IMPORT-01, IMPORT-02, IMPORT-03, IMPORT-04, IMPORT-05, IMPORT-06, TOGGLE-01, TOGGLE-02, TOGGLE-03, TOGGLE-05, TOGGLE-07, CONFLICT-01, CONFLICT-02, CONFLICT-03
**Success Criteria** (what must be TRUE):
  1. User can import a mod from a .zip file and see it in the mod list with its file manifest
  2. User can toggle a mod on/off with one click, with state persisting across app restarts
  3. PAK/ucas/utoc file triples are auto-grouped into a single logical mod at import
  4. Sub-mod option folders are detected at import and can be toggled independently
  5. Enabling a mod that conflicts with an already-enabled mod shows a conflict warning naming both mods and the overlapping files
**Plans**: 5 plans

Plans:
- [ ] 02-01-PLAN.md — Zip crate dependency, DB migrations for sub-mods, import service (extraction + sub-mod detection)
- [ ] 02-02-PLAN.md — Toggle service (enable/disable/delete), extended DB queries, conflict detection queries
- [ ] 02-03-PLAN.md — Tauri commands for all mod operations, React Query hooks (useMods, useImportMod, etc.)
- [ ] 02-04-PLAN.md — Frontend UI: ModList, ModCard, SubModOptions, ImportDialog, ConflictDialog, drag-and-drop
- [ ] 02-05-PLAN.md — Full test suite validation and end-to-end human verification checkpoint

### Phase 3: Profiles
**Goal**: Users can save and restore named mod configurations per game
**Depends on**: Phase 2
**Requirements**: PROFILE-01, PROFILE-02, PROFILE-03, PROFILE-04
**Success Criteria** (what must be TRUE):
  1. User can save the current mod enabled/disabled state as a named profile for a game
  2. User can load a saved profile and the app enables/disables mods to match it
  3. User can delete a profile
  4. Profiles are scoped per game — game A's profiles do not appear in game B's profile list
**Plans**: 2 plans

Plans:
- [ ] 03-01-PLAN.md — Profile backend: DB migration, CRUD queries, save/apply service, Tauri commands
- [ ] 03-02-PLAN.md — Profile frontend: React Query hooks, Zustand state, dropdown, save/manage dialogs, ModList integration

### Phase 4: Loose-File Games
**Goal**: Users can manage mods for games where mod files are scattered across the game root, with manual file tagging, destination path mapping, and the same toggle/conflict infrastructure as structured mods
**Depends on**: Phase 2
**Requirements**: LOOSE-01, LOOSE-02, LOOSE-03, LOOSE-04
**Success Criteria** (what must be TRUE):
  1. User can configure a game as loose-file (mishmash) mode
  2. User can import a mod for a loose-file game by manually tagging which files belong to it and specifying their destination paths
  3. Toggling a loose-file mod moves its tagged files to/from the staging folder the same way structured mods work
**Plans**: 3 plans

Plans:
- [ ] 04-01-PLAN.md — DB migration v8 (destination_path + mod_type), extended queries, toggle service extension, loose import helper
- [ ] 04-02-PLAN.md — Tauri commands for loose-file operations, bindings regeneration, React Query hooks
- [ ] 04-03-PLAN.md — Frontend UI: LooseImportDialog, FileMapTable, GameSelector badge, ModCard/ModList extensions, visual checkpoint

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 5/5 | Complete | 2026-03-05 |
| 2. Core Mod Loop | 4/5 | In Progress|  |
| 3. Profiles | 2/2 | Complete   | 2026-03-05 |
| 4. Loose-File Games | 0/3 | Not started | - |
