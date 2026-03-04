# ModToggler

## What This Is

A Tauri desktop application for managing game mods across multiple games. Users import mods from .zip archives, and the app tracks their file structure to enable toggling mods on/off by moving files between the game directory and an app-managed staging area. Supports games with simple mod directories (like Tekken 8's Mods folder) as well as games where mod files are scattered across the game's root directory.

## Core Value

Users can quickly toggle mods on and off without manually moving files around, with confidence that the app tracks what belongs to which mod and won't break their setup.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Manage multiple games, each with a configured mod path
- [ ] Import mods from .zip archives — app learns file structure on import
- [ ] Toggle mods on/off by moving files between game directory and app-managed staging
- [ ] Support Unreal Engine mod structure (.pak, .ucas, .utoc grouped by base name)
- [ ] Support mod options (sub-folders within a mod that can be individually toggled)
- [ ] Support "mishmash" games where mod files are manually tagged to arbitrary paths
- [ ] Auto-detect mod grouping by file stem in structured mod directories
- [ ] Conflict detection — warn when two mods modify the same files
- [ ] Save/load named profiles of mod configurations per game
- [ ] Single game view UI — select a game, then see its mods with toggle controls
- [ ] App-managed disabled mods folder (~/.modtoggler/disabled/[game]/)

### Out of Scope

- Mobile app — desktop only
- Mod downloading/browsing from mod sites — import only
- Auto-updating mods — manual management
- Game launching — this is a file manager, not a launcher

## Context

- Primary use case: Tekken 8 modding on Windows
  - Mods directory: `C:\Program Files (x86)\Steam\steamapps\common\TEKKEN 8\Polaris\Content\Paks\Mods`
  - Mods consist of .pak/.ucas/.utoc file groups (e.g. ExampleMod.pak, ExampleMod.ucas, ExampleMod.utoc)
  - Some mods have optional sub-mods in folders (e.g. Option_ExampleMod_ColorTexture/ with subfolders each containing .pak/.ucas/.utoc)
- Secondary use case: games where mods are loose files scattered in the game root
  - These require manual file tagging since there's no consistent structure
- Windows-first but Tauri enables cross-platform if needed later

## Constraints

- **Tech stack**: Tauri v2 + React + TypeScript — cross-platform desktop with web UI
- **Platform**: Windows primary target (where the games are)
- **File operations**: Must handle large files (game mods can be hundreds of MB) and paths in Program Files (may need elevated permissions)
- **Storage**: App-managed staging at ~/.modtoggler/ for disabled mods

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri over Electron | Smaller binary, better performance, Rust backend for file ops | — Pending |
| Import from .zip | Lets app learn mod structure automatically vs manual file tracking | — Pending |
| Move files for toggle | Clean separation — disabled mods completely out of game directory | — Pending |
| App-managed staging folder | Centralized, predictable location for disabled mods | — Pending |
| React frontend | User preference, large ecosystem | — Pending |

---
*Last updated: 2026-03-04 after initialization*
