# Requirements: ModToggler

**Defined:** 2026-03-04
**Core Value:** Users can quickly toggle mods on and off without manually moving files, with confidence the app tracks what belongs to which mod.

## v1 Requirements

### Game Management

- [x] **GAME-01**: User can add a game with a name and mod directory path
- [x] **GAME-02**: User can remove a game from the app
- [x] **GAME-03**: User can edit a game's name and mod directory path
- [x] **GAME-04**: User can select a game to view its mod list

### Mod Import

- [x] **IMPORT-01**: User can import a mod from a .zip archive
- [x] **IMPORT-02**: App extracts .zip and records the full file manifest (every file path within the mod)
- [x] **IMPORT-03**: App auto-groups .pak/.ucas/.utoc files by base stem as a single logical mod
- [x] **IMPORT-04**: App detects sub-mod option folders (e.g. Option_ModName_Variant/) and registers them as independently toggleable options
- [ ] **IMPORT-05**: User can see which files belong to each mod
- [x] **IMPORT-06**: App validates zip contents to prevent path traversal (ZipSlip protection)

### Mod Toggling

- [ ] **TOGGLE-01**: User can toggle a mod on/off with one click
- [ ] **TOGGLE-02**: Disabling a mod moves all its files from the game directory to the app-managed staging folder (~/.modtoggler/disabled/[game]/)
- [ ] **TOGGLE-03**: Enabling a mod moves its files back from staging to the game directory
- [x] **TOGGLE-04**: Mod enabled/disabled state persists across app restarts
- [ ] **TOGGLE-05**: User can toggle individual sub-mod options on/off independently of the parent mod
- [x] **TOGGLE-06**: App uses a transaction journal to ensure file moves are atomic — crash mid-toggle can be recovered
- [ ] **TOGGLE-07**: User can permanently delete a mod and all its files (from staging or game directory)

### Conflict Detection

- [ ] **CONFLICT-01**: App detects when two enabled mods share overlapping files (same file stem)
- [ ] **CONFLICT-02**: App displays which specific mods conflict and over which files
- [ ] **CONFLICT-03**: Conflict warnings appear when enabling a mod that conflicts with an already-enabled mod

### Profiles

- [ ] **PROFILE-01**: User can save current mod configuration as a named profile
- [ ] **PROFILE-02**: User can load a saved profile, which enables/disables mods to match the profile state
- [ ] **PROFILE-03**: User can delete a saved profile
- [ ] **PROFILE-04**: Profiles are per-game

### Loose-File Games

- [ ] **LOOSE-01**: User can add a game configured for loose-file (mishmash) mod structure
- [ ] **LOOSE-02**: User can manually tag which files belong to a mod when importing for loose-file games
- [ ] **LOOSE-03**: User can specify destination paths for each file relative to the game root
- [ ] **LOOSE-04**: Toggling works the same way for loose-file mods (move to/from staging)

### Reliability

- [x] **RELIAB-01**: App performs integrity scan on startup to detect files moved outside the app (antivirus, manual moves)
- [x] **RELIAB-02**: App handles cross-drive file moves gracefully (copy+delete with progress indication when rename fails)
- [x] **RELIAB-03**: App provides clear error messages when file operations fail (permissions, missing files)

## v2 Requirements

### Enhanced Features

- **EXPORT-01**: User can export a profile as a JSON file for sharing
- **EXPORT-02**: User can import a profile from a JSON file
- **NOTES-01**: User can add notes to individual mods
- **MULTI-01**: Cross-platform support (macOS/Linux)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mod downloading from Nexus/TekkenMods | Scope creep — stay local-file-only; avoids API keys, login, rate limits |
| Auto-updating mods | Can break working setups silently; outside scope of file manager |
| Game launching / injection | Varies wildly per game; anti-cheat complications; separate domain |
| Virtual file system (VFS) | Enormous Windows complexity (USVFS, kernel hooks); file-move is simpler and sufficient |
| Load order management | PAK files don't merge — last writer wins; conflict detection is the right pattern |
| Mod merging | Requires understanding internal file formats; deeply game-specific |
| Cloud sync | Requires auth + cloud storage; niche use case; JSON export covers sharing |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| GAME-01 | Phase 1 | Complete |
| GAME-02 | Phase 1 | Complete |
| GAME-03 | Phase 1 | Complete |
| GAME-04 | Phase 1 | Complete |
| TOGGLE-04 | Phase 1 | Complete |
| TOGGLE-06 | Phase 1 | Complete |
| RELIAB-01 | Phase 1 | Complete |
| RELIAB-02 | Phase 1 | Complete |
| RELIAB-03 | Phase 1 | Complete |
| IMPORT-01 | Phase 2 | Complete |
| IMPORT-02 | Phase 2 | Complete |
| IMPORT-03 | Phase 2 | Complete |
| IMPORT-04 | Phase 2 | Complete |
| IMPORT-05 | Phase 2 | Pending |
| IMPORT-06 | Phase 2 | Complete |
| TOGGLE-01 | Phase 2 | Pending |
| TOGGLE-02 | Phase 2 | Pending |
| TOGGLE-03 | Phase 2 | Pending |
| TOGGLE-05 | Phase 2 | Pending |
| TOGGLE-07 | Phase 2 | Pending |
| CONFLICT-01 | Phase 2 | Pending |
| CONFLICT-02 | Phase 2 | Pending |
| CONFLICT-03 | Phase 2 | Pending |
| PROFILE-01 | Phase 3 | Pending |
| PROFILE-02 | Phase 3 | Pending |
| PROFILE-03 | Phase 3 | Pending |
| PROFILE-04 | Phase 3 | Pending |
| LOOSE-01 | Phase 4 | Pending |
| LOOSE-02 | Phase 4 | Pending |
| LOOSE-03 | Phase 4 | Pending |
| LOOSE-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 31 total
- Mapped to phases: 31
- Unmapped: 0

Note: The initial header said 27 requirements; actual count from the requirement list is 31. All 31 are mapped.

---
*Requirements defined: 2026-03-04*
*Last updated: 2026-03-04 after roadmap creation — traceability populated*
