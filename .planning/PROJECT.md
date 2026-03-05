# ModToggler

## What This Is

A Tauri v2 desktop application for managing game mods across multiple games. Users import mods from .zip archives, and the app tracks their file structure to enable toggling mods on/off by moving files between the game directory and an app-managed staging area. Supports structured mod directories (like Tekken 8's Mods folder with .pak/.ucas/.utoc grouping) and loose-file games where mod files are scattered across the game root with manual file tagging.

## Core Value

Users can quickly toggle mods on and off without manually moving files, with confidence the app tracks what belongs to which mod and won't break their setup.

## Requirements

### Validated

- ✓ Manage multiple games with configured mod paths — v1.0
- ✓ Import mods from .zip archives with automatic file structure learning — v1.0
- ✓ Toggle mods on/off by moving files between game directory and staging — v1.0
- ✓ Unreal Engine mod structure support (.pak/.ucas/.utoc grouping) — v1.0
- ✓ Sub-mod option folders with independent toggling — v1.0
- ✓ Loose-file game support with manual file tagging and destination paths — v1.0
- ✓ Conflict detection with specific file overlap warnings — v1.0
- ✓ Per-game named profiles for save/load of mod configurations — v1.0
- ✓ Transaction journal for crash recovery during file moves — v1.0
- ✓ Integrity scan on startup to detect externally moved files — v1.0
- ✓ Cross-drive file moves with copy+delete fallback — v1.0
- ✓ ZipSlip protection on import — v1.0

### Active

- [ ] Export/import profiles as JSON for sharing
- [ ] User-editable notes on individual mods
- [ ] Cross-platform support (macOS/Linux)

### Out of Scope

- Mod downloading from Nexus/TekkenMods — local-file-only; avoids API keys, login, rate limits
- Auto-updating mods — can break working setups; outside scope of file manager
- Game launching/injection — varies per game; anti-cheat complications; separate domain
- Virtual file system (VFS) — enormous Windows complexity; file-move is simpler and sufficient
- Load order management — PAK files don't merge; conflict detection is the right pattern
- Mod merging — requires understanding internal file formats; deeply game-specific
- Cloud sync — niche use case; JSON export covers sharing

## Context

Shipped v1.0 with 7,733 LOC across Rust and TypeScript.
Tech stack: Tauri v2, React 19, SQLite (sqlx), TanStack Query, Zustand, shadcn/ui, Tailwind CSS.
Primary use case: Tekken 8 modding on Windows.
Windows-first but Tauri enables cross-platform if needed later.

## Constraints

- **Tech stack**: Tauri v2 + React + TypeScript
- **Platform**: Windows primary target
- **File operations**: Must handle large files (hundreds of MB) and paths in Program Files
- **Storage**: App-managed staging at ~/.modtoggler/ for disabled mods

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri over Electron | Smaller binary, better performance, Rust backend for file ops | ✓ Good |
| Import from .zip | Lets app learn mod structure automatically | ✓ Good |
| Move files for toggle | Clean separation — disabled mods out of game directory | ✓ Good |
| App-managed staging folder | Centralized, predictable location for disabled mods | ✓ Good |
| React frontend | User preference, large ecosystem | ✓ Good |
| SQLite via sqlx (not tauri-plugin-sql) | Rust-side queries, type-safe, migration support | ✓ Good |
| Transaction journal for crash recovery | Ensures file moves are atomic; no orphaned files | ✓ Good |
| tauri-specta for typed bindings | Type-safe IPC between Rust and TypeScript | ✓ Good |
| mod_type column (not inferred) | Explicit per-mod type tracking for loose vs structured | ✓ Good |
| Profiles store sub_mod_states as JSON | Avoids third join table; simpler schema | ✓ Good |

---
*Last updated: 2026-03-05 after v1.0 milestone*
