# Requirements: ModToggler

**Defined:** 2026-03-04
**Core Value:** Users can quickly toggle mods on and off without manually moving files, with confidence the app tracks what belongs to which mod.

## v1 Requirements

### Game Management

- [ ] **GAME-01**: User can add a game with a name and mod directory path
- [ ] **GAME-02**: User can remove a game from the app
- [ ] **GAME-03**: User can edit a game's name and mod directory path
- [ ] **GAME-04**: User can select a game to view its mod list

### Mod Import

- [ ] **IMPORT-01**: User can import a mod from a .zip archive
- [ ] **IMPORT-02**: App extracts .zip and records the full file manifest (every file path within the mod)
- [ ] **IMPORT-03**: App auto-groups .pak/.ucas/.utoc files by base stem as a single logical mod
- [ ] **IMPORT-04**: App detects sub-mod option folders (e.g. Option_ModName_Variant/) and registers them as independently toggleable options
- [ ] **IMPORT-05**: User can see which files belong to each mod
- [ ] **IMPORT-06**: App validates zip contents to prevent path traversal (ZipSlip protection)

### Mod Toggling

- [ ] **TOGGLE-01**: User can toggle a mod on/off with one click
- [ ] **TOGGLE-02**: Disabling a mod moves all its files from the game directory to the app-managed staging folder (~/.modtoggler/disabled/[game]/)
- [ ] **TOGGLE-03**: Enabling a mod moves its files back from staging to the game directory
- [ ] **TOGGLE-04**: Mod enabled/disabled state persists across app restarts
- [ ] **TOGGLE-05**: User can toggle individual sub-mod options on/off independently of the parent mod
- [ ] **TOGGLE-06**: App uses a transaction journal to ensure file moves are atomic — crash mid-toggle can be recovered
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

- [ ] **RELIAB-01**: App performs integrity scan on startup to detect files moved outside the app (antivirus, manual moves)
- [ ] **RELIAB-02**: App handles cross-drive file moves gracefully (copy+delete with progress indication when rename fails)
- [ ] **RELIAB-03**: App provides clear error messages when file operations fail (permissions, missing files)

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
| *(populated during roadmap creation)* | | |

**Coverage:**
- v1 requirements: 27 total
- Mapped to phases: 0
- Unmapped: 27 ⚠️

---
*Requirements defined: 2026-03-04*
*Last updated: 2026-03-04 after initial definition*
