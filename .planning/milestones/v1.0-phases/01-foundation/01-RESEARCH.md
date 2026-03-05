# Phase 1: Foundation - Research

**Researched:** 2026-03-04
**Domain:** Tauri v2 + React desktop app — project scaffolding, SQLite schema, transaction journal, cross-drive file ops, UAC elevation, startup integrity scan
**Confidence:** HIGH (stack from official docs + existing project research), MEDIUM (UAC helper IPC pattern — community patterns, not official Tauri docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Game Setup Flow**
- Folder picker dialog OR paste-path text field — both options available (picker button next to text input)
- Game entry captures: name, mod directory path, mod structure type (structured vs loose-file)
- No auto-detection of Steam library games — manual add only (auto-detect deferred to future)
- When adding a game with an existing mod directory, offer to scan for existing mods (auto-detect .pak/.ucas/.utoc groups)

**Staging Location**
- Default: each game gets its own staging folder inside an app-managed games directory (e.g. `~/.modtoggler/games/[game-name]/staging/`)
- User can override and choose a custom staging folder per game
- Same-drive staging preferred by default to enable instant `rename()` — app should detect drive mismatch and warn
- Flat per-mod layout inside staging: `staging/[modname]/` contains that mod's disabled files

**Permission Strategy**
- UAC elevation prompted once per session — not per operation
- Elevated helper process spawned at app start when games are configured in protected paths (Program Files)
- Helper stays running for the session, handles all file moves in protected directories
- If no games are in protected paths, no elevation needed — helper not spawned

**App Shell / Layout**
- Single game view — select a game first, then full-screen mod view
- Dark theme default — fits gaming context, standard for gaming tools
- Navigation: game selector (dropdown or picker page) at top level, mod list fills the main area
- Settings via gear icon — opens panel for game management, staging config, preferences
- Follow best practices for Tauri + React desktop apps — Claude's discretion on specific component choices

### Claude's Discretion
- Specific UI component library and styling approach
- Exact layout spacing, typography, visual details
- Loading states and skeleton designs
- Error state presentation (beyond "clear error messages" requirement)
- Transaction journal implementation details
- Integrity scan frequency and depth

### Deferred Ideas (OUT OF SCOPE)
- Auto-detect Steam library games — future enhancement to game setup
- Custom game icons/thumbnails — nice to have, not v1 priority
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GAME-01 | User can add a game with a name and mod directory path | Game config form pattern, SQLite games table schema, tauri-plugin-dialog for folder picker |
| GAME-02 | User can remove a game from the app | SQLite DELETE with CASCADE, staging directory cleanup pattern |
| GAME-03 | User can edit a game's name and mod directory path | SQLite UPDATE, cross-drive re-detection on path change |
| GAME-04 | User can select a game to view its mod list (empty in Phase 1) | TanStack Router route params, Zustand activeGameId, empty state component |
| TOGGLE-04 | Mod enabled/disabled state persists across app restarts | SQLite mods.enabled column, read on startup, Zustand populated from Rust |
| TOGGLE-06 | App uses a transaction journal to ensure file moves are atomic — crash mid-toggle can be recovered | SQLite journal table, startup scan for in_progress entries, complete-or-rollback pattern |
| RELIAB-01 | App performs integrity scan on startup to detect files moved outside the app | Startup command reads DB, checks file existence, surfaces discrepancies to UI |
| RELIAB-02 | App handles cross-drive file moves gracefully (copy+delete with progress when rename fails) | tokio::fs rename fails cross-volume, catch error, fall back to copy+delete with app.emit() progress |
| RELIAB-03 | App provides clear error messages when file operations fail (permissions, missing files) | AppError enum with distinct variants, tauri-specta serializes to frontend, error display component |
</phase_requirements>

---

## Summary

Phase 1 builds the foundation everything else stands on: a working Tauri v2 + React project, the SQLite schema, game configuration UI, staging directory management, and the reliability infrastructure (transaction journal + startup integrity scan). No mod import or toggle UI is built in this phase — but the atomic file operation primitives and crash recovery mechanism are built here because they cannot be retrofitted later without significant rework.

The existing project research (STACK.md, ARCHITECTURE.md, PITFALLS.md) is high-quality and directly applicable. This phase research adds Phase 1-specific depth: exact tauri-plugin-sql migration setup, tauri-specta wiring pattern, cross-volume rename detection and fallback, the transaction journal data model, and the UAC helper process architecture decision (partially resolved — see Open Questions).

**Primary recommendation:** Scaffold with `create-tauri-app`, wire tauri-specta bindings and SQLite migrations in Wave 1 before any feature code, build the transaction journal into the `file_ops` service from the start, and detect cross-drive staging paths at game-add time.

---

## Standard Stack

### Core (from STACK.md — HIGH confidence)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.10.3 | App shell, IPC bridge, webview | Project-chosen; Rust backend for file ops |
| React | 19.x | Frontend UI | Project-chosen; current stable |
| TypeScript | 5.x | Type safety | Required for tauri-specta value |
| Vite | 7.x | Build/dev server | Default for Tauri React template |
| Rust stable | 1.77.2+ | Backend logic, file operations | Required by Tauri; tauri-plugin-sql minimum |

### State + Data Layer

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Zustand | 5.x (5.0.6+) | UI state cache (games[], activeGameId) | Minimal boilerplate, no Provider wrapping, React 19 compatible |
| TanStack Query | 5.x (5.90+) | Async data over invoke() (loading, error, cache) | Eliminates manual loading state; wraps all invoke() calls |
| TanStack Router | 1.x | Client-side routing | Type-safe route params; designed for Vite SPAs |
| tauri-plugin-sql | 2.3.x | SQLite from frontend + Rust | Official plugin; migrations, parameterized queries |
| tauri-specta | 2.0.0-rc.21 | TypeScript bindings from Rust commands | Eliminates hand-typed invoke() type casts; compile-time IPC safety |

### UI

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Tailwind CSS | 4.x (4.2+) | Utility-first styling | Always; v4 is current stable (Jan 2025) |
| shadcn/ui | latest (CLI) | Component primitives (owned code) | Always; copied into codebase, not a dependency |
| Lucide React | latest | Icons | Ships with shadcn/ui; use for all icons |

### Tauri Plugins (Phase 1 relevant)

| Plugin/Crate | Version | Purpose |
|------|---------|---------|
| @tauri-apps/plugin-sql | 2.x | SQLite from TypeScript frontend |
| tauri-plugin-sql (Rust) | 2.3.x | SQLite via sqlx in Rust commands |
| @tauri-apps/plugin-dialog | 2.x | Folder picker dialog for game path selection |
| @tauri-apps/plugin-fs | 2.x | Scoped file operations (used minimally) |

### Installation

```bash
# Scaffold (run once)
npm create tauri-app@latest ModToggler -- --template react-ts
cd ModToggler

# State + routing
npm install zustand @tanstack/react-query @tanstack/react-router

# UI
npm install tailwindcss @tailwindcss/vite
npx shadcn@latest init
npx shadcn@latest add button card dialog input label select separator

# Icons
npm install lucide-react

# Tauri plugins
npm run tauri add sql
npm run tauri add dialog
npm run tauri add fs

# Dev tools
npm install -D vitest @testing-library/react @vitest/ui

# Rust side — in src-tauri/:
cargo add tauri-plugin-sql --features sqlite
cargo add tauri-specta --features typescript,tauri
cargo add specta-typescript
cargo add serde --features derive
cargo add tokio --features full
```

---

## Architecture Patterns

### Recommended Project Structure

```
ModToggler/
├── src/                              # React frontend
│   ├── components/
│   │   ├── GameSelector.tsx          # Dropdown to switch active game
│   │   ├── GameForm.tsx              # Add/edit game modal form
│   │   ├── EmptyModView.tsx          # Placeholder — "No mods yet" for Phase 1
│   │   ├── SettingsPanel.tsx         # Gear icon panel: game management, staging config
│   │   ├── IntegrityAlert.tsx        # Surface startup integrity scan warnings
│   │   └── ErrorToast.tsx            # Typed error display from Rust AppError
│   ├── hooks/
│   │   ├── useGames.ts               # TanStack Query: list_games, add_game, remove_game, edit_game
│   │   └── useIntegrityScan.ts       # Reads integrity scan results on app startup
│   ├── store/
│   │   └── gameStore.ts              # Zustand: games[], activeGameId, setActiveGame
│   ├── lib/
│   │   └── tauri.ts                  # Typed invoke() wrappers (or import from bindings.ts)
│   ├── bindings.ts                   # AUTO-GENERATED by tauri-specta (do not edit)
│   ├── App.tsx
│   └── main.tsx
│
└── src-tauri/
    ├── src/
    │   ├── main.rs                   # App entry, plugin registration, tauri-specta builder
    │   ├── lib.rs                    # run() function
    │   ├── error.rs                  # AppError enum + impl Into<InvokeError>
    │   ├── state.rs                  # AppState struct (Arc<Mutex<T>>) — Phase 1: minimal
    │   ├── db/
    │   │   ├── mod.rs
    │   │   ├── migrations.rs         # All SQL migration strings as const
    │   │   └── queries.rs            # insert_game, list_games, delete_game, update_game
    │   ├── commands/
    │   │   ├── mod.rs
    │   │   ├── games.rs              # add_game, remove_game, edit_game, list_games
    │   │   └── integrity.rs          # run_integrity_scan, resolve_integrity_issue
    │   └── services/
    │       ├── file_ops.rs           # move_file_atomic(), create_staging_dir(), detect_cross_drive()
    │       └── journal.rs            # write_journal_entry(), complete_journal(), scan_incomplete_journals()
    ├── Cargo.toml
    ├── tauri.conf.json
    └── capabilities/
        └── default.json              # fs, sql, dialog permissions
```

### Pattern 1: tauri-specta Wiring (IPC Type Safety)

**What:** Generate TypeScript bindings from Rust command signatures automatically.
**When to use:** Every Tauri command in the project — establish this in Wave 1.

```rust
// src-tauri/src/main.rs
use tauri_specta::{collect_commands, ts};

fn main() {
    let invoke_handler = {
        let builder = ts::builder()
            .commands(collect_commands![
                commands::games::add_game,
                commands::games::remove_game,
                commands::games::edit_game,
                commands::games::list_games,
                commands::integrity::run_integrity_scan,
            ]);

        #[cfg(debug_assertions)]
        let builder = builder.path("../src/bindings.ts");

        builder.build().unwrap()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:modtoggler.db", db::migrations::get_migrations())
            .build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```rust
// Commands with specta annotations
#[tauri::command]
#[specta::specta]
pub async fn add_game(
    name: String,
    mod_dir: String,
    staging_dir: Option<String>,
    mod_structure: ModStructureType,
    state: tauri::State<'_, AppState>,
) -> Result<GameRecord, AppError> {
    services::games::add_game(&state, name, mod_dir, staging_dir, mod_structure).await
}
```

Frontend usage (from generated `bindings.ts`):
```typescript
// src/lib/tauri.ts
import * as commands from '../bindings';
export { commands };

// In component/hook:
const games = await commands.listGames();
const game = await commands.addGame({ name, modDir, stagingDir, modStructure });
```

### Pattern 2: SQLite Migrations via tauri-plugin-sql

**What:** Define schema as versioned migration structs in Rust; plugin applies them automatically on startup.
**When to use:** All DDL in Phase 1 — the schema established here carries through all four phases.

```rust
// src-tauri/src/db/migrations.rs
use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_games_table",
            sql: "
                CREATE TABLE IF NOT EXISTS games (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    name            TEXT NOT NULL,
                    mod_dir         TEXT NOT NULL,
                    staging_dir     TEXT NOT NULL,
                    mod_structure   TEXT NOT NULL CHECK(mod_structure IN ('structured', 'loose')),
                    created_at      INTEGER NOT NULL DEFAULT (unixepoch())
                );
            ",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_mods_table",
            sql: "
                CREATE TABLE IF NOT EXISTS mods (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_id         INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                    name            TEXT NOT NULL,
                    enabled         INTEGER NOT NULL DEFAULT 0,
                    staged_path     TEXT NOT NULL,
                    imported_at     INTEGER NOT NULL DEFAULT (unixepoch())
                );
            ",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_file_entries_table",
            sql: "
                CREATE TABLE IF NOT EXISTS file_entries (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    mod_id          INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
                    relative_path   TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_file_entries_mod_id
                    ON file_entries(mod_id);
                CREATE INDEX IF NOT EXISTS idx_file_entries_path
                    ON file_entries(relative_path, mod_id);
            ",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_toggle_journal_table",
            sql: "
                CREATE TABLE IF NOT EXISTS toggle_journal (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    mod_id          INTEGER NOT NULL REFERENCES mods(id),
                    operation       TEXT NOT NULL CHECK(operation IN ('enable', 'disable')),
                    status          TEXT NOT NULL CHECK(status IN ('in_progress', 'done', 'rolled_back')),
                    files_json      TEXT NOT NULL,   -- JSON array of {src, dst} pairs
                    started_at      INTEGER NOT NULL DEFAULT (unixepoch()),
                    completed_at    INTEGER
                );
            ",
            kind: MigrationKind::Up,
        },
    ]
}
```

### Pattern 3: Transaction Journal for Atomic File Moves

**What:** Before any file operation, write a journal entry. After each file move, update it. On startup, scan for `in_progress` entries and recover.
**When to use:** Every toggle operation (Phase 2+) — but the service and table are built in Phase 1.

```rust
// src-tauri/src/services/journal.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePair {
    pub src: String,
    pub dst: String,
    pub done: bool,
}

pub async fn begin_toggle(
    db: &Database,
    mod_id: i64,
    operation: &str,
    files: &[FilePair],
) -> Result<i64, AppError> {
    let files_json = serde_json::to_string(files)?;
    let row: (i64,) = db.select(
        "INSERT INTO toggle_journal (mod_id, operation, status, files_json)
         VALUES ($1, $2, 'in_progress', $3)
         RETURNING id",
        vec![mod_id.into(), operation.into(), files_json.into()],
    ).await?;
    Ok(row.0)
}

pub async fn mark_file_done(db: &Database, journal_id: i64, updated_files: &[FilePair]) -> Result<(), AppError> {
    let files_json = serde_json::to_string(updated_files)?;
    db.execute(
        "UPDATE toggle_journal SET files_json = $1 WHERE id = $2",
        vec![files_json.into(), journal_id.into()],
    ).await?;
    Ok(())
}

pub async fn complete_journal(db: &Database, journal_id: i64) -> Result<(), AppError> {
    db.execute(
        "UPDATE toggle_journal SET status = 'done', completed_at = unixepoch() WHERE id = $1",
        vec![journal_id.into()],
    ).await?;
    Ok(())
}

/// Called at startup — find any in_progress entries and attempt recovery
pub async fn scan_incomplete(db: &Database) -> Result<Vec<IncompleteJournalEntry>, AppError> {
    db.select(
        "SELECT id, mod_id, operation, files_json FROM toggle_journal WHERE status = 'in_progress'",
        vec![],
    ).await.map_err(Into::into)
}
```

### Pattern 4: Cross-Drive Detection and Fallback

**What:** `tokio::fs::rename()` fails silently-ish when source and destination are on different volumes. Detect this and fall back to copy+delete with progress events.
**When to use:** All file move operations in `file_ops.rs` service.

```rust
// src-tauri/src/services/file_ops.rs
use tokio::fs;
use tauri::{AppHandle, Emitter};
use std::path::Path;

/// Atomic-or-fallback file move with progress reporting
pub async fn move_file(
    app: &AppHandle,
    src: &Path,
    dst: &Path,
) -> Result<(), AppError> {
    // Ensure destination parent directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Attempt atomic rename first
    match fs::rename(src, dst).await {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device_error(&e) => {
            // Cross-drive: fall back to copy + delete with progress events
            copy_with_progress(app, src, dst).await?;
            fs::remove_file(src).await?;
            Ok(())
        }
        Err(e) => Err(AppError::IoError(e)),
    }
}

fn is_cross_device_error(e: &std::io::Error) -> bool {
    // Windows: ERROR_NOT_SAME_DEVICE = os error 17
    // Also catches cross-filesystem on Linux (EXDEV = 18)
    matches!(e.raw_os_error(), Some(17) | Some(18))
}

async fn copy_with_progress(app: &AppHandle, src: &Path, dst: &Path) -> Result<(), AppError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let file_size = fs::metadata(src).await?.len();
    let mut reader = fs::File::open(src).await?;
    let mut writer = fs::File::create(dst).await?;
    let mut buf = vec![0u8; 256 * 1024]; // 256KB chunks
    let mut bytes_copied: u64 = 0;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 { break; }
        writer.write_all(&buf[..n]).await?;
        bytes_copied += n as u64;

        let pct = (bytes_copied * 100 / file_size.max(1)) as u32;
        let _ = app.emit("file-move-progress", MoveProgressEvent {
            file: src.file_name().unwrap_or_default().to_string_lossy().into(),
            percent: pct,
        });
    }

    writer.flush().await?;
    Ok(())
}

#[derive(Clone, serde::Serialize)]
pub struct MoveProgressEvent {
    pub file: String,
    pub percent: u32,
}
```

### Pattern 5: Error Type Hierarchy

**What:** A single `AppError` enum covering all error cases, serializable to the frontend via tauri-specta.
**When to use:** Every Rust command's return type is `Result<T, AppError>`.

```rust
// src-tauri/src/error.rs
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    IoError(String),
    PermissionDenied(String),
    DatabaseError(String),
    GameNotFound(String),
    ModNotFound(String),
    StagingDirConflict(String),
    CrossDriveWarning(String),  // not an error per se, but surfaced
    JournalCorrupt(String),
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                AppError::PermissionDenied(e.to_string())
            }
            std::io::ErrorKind::NotFound => {
                AppError::IoError(format!("File not found: {}", e))
            }
            _ => AppError::IoError(e.to_string()),
        }
    }
}

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(e: AppError) -> Self {
        tauri::ipc::InvokeError::from_anyhow(anyhow::anyhow!("{:?}", e))
    }
}
```

### Pattern 6: Startup Integrity Scan

**What:** On every app launch, read the DB and verify file system state matches.
**When to use:** Called from a `setup` hook or the first `use_effect` on mount.

```rust
// src-tauri/src/commands/integrity.rs
#[tauri::command]
#[specta::specta]
pub async fn run_integrity_scan(
    db: tauri::State<'_, Database>,
) -> Result<IntegrityScanResult, AppError> {
    let mods: Vec<ModRecord> = db::queries::list_all_mods(&db).await?;
    let mut missing_from_game: Vec<ModRecord> = vec![];
    let mut missing_from_staging: Vec<ModRecord> = vec![];
    let mut incomplete_journals: Vec<JournalEntry> = vec![];

    // Check incomplete journals first — these take priority
    let journals = services::journal::scan_incomplete(&db).await?;
    incomplete_journals.extend(journals);

    for mod_rec in mods {
        let files: Vec<FileEntry> = db::queries::list_file_entries(&db, mod_rec.id).await?;
        if mod_rec.enabled {
            // Files should be in the game directory
            for file in &files {
                let expected = Path::new(&mod_rec.game_mod_dir).join(&file.relative_path);
                if !expected.exists() {
                    missing_from_game.push(mod_rec.clone());
                    break;
                }
            }
        } else {
            // Files should be in staging
            for file in &files {
                let expected = Path::new(&mod_rec.staged_path).join(&file.relative_path);
                if !expected.exists() {
                    missing_from_staging.push(mod_rec.clone());
                    break;
                }
            }
        }
    }

    Ok(IntegrityScanResult {
        missing_from_game,
        missing_from_staging,
        incomplete_journals,
    })
}
```

### Pattern 7: Cross-Drive Detection at Game-Add Time

**What:** When the user adds or edits a game, detect if the proposed staging path is on a different drive. Warn immediately rather than on first toggle.

```rust
// src-tauri/src/services/file_ops.rs
pub fn same_volume(path_a: &Path, path_b: &Path) -> bool {
    // Simple approach: compare drive prefixes on Windows
    // e.g. "C:\" vs "D:\" — works for absolute paths
    let root_a = path_a.components().next();
    let root_b = path_b.components().next();
    root_a == root_b
}

// More robust: compare volume serial numbers via Windows API
// Requires the `windows` crate — use only if simple prefix comparison proves insufficient
```

When adding a game, if `!same_volume(mod_dir, staging_dir)`:
- Return a `CrossDriveWarning` in the result (not a hard error)
- Frontend surfaces this as a dismissible banner: "Staging folder is on a different drive. File moves will be slower. Consider placing staging near your game directory."

### Anti-Patterns to Avoid

- **Putting SQL in command handlers:** All queries go in `db/queries.rs`. Commands call queries, not inline SQL.
- **Storing absolute paths in file_entries:** Store only `relative_path` (relative to mod root). Derive absolute path at runtime from `staged_path + relative_path`.
- **Doing file I/O from React via plugin-fs:** All file operations go through Rust commands.
- **Treating Zustand as source of truth:** After any mutating command, call a refresh to re-read from Rust.
- **Skipping the journal:** Even in Phase 1 when there are no toggles yet — build the journal table and service now. Phase 2 just uses it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| IPC type safety | Hand-typed `invoke<ReturnType>('command_name')` everywhere | tauri-specta | Type drift between Rust and TS is guaranteed to cause bugs; specta gives compile-time safety |
| SQL migrations | Manual `CREATE TABLE IF NOT EXISTS` in setup hooks | tauri-plugin-sql migrations | Plugin handles versioning, ordering, transactions, and rollback automatically |
| Folder picker dialog | Custom file input component | tauri-plugin-dialog `open()` with `directory: true` | Native OS folder picker, handles edge cases, returns proper path |
| Dark mode toggle | Custom CSS variables solution | shadcn/ui ThemeProvider with `defaultTheme="dark"` | Handles system preference, localStorage persistence, no flash |
| Progress bar | Custom CSS animation | shadcn/ui Progress component | Accessible, animated, works with tauri event payloads directly |
| Error toasting | Custom notification div | shadcn/ui Sonner or Toast | Accessible, stacking, auto-dismiss; handles multiple simultaneous errors |
| Volume comparison | Parsing path strings manually | Compare drive prefix (path root component) | Suffix parsing is fragile; component comparison is reliable for Windows absolute paths |

**Key insight:** In Tauri v2, the plugin ecosystem handles the "plumbing" work. The app's value is in the file-tracking logic — spend engineering time there, not reinventing dialog boxes.

---

## Common Pitfalls

### Pitfall 1: Missing `#[specta::specta]` on Commands

**What goes wrong:** Commands work at runtime but TypeScript bindings file has no type information for them. Frontend falls back to raw untyped `invoke()` calls.
**Why it happens:** Developer adds a new command to `collect_commands![]` but forgets the `#[specta::specta]` attribute on the function.
**How to avoid:** Lint rule or code review checklist: every `#[tauri::command]` must also have `#[specta::specta]`.
**Warning signs:** `bindings.ts` exists but is smaller than expected; TypeScript says command doesn't exist on `commands.*`.

### Pitfall 2: Migration Version Numbers Conflicting

**What goes wrong:** App fails to start with "migration version already applied" error after a refactor moves migrations around.
**Why it happens:** Each `Migration.version` must be globally unique and monotonically increasing. Inserting a new migration between two existing ones with a duplicate number breaks the migration system.
**How to avoid:** Never reuse version numbers. New migrations always get the next highest number. Document this in a comment at the top of `migrations.rs`.
**Warning signs:** App panics at startup with `tauri_plugin_sql` error mentioning migration version.

### Pitfall 3: Running Whole Tauri App Elevated Breaks WebView2

**What goes wrong:** If the embedded manifest requests `requireAdministrator`, WebView2 fails to start on Windows 11 with Administrator Protection enabled because it cannot access the elevated user's AppData directory.
**Why it happens:** WebView2 initialization resolves user data dirs relative to the calling user context; running as admin changes that context in a way WebView2 doesn't expect.
**How to avoid:** Never embed an admin-elevation manifest in the main Tauri app. Use a separate small helper binary for privileged file operations.
**Warning signs:** `tauri::Builder::default()...run()` fails on Windows 11 when manifest includes `requireAdministrator`. GitHub issue #13926 documents this.

### Pitfall 4: cross-volume rename() Fails Silently

**What goes wrong:** `tokio::fs::rename()` returns `Err` with OS error 17 (Windows) or 18 (Linux) when source and destination are on different volumes. The toggle appears to fail with a cryptic IO error rather than "this will be slow, please wait."
**Why it happens:** `rename()` is documented as not working across mount points but developers testing on same-drive setups never trigger this.
**How to avoid:** Wrap every `rename()` call in the `move_file()` helper that catches OS error 17/18 and falls back to copy+delete with progress events.
**Warning signs:** Users report "file move failed" errors when their staging dir is on C: and game is on D:.

### Pitfall 5: Incomplete Journal on First Startup (Before Mods Exist)

**What goes wrong:** Integrity scan runs on first startup when the DB has no mods. Scan logic iterates over empty mod list, incorrectly handles `None` case, and either panics or shows a false "integrity issue" warning.
**Why it happens:** Scan logic tested only with mods present; empty-DB case not validated.
**How to avoid:** `scan_incomplete` and `run_integrity_scan` must handle empty DB gracefully — empty results, no warnings. Write a test for this case explicitly.

### Pitfall 6: Staging Directory Not Created Before File Ops

**What goes wrong:** When a mod is first disabled (in Phase 2), the staging directory `~/.modtoggler/games/[game-name]/staging/` doesn't exist yet. File move fails with `NotFound` error.
**Why it happens:** Staging dir is created at game-add time in the plan, but if the game was added before this code existed (e.g., migrated data), or the directory was manually deleted, the move fails.
**How to avoid:** `create_staging_dir()` is idempotent — call it before every file move operation, not just at game-add time. `tokio::fs::create_dir_all()` is safe to call on an existing directory.

---

## Code Examples

### Folder Picker Dialog (game path selection)

```typescript
// Source: https://v2.tauri.app/plugin/dialog/
import { open } from '@tauri-apps/plugin-dialog';

async function pickGameDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Mod Directory',
  });
  return selected as string | null;
}
```

### Load Database Connection on Frontend

```typescript
// Source: https://v2.tauri.app/plugin/sql/
// Note: In this project, DB access is done via Rust commands, not directly from JS.
// Direct JS DB access is only needed for debug/dev queries — use Rust commands in production.
import Database from '@tauri-apps/plugin-sql';
const db = await Database.load('sqlite:modtoggler.db');
```

### Listening to Progress Events

```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/
import { listen } from '@tauri-apps/api/event';

type MoveProgressEvent = { file: string; percent: number };

const unlisten = await listen<MoveProgressEvent>('file-move-progress', (event) => {
  setProgress(event.payload.percent);
  setCurrentFile(event.payload.file);
});

// Call unlisten() when component unmounts to prevent memory leak
```

### Zustand Game Store

```typescript
// Pattern: display cache only — always refresh from Rust after mutations
import { create } from 'zustand';
import type { GameRecord } from '../bindings';

interface GameStore {
  games: GameRecord[];
  activeGameId: number | null;
  setGames: (games: GameRecord[]) => void;
  setActiveGame: (id: number | null) => void;
}

export const useGameStore = create<GameStore>((set) => ({
  games: [],
  activeGameId: null,
  setGames: (games) => set({ games }),
  setActiveGame: (id) => set({ activeGameId: id }),
}));
```

### TanStack Query for Game List

```typescript
// Pattern: wraps invoke() with loading/error/cache semantics
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commands } from '../lib/tauri';

export function useGames() {
  return useQuery({
    queryKey: ['games'],
    queryFn: () => commands.listGames(),
  });
}

export function useAddGame() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: commands.addGame,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['games'] });
    },
  });
}
```

---

## UAC Helper Process Architecture

> This is a partially-resolved open question. The decision is made (separate helper binary) but the Tauri v2 IPC pattern between main app and helper is not fully resolved. See Open Questions.

### What's Confirmed (HIGH confidence)

1. **Do NOT run the whole Tauri app elevated.** Running the main app as admin via embedded manifest breaks WebView2 on Windows 11 with Administrator Protection. Source: [Tauri issue #13926](https://github.com/tauri-apps/tauri/issues/13926).

2. **Use a separate Rust CLI binary** that does only the privileged file operations. The main app spawns this helper when needed.

3. **The `runas` crate** provides a `Command` builder for spawning processes with UAC elevation on Windows. It triggers the UAC prompt once, then the helper runs elevated.

4. **The `is_elevated` crate** lets you detect whether the current process is already running elevated, avoiding double-prompts.

5. **Detect need for elevation at game-add time**: check whether `mod_dir` is inside `C:\Program Files` or `C:\Program Files (x86)`. If yes, note this in the `games` table (add a `requires_elevation` boolean column). On startup, if any game requires elevation, spawn the helper.

### Elevated Helper Architecture (MEDIUM confidence)

```
ModToggler (main app, not elevated)
    |
    | stdin/stdout pipe OR named pipe
    v
modtoggler-helper.exe (small CLI, spawned elevated via runas)
    |
    | std::fs::rename() / copy / delete
    v
C:\Program Files\...\Mods\  (protected directory)
```

The helper accepts JSON-line commands on stdin and writes JSON-line results to stdout:
```json
{"op": "move", "src": "C:\\Users\\...\\staging\\ModA\\file.pak", "dst": "C:\\Program Files (x86)\\Steam\\...\\Mods\\file.pak"}
```

The helper responds:
```json
{"ok": true}
// or
{"ok": false, "error": "PermissionDenied: ..."}
```

**Note:** The exact process spawning + stdio communication pattern within Tauri v2 (using `tauri-plugin-shell` or `std::process::Command` + `tauri-plugin-process`) needs validation against an actual Program Files path during Phase 1 development. See Open Questions.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 + CRA | Tauri v2 + Vite | 2024 | CRA deprecated; Vite is faster and the default |
| Hand-typed `invoke<T>('name')` | tauri-specta 2.x type generation | 2023-2024 | Eliminates IPC type drift; compile-time safety |
| Tailwind CSS v3 | Tailwind CSS v4 | Jan 2025 | CSS-native approach; faster builds; new config format |
| tauri-plugin-store for all config | SQLite for relational data + plugin-store for simple prefs | Tauri v2 era | Relational data (mods/files/profiles) needs SQLite; plugin-store still fine for window state |
| Direct invoke() in components | TanStack Query wrapping invoke() | 2023+ | Loading/error/stale states managed automatically; no manual useState |

**Deprecated/outdated:**
- `tauri-plugin-extract`: 3 stars, 9 commits, no confirmed Tauri v2 support — use `zip` Rust crate directly
- `zip-extract` crate: explicitly deprecated in favor of `zip` crate's built-in `extract()` method
- `create-react-app`: officially deprecated; do not use
- `tauri-specta 1.x`: targets Tauri v1 only; use `2.0.0-rc.21` for Tauri v2

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest 2.x |
| Config file | `vitest.config.ts` — Wave 0 creates if not present |
| Quick run command | `npm run test -- --run` |
| Full suite command | `npm run test -- --run --reporter=verbose` |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GAME-01 | `add_game` inserts record with correct fields | unit (Rust) | `cargo test commands::games::test_add_game` | ❌ Wave 0 |
| GAME-02 | `remove_game` deletes game and cascades to mods | unit (Rust) | `cargo test commands::games::test_remove_game_cascade` | ❌ Wave 0 |
| GAME-03 | `edit_game` updates name and path | unit (Rust) | `cargo test commands::games::test_edit_game` | ❌ Wave 0 |
| GAME-04 | Game selector renders, selecting game updates activeGameId | unit (Vitest) | `npm run test -- --run src/components/GameSelector.test.tsx` | ❌ Wave 0 |
| TOGGLE-04 | Mod `enabled` state persists after DB close/reopen | unit (Rust) | `cargo test db::queries::test_mod_enabled_persists` | ❌ Wave 0 |
| TOGGLE-06 | Journal entry written before move; startup scan finds `in_progress` | unit (Rust) | `cargo test services::journal::test_incomplete_journal_detected` | ❌ Wave 0 |
| RELIAB-01 | Integrity scan detects file missing from game dir | unit (Rust) | `cargo test commands::integrity::test_missing_from_game_dir` | ❌ Wave 0 |
| RELIAB-02 | Cross-drive error triggers copy+delete fallback | unit (Rust) | `cargo test services::file_ops::test_cross_drive_fallback` | ❌ Wave 0 |
| RELIAB-03 | PermissionDenied mapped to `AppError::PermissionDenied` with clear message | unit (Rust) | `cargo test error::test_permission_denied_mapping` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml` (Rust unit tests, ~5s)
- **Per wave merge:** Full suite: `cargo test` + `npm run test -- --run` (~15s)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/src/commands/games.rs` — test module with `test_add_game`, `test_remove_game_cascade`, `test_edit_game`
- [ ] `src-tauri/src/services/journal.rs` — test module with `test_incomplete_journal_detected`
- [ ] `src-tauri/src/services/file_ops.rs` — test module with `test_cross_drive_fallback` (mock IO)
- [ ] `src-tauri/src/commands/integrity.rs` — test module with `test_missing_from_game_dir`, `test_empty_db_no_warnings`
- [ ] `src-tauri/src/error.rs` — test module with `test_permission_denied_mapping`
- [ ] `src/components/GameSelector.test.tsx` — Vitest + React Testing Library; mock `commands.listGames()`
- [ ] `vitest.config.ts` — configure with `@tauri-apps/api` mock via `vi.mock('@tauri-apps/api/core')`
- [ ] `src-tauri/src/db/test_helpers.rs` — in-memory SQLite database fixture for Rust tests

---

## Open Questions

1. **UAC Helper: stdin/stdout vs. named pipe IPC**
   - What we know: Helper binary is the right pattern; `runas` crate triggers the UAC prompt; the helper runs the elevated file moves.
   - What's unclear: The exact Tauri v2 integration — should the main app use `tauri-plugin-shell` to spawn the helper and communicate via stdin/stdout? Or use a Windows named pipe for IPC? tauri-plugin-shell may not support long-running background processes well.
   - Recommendation: Start with `std::process::Command` in the Rust backend (not via plugin) to spawn the helper with stdio piped. JSON-line protocol over stdin/stdout is simple and testable. Validate against a real `C:\Program Files` path early — this is the "test early" principle from PITFALLS.md.

2. **tauri-specta RC Status**
   - What we know: Version `2.0.0-rc.21` is widely used in production Tauri projects. The API has been stable.
   - What's unclear: Whether a stable release will have breaking API changes before Phase 4 ships.
   - Recommendation: Pin exact version in `Cargo.toml`: `tauri-specta = "=2.0.0-rc.21"`. Check GitHub for stable release before Phase 3 begins.

3. **Staging Dir Location: `~/.modtoggler/games/[name]/staging/` vs. `~/.modtoggler/disabled/[name]/`**
   - What we know: CONTEXT.md locked the default to `~/.modtoggler/games/[game-name]/staging/` (slightly different from what REQUIREMENTS.md says: `~/.modtoggler/disabled/[game]/`).
   - What's unclear: This is a naming inconsistency between CONTEXT.md and REQUIREMENTS.md. The path structure is functionally equivalent.
   - Recommendation: Use `~/.modtoggler/games/[game-slug]/staging/[mod-name]/` as locked in CONTEXT.md. This structure better separates per-game data. Update DB schema comment accordingly.

---

## Sources

### Primary (HIGH confidence)

- [Tauri v2 official docs](https://v2.tauri.app/) — architecture, IPC, plugins, version 2.10.3
- [Tauri v2 SQL Plugin](https://v2.tauri.app/plugin/sql/) — migration struct API, `add_migrations()`, JS usage
- [Tauri v2 Calling Frontend](https://v2.tauri.app/develop/calling-frontend/) — `app.emit()`, `listen()` pattern, channels for high-throughput
- [tokio::fs::rename docs](https://docs.rs/tokio/latest/tokio/fs/fn.rename.html) — confirmed: fails across mount points
- [tauri-plugin-sql docs.rs](https://docs.rs/crate/tauri-plugin-sql/latest) — version 2.3.x confirmed
- [Tauri WebView2 elevation bug #13926](https://github.com/tauri-apps/tauri/issues/13926) — confirmed: running main app elevated breaks WebView2 on Win11

### Secondary (MEDIUM confidence)

- [specta.dev tauri-specta v2 docs](https://specta.dev/docs/tauri-specta/v2) — setup pattern, `collect_commands!`, `ts::builder()`
- [tauri-apps/tauri discussion #4201](https://github.com/tauri-apps/tauri/discussions/4201) — UAC elevation: separate helper binary recommendation
- [elevated-command crate](https://crates.io/crates/elevated-command) — UAC elevation for spawned commands on Windows
- [runas crate docs](https://docs.rs/runas/latest/runas/struct.Command.html) — `Command` builder for elevated process spawning
- [is_elevated crate](https://crates.io/crates/is_elevated) — detect if current process is elevated
- [shadcn/ui dark mode docs](https://ui.shadcn.com/docs/dark-mode) — ThemeProvider setup with `defaultTheme="dark"`
- [Tauri create-project docs](https://v2.tauri.app/start/create-project/) — scaffold command confirmed

### Tertiary (LOW confidence)

- [DEV Community: Tauri 2.0 + SQLite + React](https://dev.to/focuscookie/tauri-20-sqlite-db-react-2aem) — community tutorial; cross-validates plugin setup
- [deepwiki: tauri-specta getting started](https://deepwiki.com/specta-rs/tauri-specta/2-getting-started) — community wiki; cross-validates specta pattern

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against official Tauri v2 docs, npm, crates.io, existing STACK.md research
- Architecture patterns: HIGH — command-service-db layering from official Tauri docs + existing ARCHITECTURE.md
- SQLite schema: HIGH — migration API verified from official tauri-plugin-sql docs
- Transaction journal: HIGH — pattern design, SQLite schema; MEDIUM on exact serialization of `files_json` (implementation choice)
- Cross-drive detection: HIGH — tokio rename failure documented; OS error 17/18 confirmed
- UAC helper IPC: MEDIUM — pattern confirmed (separate binary), exact Tauri v2 spawn+communicate pattern unvalidated
- Pitfalls: HIGH — backed by official sources (GitHub issues, Microsoft Learn, Snyk)

**Research date:** 2026-03-04
**Valid until:** 2026-04-04 for stable libraries; check tauri-specta GitHub monthly (RC status)
