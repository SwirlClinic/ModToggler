# Architecture Research

**Domain:** Mod management desktop application (Tauri v2 + React + TypeScript)
**Researched:** 2026-03-04
**Confidence:** HIGH (Tauri official docs + Vortex open source reference + established patterns)

## Standard Architecture

### System Overview

```
+---------------------------------------------------------------------+
|                          React Frontend (Webview)                   |
|                                                                     |
|  +-----------+  +-----------+  +-----------+  +-----------+        |
|  |   Game    |  |   Mod     |  | Profile   |  | Settings  |        |
|  |   View    |  |  List     |  |  Panel    |  |  Panel    |        |
|  +-----------+  +-----------+  +-----------+  +-----------+        |
|       |              |               |               |             |
|  +-----------------------------------------------------------+     |
|  |                 Zustand State Store                        |     |
|  |  (games[], activeMods[], profiles[], pendingOps[])        |     |
|  +-----------------------------------------------------------+     |
|       |                                                      |     |
|  +-----------------------------------------------------------+     |
|  |          IPC Layer — invoke() / @tauri-apps/api           |     |
+--+-----------------------------------------------------------+-----+
                              | (JSON over ipc://)
+---------------------------------------------------------------------+
|                          Rust Backend (Core)                        |
|                                                                     |
|  +-----------+  +-----------+  +-----------+  +-----------+        |
|  |  Command  |  | File Ops  |  | Conflict  |  | Archive   |        |
|  |  Handler  |  | Service   |  | Detector  |  | Extractor |        |
|  +-----------+  +-----------+  +-----------+  +-----------+        |
|       |              |               |               |             |
|  +-----------------------------------------------------------+     |
|  |                    App State (Arc<Mutex<T>>)               |     |
|  |  (GameRegistry, ModRegistry, FileIndex, ActiveProfiles)   |     |
|  +-----------------------------------------------------------+     |
|       |                                                            |
|  +-----------------------------------------------------------+     |
|  |                  Persistence Layer                         |     |
|  |   tauri-plugin-store (JSON)  |  SQLite (via sqlx)         |     |
|  +-----------------------------------------------------------+     |
|                              |                                     |
+------------------------------+-------------------------------------+
                               |
+---------------------------------------------------------------------+
|                         File System                                 |
|                                                                     |
|  ~/.modtoggler/                                                     |
|  +-- config.json          (app config via tauri-plugin-store)      |
|  +-- db.sqlite            (mod registry, file index)               |
|  +-- disabled/            (staging area for disabled mods)         |
|       +-- tekken8/        (per-game staging directories)           |
|            +-- ModName/   (extracted mod files sit here)           |
|                                                                     |
|  C:\...\TEKKEN 8\...\Mods\   (live game mod directory)            |
+---------------------------------------------------------------------+
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| Game View | Display one game at a time, list its mods, handle selection | React page component, driven by Zustand |
| Mod List | Render mods with toggle controls, conflict badges, option expanders | React list with per-item state from store |
| Profile Panel | Save/load named configurations of which mods are enabled | React modal, invokes Rust profile commands |
| Settings Panel | Configure game paths, add new games | React form, persisted via tauri-plugin-store |
| Zustand Store | Client-side cache of app state; invalidated after Rust commands succeed | Zustand store (no persistence — Rust is source of truth) |
| IPC Layer | Serializes invoke() calls and responses between webview and Rust | @tauri-apps/api/core — JSON over ipc:// protocol |
| Command Handler | Thin Tauri #[command] functions that route to services | Rust functions in src-tauri/src/commands/ |
| File Ops Service | All file moves between game dir and staging area; handles rename, mkdir | Rust in src-tauri/src/services/file_ops.rs |
| Conflict Detector | Compares FileIndex entries across enabled mods to find path overlaps | Rust in src-tauri/src/services/conflict.rs |
| Archive Extractor | Reads .zip, extracts to staging, records file manifest | Rust using zip crate |
| App State | Shared Rust state (Arc<Mutex<T>>) injected into commands via Tauri State<T> | Structs in src-tauri/src/state.rs |
| SQLite (db) | Persistent source of truth: games, mods, file manifests, profiles | sqlx with SQLite; managed at ~/.modtoggler/db.sqlite |
| tauri-plugin-store | Lightweight JSON key-value store for app-level config (window state, preferences) | tauri-plugin-store; stored at ~/.modtoggler/config.json |
| Staging Area | Disabled mods reside here; file names preserved, directory mirrors game layout | Plain filesystem at ~/.modtoggler/disabled/[game-slug]/ |

## Recommended Project Structure

```
ModToggler/
+-- src/                             # React frontend
|   +-- components/
|   |   +-- GameSelector.tsx         # Dropdown to switch active game
|   |   +-- ModList.tsx              # Scrollable list of mods for active game
|   |   +-- ModItem.tsx              # Single mod row with toggle, options, conflicts
|   |   +-- ModOptions.tsx           # Expandable sub-mod options (UE4 pak groups)
|   |   +-- ProfilePanel.tsx         # Save/load profile modal
|   |   +-- SettingsPanel.tsx        # Game path configuration
|   |   +-- ConflictBadge.tsx        # Visual indicator for conflicting mods
|   +-- hooks/
|   |   +-- useModToggle.ts          # Wraps invoke("toggle_mod"), handles loading state
|   |   +-- useImportMod.ts          # Handles file picker + invoke("import_mod")
|   |   +-- useConflicts.ts          # Derives conflict data from store state
|   +-- store/
|   |   +-- gameStore.ts             # Zustand: games[], activeGameId
|   |   +-- modStore.ts              # Zustand: mods[], enabled status, options
|   |   +-- profileStore.ts          # Zustand: profiles[] for active game
|   +-- lib/
|   |   +-- tauri.ts                 # Typed wrappers around invoke() calls
|   |   +-- types.ts                 # Shared TypeScript types matching Rust structs
|   +-- App.tsx
|   +-- main.tsx
|
+-- src-tauri/                       # Rust backend
|   +-- src/
|   |   +-- main.rs                  # App setup, plugin registration, command registration
|   |   +-- state.rs                 # App state structs (GameRegistry, FileIndex etc.)
|   |   +-- db/
|   |   |   +-- mod.rs               # DB module root
|   |   |   +-- schema.rs            # Table definitions (migrations inline or via sqlx-cli)
|   |   |   +-- queries.rs           # Named query functions (insert_mod, list_mods, etc.)
|   |   +-- commands/
|   |   |   +-- mod.rs               # pub mod declarations
|   |   |   +-- games.rs             # add_game, remove_game, list_games
|   |   |   +-- mods.rs              # import_mod, toggle_mod, remove_mod, list_mods
|   |   |   +-- profiles.rs          # save_profile, load_profile, list_profiles
|   |   |   +-- conflicts.rs         # get_conflicts (read-only query)
|   |   +-- services/
|   |   |   +-- file_ops.rs          # move_to_staging(), move_to_game_dir(), atomic ops
|   |   |   +-- archive.rs           # extract_zip(), build_file_manifest()
|   |   |   +-- conflict.rs          # compute_conflicts() from FileIndex
|   |   |   +-- mod_detector.rs      # auto-detect UE4 pak groupings from file stems
|   |   +-- error.rs                 # AppError enum, impl Into<tauri::InvokeError>
|   +-- Cargo.toml
|   +-- tauri.conf.json
|   +-- capabilities/
|       +-- default.json             # Tauri v2 permissions (fs, shell, store, sql)
```

### Structure Rationale

- **commands/ vs services/:** Commands are thin Tauri entry points (#[command] macros). Services contain the actual logic. This keeps commands testable (services can be called from unit tests) and the IPC boundary explicit.
- **db/ as its own module:** Database access is centralized. No ad-hoc SQL strings scattered through command handlers.
- **store/ in frontend:** Zustand stores are a client-side cache only — they are populated by reading Rust state after each command succeeds. They are never the primary source of truth for anything that touches the filesystem.
- **lib/tauri.ts:** All invoke() calls live here with full TypeScript typing, not spread across components. When a command signature changes in Rust, there is one place to update on the frontend.

## Architectural Patterns

### Pattern 1: Command-Service Separation

**What:** Tauri #[command] functions accept only raw inputs and State references. They call into service functions for logic, then return results. No file I/O or DB access inside command functions.

**When to use:** Always. Commands are thin adapters over services.

**Trade-offs:** Slight indirection overhead, but services can be unit-tested without a running Tauri instance.

**Example:**
```rust
// commands/mods.rs
#[tauri::command]
pub async fn toggle_mod(
    mod_id: i64,
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<ModStatus, AppError> {
    let app_state = state.lock().await;
    services::file_ops::toggle_mod(&app_state, mod_id, enabled).await
}

// services/file_ops.rs
pub async fn toggle_mod(state: &AppState, mod_id: i64, enabled: bool) -> Result<ModStatus, AppError> {
    // actual move logic lives here — testable independently
}
```

### Pattern 2: Optimistic UI with Rollback

**What:** The frontend marks the mod as toggling immediately (spinner state), then awaits the Rust command. On success, it refreshes state from Rust. On failure, it reverts and shows an error.

**When to use:** Toggle operations which may take time (large file moves). Makes UI feel instant.

**Trade-offs:** Adds complexity to the toggle hook. Required because file moves on Windows for large .pak files can take seconds.

**Example:**
```typescript
// hooks/useModToggle.ts
export function useModToggle(modId: number) {
  const { setModPending, refreshMods } = useModStore();

  return async (enabled: boolean) => {
    setModPending(modId, true);
    try {
      await invoke<ModStatus>('toggle_mod', { modId, enabled });
      await refreshMods(); // re-read from Rust after success
    } catch (err) {
      setModPending(modId, false); // revert spinner
      toast.error(`Failed to toggle mod: ${err}`);
    }
  };
}
```

### Pattern 3: File Manifest as Source of Truth

**What:** At import time, the archive extractor records every file path within a mod into the SQLite `file_entries` table, associated with the mod's ID. Toggle operations use this manifest to know exactly which files to move. Conflict detection queries this table to find path overlaps between enabled mods.

**When to use:** Always — this is the core data model for a file-tracking mod manager.

**Trade-offs:** Import is slower (must record manifests), but all subsequent operations are fast index lookups rather than filesystem scans.

**Example schema:**
```sql
CREATE TABLE mods (
    id          INTEGER PRIMARY KEY,
    game_id     INTEGER NOT NULL REFERENCES games(id),
    name        TEXT NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT 0,
    staged_path TEXT NOT NULL   -- path in ~/.modtoggler/disabled/[game]/
);

CREATE TABLE file_entries (
    id           INTEGER PRIMARY KEY,
    mod_id       INTEGER NOT NULL REFERENCES mods(id),
    relative_path TEXT NOT NULL  -- relative to mod root, e.g. "ExampleMod.pak"
    -- no absolute path stored; derived at runtime from mod.staged_path + relative_path
);
```

## Data Flow

### Import Mod Flow

```
User picks .zip file
    |
    v
invoke("import_mod", { game_id, zip_path })
    |
    v
[Rust] archive::extract_zip()
    -> extract files to ~/.modtoggler/disabled/[game]/[mod_name]/
    -> return list of relative file paths (manifest)
    |
    v
[Rust] mod_detector::detect_groups()
    -> scan manifest for UE4 stem groups (.pak/.ucas/.utoc with same base name)
    -> return suggested Mod + ModOption structure
    |
    v
[Rust] db::insert_mod() + db::insert_file_entries()
    -> persist mod metadata and full file manifest
    |
    v
[Rust] return ModRecord to frontend
    |
    v
[Frontend] modStore.addMod(record) -- optimistic add, already confirmed by Rust
```

### Toggle Mod Flow

```
User flips toggle switch for Mod A
    |
    v
invoke("toggle_mod", { mod_id, enabled: true })
    |
    v
[Rust] conflict::check_before_enable(mod_id)
    -> query file_entries for mod_id
    -> cross-join against file_entries for all currently-enabled mods
    -> if overlap found: return ConflictWarning (frontend shows warning, user confirms)
    |
    v
[Rust] file_ops::move_to_game_dir(mod_id)   [or move_to_staging for disable]
    -> for each file_entry: fs::rename(staged_path, game_path)
    -> atomic: if any rename fails, attempt rollback of moved files
    |
    v
[Rust] db::set_mod_enabled(mod_id, true)
    |
    v
[Frontend] refreshMods() -- re-reads from Rust to get authoritative state
```

### State Management

```
Rust DB (SQLite) — source of truth for all persistent state
    |
    | (invoke() on mount + after mutations)
    v
Zustand Store — client-side cache
    |
    | (subscribe)
    v
React Components -- render from store, dispatch actions via hooks
    |
    | (user action -> hook -> invoke)
    v
Rust Command -> Service -> DB -> return updated record
    |
    v
Zustand Store updated (refresh or optimistic)
```

### Key Data Flows

1. **Import:** zip file -> Rust extractor -> staging dir + DB manifest -> frontend cache updated
2. **Toggle on:** Frontend optimistic update -> Rust conflict check -> Rust file moves -> Rust DB update -> frontend re-reads authoritative state
3. **Toggle off:** Same as toggle on in reverse; files move from game dir to staging
4. **Conflict check:** Pure DB query — cross-join file_entries for mod against all enabled mods — no filesystem scan needed
5. **Profile save:** Frontend sends current mod enable states -> Rust serializes to DB profile table
6. **Profile load:** Rust reads profile -> computes delta (what needs to move) -> executes file ops in sequence

## Scaling Considerations

This is a single-user desktop app, so "scaling" means "how does it hold up as the user's mod library grows."

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1-20 mods | No concerns. Everything is fast. |
| 20-200 mods | File manifest queries need indexes on (mod_id, relative_path). SQLite handles this fine. |
| 200+ mods / large files | File move operations should show progress via Tauri events (emit progress from Rust, listen in frontend). Toggling a mod with 1GB of .pak files takes time — never block the UI. |

### Scaling Priorities

1. **First bottleneck:** Large file moves blocking the UI thread. Fix with async Rust + Tauri event progress reporting. This is important even for moderate use.
2. **Second bottleneck:** Conflict detection query time with large manifests. Fix with a composite index on file_entries(relative_path, mod_id). Should not be needed under 500 mods.

## Anti-Patterns

### Anti-Pattern 1: Storing Absolute Paths in the File Index

**What people do:** Record the full absolute path of each mod file (e.g., `C:\Users\eric\.modtoggler\disabled\tekken8\ExampleMod\ExampleMod.pak`) in the database.

**Why it's wrong:** If the user moves the app data folder, renames their Windows user account, or the app is used on a different machine via backup, every path in the DB is broken. Conflict detection also becomes a string comparison mess across different game root paths.

**Do this instead:** Store relative paths only (relative to the mod's staging root). Derive absolute paths at runtime by joining the configured staging dir + game slug + mod name + relative path. Store the game's live mod directory separately, configured per-game.

### Anti-Pattern 2: Doing File I/O from the Frontend

**What people do:** Use Tauri's `fs` plugin directly from React to read/write files, bypass the Rust command layer for "simple" operations.

**Why it's wrong:** File operations that seem simple can fail in non-obvious ways on Windows (permission errors in Program Files, files locked by the game process, long paths). Rust has better error handling primitives and can do atomic renames. Bypassing the command layer also means the SQLite manifest gets out of sync with reality.

**Do this instead:** All file operations go through Rust commands. The frontend only reads state and triggers commands. Keep the `fs` plugin permission scope minimal (or disabled entirely).

### Anti-Pattern 3: Treating the Zustand Store as the Source of Truth

**What people do:** After import, add the mod to the Zustand store and consider that the record. Skip re-reading from Rust because "we already know what we just imported."

**Why it's wrong:** The Rust side may have corrected names, detected groupings, assigned IDs, or failed partway through. The frontend's optimistic state drifts from reality. Subsequent toggle operations send stale data to Rust.

**Do this instead:** After any Rust command that mutates state (import, toggle, remove), call a refresh command that re-reads the canonical state from Rust and replaces the Zustand store contents. Zustand is a display cache, not a database.

### Anti-Pattern 4: Synchronous File Moves for Large Mods

**What people do:** Call a blocking file move command and wait for it to return, displaying a spinner. No progress indication. UI appears frozen.

**Why it's wrong:** Moving a 1GB .pak file on a spinning hard disk can take 10-30 seconds. The user cannot distinguish a slow operation from a crash. On Windows, large moves across volumes (if staging dir is on a different drive than the game) are copy+delete, which is even slower.

**Do this instead:** Use Tauri's event system. Emit progress events from Rust as files are moved (`app_handle.emit("move_progress", payload)`). Listen in the frontend and render a progress bar. Keep the UI responsive throughout.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Game mod directory | Direct filesystem access via Rust std::fs | May require elevated permissions on Program Files paths; handle PermissionDenied explicitly |
| .zip archives | zip crate (Rust) — synchronous extraction | Run on Tauri async command thread to avoid blocking. Large zips can be slow. |
| Windows shell (UAC) | If needed: tauri-plugin-shell or runas crate | Only needed if game is in Program Files and UAC is enforced; test early |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| React components <-> Zustand store | Direct hook access (useGameStore, useModStore) | No prop drilling; components subscribe to slices |
| Zustand store <-> Rust | invoke() calls via lib/tauri.ts wrappers | All invoke calls typed; never raw string command names in components |
| Rust commands <-> services | Direct function calls (same process) | Services return Result<T, AppError>; commands map AppError to InvokeError |
| Rust services <-> SQLite | sqlx async queries via db module | All queries in db/queries.rs — no inline SQL in services |
| Rust services <-> filesystem | std::fs (sync) or tokio::fs (async) | Use tokio::fs for large file moves to keep async executor happy |
| Tauri events (Rust -> Frontend) | app_handle.emit() / listen() in React | Used for progress reporting on long operations; not for primary data sync |

## Sources

- [Tauri v2 Architecture](https://v2.tauri.app/concept/architecture/) — official Tauri documentation
- [Tauri v2 IPC / Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) — official Tauri documentation
- [Tauri v2 State Management](https://v2.tauri.app/develop/state-management/) — official Tauri documentation
- [Tauri v2 Store Plugin](https://v2.tauri.app/plugin/store/) — official Tauri documentation
- [Tauri v2 SQL Plugin](https://v2.tauri.app/plugin/sql/) — official Tauri documentation
- [IronyModManager Conflict Detection Architecture](https://deepwiki.com/bcssov/IronyModManager/3.4-conflict-detection-and-resolution) — open source mod manager reference
- [Vortex Mod Manager (open source)](https://github.com/Nexus-Mods/Vortex) — reference implementation (Electron-based, same domain)
- [Tauri Global State Management pattern](https://github.com/robosushie/tauri-global-state-management) — Zustand + Rust state sync reference

---
*Architecture research for: Mod management desktop app (Tauri v2 + React + TypeScript)*
*Researched: 2026-03-04*
