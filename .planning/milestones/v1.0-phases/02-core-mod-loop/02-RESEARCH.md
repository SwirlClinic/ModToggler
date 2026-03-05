# Phase 2: Core Mod Loop - Research

**Researched:** 2026-03-05
**Domain:** Zip extraction, file-system mod toggling, conflict detection (Rust/Tauri + React)
**Confidence:** HIGH

## Summary

Phase 2 implements the core value proposition: importing mods from zip files, toggling them on/off by moving files between staging and game directories, and warning users about file conflicts. The existing codebase provides strong foundations: `ModRecord`/`FileEntry` types, `file_ops::move_file()` with cross-drive fallback, journal service for crash-safe operations, and a React Query + Zustand frontend pattern.

The Rust `zip` crate (v8.x) is the standard for zip handling, with built-in `ZipArchive::extract()` that sanitizes paths via `enclosed_name()`. However, since we need to inspect files during extraction (to detect sub-mod options and build the file manifest), we should iterate entries manually rather than use the bulk extract method. ZipSlip protection comes from validating each entry name with `enclosed_name()` before writing.

The database schema already has `mods` and `file_entries` tables. Phase 2 needs new migrations for a `sub_mods` table (to track option folders as independently toggleable entities) and an index for conflict detection queries. The toggle flow uses the existing journal service to ensure crash safety, and conflict detection is a SQL query joining `file_entries` on `relative_path`.

**Primary recommendation:** Build import as a single Tauri command that receives a zip path + mod name, extracts to staging, records manifest in DB, and detects sub-mod options. Build toggle as a journaled file-move operation reusing existing `file_ops::move_file()`. Build conflict detection as a DB query, not in-memory comparison.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Drag-and-drop onto mod list area OR click "Import" button to open file picker -- both paths available
- Multi-mod zips (multiple .pak groups) imported as a single mod -- no auto-splitting
- Mod name pre-filled from zip filename, editable by user before confirming import
- Unexpected zip structures (no .pak files, random files): import with warning ("No recognized mod files found"), don't block
- ZipSlip protection required (IMPORT-06)
- Card-based layout with toggle switch per mod (reuses existing Card component from Phase 1)
- Each card shows: mod name, toggle switch, file count, enabled/disabled status
- Clicking a card (not the toggle) expands it inline to show file manifest and sub-mod options
- Import button positioned top-right in the header area
- Delete button appears inside the expanded card details
- Sort order: enabled mods first, then alphabetical within each group
- Conflict detected by exact file path match (relative_path in mod_files table)
- When enabling a mod that conflicts: warning dialog shows conflicting mod names + overlapping file paths
- Dialog offers three actions: "Enable Anyway", "Disable Other", "Cancel"
- Last-enabled wins on overlap -- newly enabled mod's files overwrite in game directory; overwritten files remain in the other mod's staging
- Persistent warning badge on mod cards that are currently in conflict -- clicking badge shows conflict details
- Sub-mods detected at import by `Option_` or `option_` prefix pattern
- Displayed nested inside expanded parent card under "Options" section with individual toggles
- Multiple options can be enabled simultaneously (no radio/exclusive behavior)
- When parent mod is toggled OFF, all options are disabled too -- individual option state is remembered and restored when parent is re-enabled

### Claude's Discretion
- Import progress indicator design (progress bar, spinner, etc.)
- Exact expanded card layout and spacing
- Empty state for new game with no mods (carries over EmptyModView pattern from Phase 1)
- Toggle animation/transition details
- Error handling for corrupted zips, permission failures during extract
- Elevated helper binary integration (UAC detection hooks from Phase 1 ready)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| IMPORT-01 | User can import a mod from a .zip archive | Rust `zip` crate v8.x for extraction; Tauri dialog for file picker; drag-drop via `TauriEvent.DRAG_DROP` |
| IMPORT-02 | App extracts .zip and records the full file manifest | Manual zip entry iteration with `ZipArchive::by_index()`, insert each path into `file_entries` table |
| IMPORT-03 | App auto-groups .pak/.ucas/.utoc files by base stem | Stem-matching logic during import scan; all files go into single mod record (no splitting per decision) |
| IMPORT-04 | App detects sub-mod option folders and registers as toggleable options | Pattern match on `Option_` / `option_` prefix; new `sub_mods` table with parent_mod_id FK |
| IMPORT-05 | User can see which files belong to each mod | File manifest shown in expanded card; `list_file_entries` query already exists |
| IMPORT-06 | App validates zip contents to prevent path traversal | `ZipFile::enclosed_name()` on each entry; reject entries that return `None` |
| TOGGLE-01 | User can toggle a mod on/off with one click | Toggle switch on card triggers `toggle_mod` command; uses journal + `file_ops::move_file()` |
| TOGGLE-02 | Disabling a mod moves files from game dir to staging | Build file pairs from `file_entries` + game `mod_dir` + mod `staged_path`; journal-wrapped move |
| TOGGLE-03 | Enabling a mod moves files from staging to game dir | Reverse of TOGGLE-02; same journal pattern |
| TOGGLE-05 | User can toggle individual sub-mod options independently | Sub-mod toggle uses same move pattern but scoped to sub-mod's file entries |
| TOGGLE-07 | User can permanently delete a mod and all its files | Delete from both staging and game dir, then cascade-delete DB records |
| CONFLICT-01 | App detects overlapping files between enabled mods | SQL query: `SELECT ... FROM file_entries fe1 JOIN file_entries fe2 ON fe1.relative_path = fe2.relative_path WHERE fe1.mod_id != fe2.mod_id` |
| CONFLICT-02 | App displays which mods conflict and over which files | Query returns mod names + conflicting paths; shown in UI badge + detail dialog |
| CONFLICT-03 | Conflict warnings appear when enabling a conflicting mod | Pre-toggle conflict check; frontend dialog with Enable Anyway / Disable Other / Cancel |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `zip` | 8.x (latest) | Zip archive reading/extraction | De facto Rust zip library; built-in path sanitization; actively maintained |
| `sqlx` | 0.8 | SQLite queries for mod/file CRUD | Already in project; typed queries |
| `tokio::fs` | (via tokio 1.x) | Async file operations | Already in project; used by `file_ops` |
| `@tanstack/react-query` | 5.x | Data fetching/mutations | Already in project; established pattern |
| `zustand` | 5.x | Client state (active game, UI state) | Already in project |
| `sonner` | 2.x | Toast notifications | Already in project |
| `lucide-react` | 0.577+ | Icons (toggle, warning, file, etc.) | Already in project |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/api` | 2.x | Drag-drop events, file dialog | Import flow: both drag-drop and file picker paths |
| `@tauri-apps/plugin-dialog` | 2.x | Native file open dialog | "Import" button click path |
| `radix-ui` | 1.4+ | Switch component for toggles | Mod toggle switches in card UI |

### Not Needed
| Instead of | Why Not |
|------------|---------|
| `zip-extract` crate | Deprecated; `ZipArchive::extract()` does the same. We iterate manually anyway for manifest building |
| `async-zip` | No need for async zip reading; files are local and small; `zip` crate is simpler |
| Custom path sanitization | `zip` crate's `enclosed_name()` handles ZipSlip; don't hand-roll |

**Installation (Rust):**
```toml
# Add to src-tauri/Cargo.toml [dependencies]
zip = { version = "8", default-features = false, features = ["deflate"] }
```

No new npm packages needed -- all frontend dependencies already installed.

## Architecture Patterns

### New Files Structure
```
src-tauri/src/
├── commands/
│   ├── mods.rs          # import_mod, toggle_mod, delete_mod, list_mods
│   └── conflicts.rs     # check_conflicts, get_mod_conflicts
├── services/
│   ├── import.rs         # Zip extraction + manifest building + sub-mod detection
│   └── toggle.rs         # Journal-wrapped toggle logic (enable/disable file moves)
├── db/
│   └── queries.rs        # Extended with mod CRUD, sub-mod queries, conflict queries

src/
├── components/
│   ├── ModList.tsx        # Main mod list container (replaces EmptyModView when mods exist)
│   ├── ModCard.tsx        # Individual mod card with toggle, expandable details
│   ├── ImportDialog.tsx   # Import confirmation dialog (name edit, file preview)
│   ├── ConflictDialog.tsx # Conflict warning dialog (Enable Anyway / Disable Other / Cancel)
│   └── SubModOptions.tsx  # Nested sub-mod toggles inside expanded card
├── hooks/
│   └── useMods.ts         # useMods, useImportMod, useToggleMod, useDeleteMod, useConflicts
```

### Pattern 1: Import Flow (Rust Backend)
**What:** Single Tauri command receives zip path + user-provided name, extracts to staging, records manifest
**When to use:** Always for import -- keeps all file I/O and DB work on the Rust side

```rust
// commands/mods.rs
#[tauri::command]
#[specta::specta]
pub async fn import_mod(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    zip_path: String,
    mod_name: String,
) -> Result<ImportResult, AppError> {
    // 1. Open zip, validate
    // 2. Create mod record in DB (disabled initially)
    // 3. Extract files to staging dir, recording each in file_entries
    // 4. Detect sub-mod options (Option_ prefix folders)
    // 5. Return ImportResult with mod record + warnings
}
```

### Pattern 2: Journal-Wrapped Toggle
**What:** Toggle operation writes a journal entry before moving files, marks each file done, then completes
**When to use:** Every enable/disable operation

```rust
// services/toggle.rs
pub async fn toggle_mod(
    app: &AppHandle,
    pool: &SqlitePool,
    mod_id: i64,
    enable: bool,
) -> Result<(), AppError> {
    let mod_rec = get_mod(pool, mod_id).await?;
    let game = get_game(pool, mod_rec.game_id).await?;
    let files = list_file_entries(pool, mod_id).await?;

    // Build FilePair list
    let pairs: Vec<FilePair> = files.iter().map(|f| {
        let staging = Path::new(&mod_rec.staged_path).join(&f.relative_path);
        let game_dir = Path::new(&game.mod_dir).join(&f.relative_path);
        if enable {
            FilePair { src: staging.display().to_string(), dst: game_dir.display().to_string(), done: false }
        } else {
            FilePair { src: game_dir.display().to_string(), dst: staging.display().to_string(), done: false }
        }
    }).collect();

    // Journal: begin -> move each -> complete
    let journal_id = begin_journal(pool, mod_id, if enable { "enable" } else { "disable" }, &pairs).await?;
    for pair in &pairs {
        file_ops::move_file(app, Path::new(&pair.src), Path::new(&pair.dst)).await?;
        mark_file_done(pool, journal_id, pair).await?;
    }
    complete_journal(pool, journal_id).await?;
    update_mod_enabled(pool, mod_id, enable).await?;
    Ok(())
}
```

### Pattern 3: Conflict Detection via SQL
**What:** Query file_entries table for path overlaps between mods of the same game
**When to use:** Before enabling a mod (pre-toggle check) and for persistent conflict badges

```sql
-- Find conflicts for a specific mod being enabled
SELECT
    fe2.mod_id AS conflicting_mod_id,
    m2.name AS conflicting_mod_name,
    fe1.relative_path
FROM file_entries fe1
JOIN file_entries fe2 ON fe1.relative_path = fe2.relative_path
    AND fe1.mod_id != fe2.mod_id
JOIN mods m2 ON fe2.mod_id = m2.id
WHERE fe1.mod_id = ?
    AND m2.enabled = 1
    AND m2.game_id = ?
ORDER BY fe1.relative_path;
```

### Pattern 4: Sub-Mod Detection at Import
**What:** During zip extraction, detect top-level directories matching `Option_*` or `option_*` pattern
**When to use:** Import time only; sub-mods recorded in DB

```rust
fn detect_sub_mods(entry_name: &str) -> Option<String> {
    let parts: Vec<&str> = entry_name.split('/').collect();
    // Look for a path component starting with Option_ or option_
    for part in &parts {
        if part.starts_with("Option_") || part.starts_with("option_") {
            return Some(part.to_string());
        }
    }
    None
}
```

### Pattern 5: Drag-and-Drop + File Picker (Frontend)
**What:** Two import entry points converge on the same import flow
**When to use:** Import UI

```typescript
import { listen, TauriEvent } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

// Drag-and-drop listener
const unlisten = await listen(TauriEvent.DRAG_DROP, (event) => {
  const paths = (event.payload as { paths: string[] }).paths
  const zipPaths = paths.filter(p => p.endsWith('.zip'))
  if (zipPaths.length > 0) {
    openImportDialog(zipPaths[0])
  }
})

// File picker button
async function handleImportClick() {
  const selected = await open({
    filters: [{ name: 'Zip Archives', extensions: ['zip'] }],
    multiple: false,
  })
  if (selected) {
    openImportDialog(selected as string)
  }
}
```

### Anti-Patterns to Avoid
- **Reading entire zip into memory:** Use streaming iteration (`by_index`) not `read_to_end` on the archive
- **Frontend file handling:** Never pass file bytes over IPC; pass the file path to the Rust backend
- **In-memory conflict detection:** Use SQL joins, not loading all file entries into Rust HashMaps
- **Splitting import into multiple IPC calls:** One atomic `import_mod` command does extract + DB insert; partial state is dangerous
- **Forgetting to handle Option_ sub-folders as distinct toggle units:** They share the parent mod's staging dir but have their own enable state

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Zip extraction | Custom byte-level parsing | `zip` crate `ZipArchive` | Format is complex (deflate, zip64, multi-part) |
| Path traversal protection | Regex/string checks on paths | `ZipFile::enclosed_name()` | Handles edge cases (null bytes, `..`, absolute paths, symlinks) |
| Cross-drive file move | Manual detect + copy + delete | `file_ops::move_file()` (already built) | Handles progress events, error mapping |
| Crash-safe toggle | Ad-hoc try/catch | Journal service (already built) | Enables recovery on next launch |
| Toast notifications | Custom notification system | Sonner (already installed) | Consistent with Phase 1 UX |
| File dialog | HTML file input | `@tauri-apps/plugin-dialog` `open()` | Native OS dialog, already installed |

## Common Pitfalls

### Pitfall 1: Zip Entry Iteration Includes Directories
**What goes wrong:** `ZipArchive::by_index()` returns both files AND directory entries. Writing a directory entry as a file creates a 0-byte file instead of a directory.
**Why it happens:** Zip format stores directories as entries with names ending in `/`.
**How to avoid:** Check `entry.is_dir()` before extracting; create directories with `create_dir_all`, only write file contents for non-directory entries.
**Warning signs:** 0-byte files appearing in staging where directories should be.

### Pitfall 2: Zip Paths Use Forward Slashes But Windows Uses Backslashes
**What goes wrong:** File manifest stored with `/` separators doesn't match OS path joins on Windows.
**Why it happens:** Zip standard uses `/` regardless of OS.
**How to avoid:** Normalize all `relative_path` entries in DB to use `/` (the zip standard). When constructing OS paths for file moves, use `Path::new()` which handles conversion. Never string-concat paths.
**Warning signs:** Files "not found" during toggle despite existing on disk.

### Pitfall 3: Sub-Mod Files Must Be Separate from Parent Mod Files
**What goes wrong:** If Option_ folder files are recorded in both the parent mod's file_entries AND the sub-mod's file_entries, toggling creates duplicate moves.
**Why it happens:** Naive import records every file under the parent mod.
**How to avoid:** Files inside `Option_*` directories belong ONLY to the sub-mod record, not the parent. The parent mod's file_entries exclude Option_ subtrees.
**Warning signs:** Files moved twice, or files reappearing after being disabled.

### Pitfall 4: Conflict Check Must Include Sub-Mods
**What goes wrong:** Conflict detection only checks parent mods, missing overlaps from enabled sub-mod options.
**Why it happens:** Sub-mods stored in separate table, forgotten in join query.
**How to avoid:** Conflict query must union parent mod files and enabled sub-mod files.
**Warning signs:** No conflict warning when two Option_ folders from different mods share the same file.

### Pitfall 5: Parent Toggle Must Remember Sub-Mod States
**What goes wrong:** Toggling parent off then on again loses which sub-mod options were individually enabled.
**Why it happens:** Naive implementation sets all sub-mod `enabled` to false on parent disable.
**How to avoid:** Add a `user_enabled` column to sub_mods that tracks user intent separately from effective state. When parent is disabled, effective state is off but `user_enabled` is preserved. When parent re-enables, restore sub-mods where `user_enabled = true`.
**Warning signs:** User enables specific options, toggles parent off/on, and all options reset to off.

### Pitfall 6: Race Condition on Rapid Toggle Clicks
**What goes wrong:** User double-clicks toggle, two toggle operations run concurrently on the same mod.
**Why it happens:** No mutex/lock on per-mod operations.
**How to avoid:** Optimistic UI disable (grey out toggle during operation). Backend: check if a journal entry is already in_progress for this mod_id before starting.
**Warning signs:** Corrupted journal entries, files in unexpected locations.

### Pitfall 7: Empty Staging Path After Import
**What goes wrong:** Import extracts to staging but the staged_path on the mod record doesn't include the mod-specific subdirectory.
**Why it happens:** `staged_path` set to game's staging dir instead of `staging_dir/mod_slug/`.
**How to avoid:** Each mod gets its own subdirectory under the game's staging dir: `{game.staging_dir}/{mod_slug}/`.
**Warning signs:** All mods' files dumped into the same staging folder, overwriting each other.

## Code Examples

### Zip Extraction with ZipSlip Protection
```rust
// Source: zip crate docs (docs.rs/zip/latest)
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn extract_zip_to_staging(
    zip_path: &Path,
    staging_dir: &Path,
) -> Result<Vec<String>, AppError> {
    let file = fs::File::open(zip_path)
        .map_err(|e| AppError::IoError(format!("Cannot open zip: {}", e)))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::IoError(format!("Invalid zip: {}", e)))?;

    let mut manifest = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| AppError::IoError(e.to_string()))?;

        // ZipSlip protection: enclosed_name returns None for unsafe paths
        let relative = match entry.enclosed_name() {
            Some(name) => name.to_owned(),
            None => continue, // Skip dangerous entries silently
        };

        let target = staging_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&target)?;
            io::copy(&mut entry, &mut outfile)?;

            // Record in manifest (forward-slash normalized)
            manifest.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(manifest)
}
```

### Sub-Mod Detection During Import
```rust
/// Given a list of file paths from the zip, group them into:
/// - main_files: files NOT under Option_* directories
/// - sub_mods: map of option_folder_name -> vec of relative paths
fn partition_files(paths: &[String]) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut main_files = Vec::new();
    let mut sub_mods: HashMap<String, Vec<String>> = HashMap::new();

    for path in paths {
        let parts: Vec<&str> = path.split('/').collect();
        let option_part = parts.iter().find(|p| {
            p.starts_with("Option_") || p.starts_with("option_")
        });

        match option_part {
            Some(opt_name) => {
                sub_mods.entry(opt_name.to_string())
                    .or_default()
                    .push(path.clone());
            }
            None => main_files.push(path.clone()),
        }
    }

    (main_files, sub_mods)
}
```

### React Query Hook Pattern for Mods
```typescript
// Following useGames pattern exactly
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { commands } from '../lib/tauri'

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: { kind: string; message: string } }): T {
  if (result.status === "ok") return result.data
  throw new Error(result.error.message)
}

export function useMods(gameId: number | null) {
  return useQuery({
    queryKey: ['mods', gameId],
    queryFn: async () => {
      if (!gameId) return []
      const result = await commands.listMods(gameId)
      return unwrap(result)
    },
    enabled: gameId !== null,
  })
}

export function useToggleMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { modId: number; enable: boolean }) => {
      const result = await commands.toggleMod(args.modId, args.enable)
      return unwrap(result)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Toggle failed')
    },
  })
}
```

## Database Schema Changes

### New Migration: Sub-Mods Table (version 5)
```sql
CREATE TABLE IF NOT EXISTS sub_mods (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    mod_id          INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    folder_name     TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 0,
    user_enabled    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sub_mods_mod_id ON sub_mods(mod_id);
```

### New Migration: Sub-Mod File Entries (version 6)
```sql
-- file_entries for sub-mod files reference the sub_mod_id
ALTER TABLE file_entries ADD COLUMN sub_mod_id INTEGER REFERENCES sub_mods(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_file_entries_sub_mod ON file_entries(sub_mod_id);
```

**Key design:** `file_entries.sub_mod_id` is nullable. When NULL, the file belongs to the parent mod. When set, the file belongs to that sub-mod. This avoids a separate file table while keeping queries simple.

### Conflict Detection Query
```sql
-- Check if enabling mod ? would conflict with any currently-enabled mod in game ?
SELECT
    fe_other.mod_id AS conflicting_mod_id,
    m_other.name AS conflicting_mod_name,
    fe_target.relative_path
FROM file_entries fe_target
JOIN file_entries fe_other
    ON fe_target.relative_path = fe_other.relative_path
    AND fe_target.mod_id != fe_other.mod_id
JOIN mods m_other ON fe_other.mod_id = m_other.id
WHERE fe_target.mod_id = ?1
    AND m_other.game_id = ?2
    AND m_other.enabled = 1
    AND (fe_other.sub_mod_id IS NULL OR EXISTS (
        SELECT 1 FROM sub_mods sm WHERE sm.id = fe_other.sub_mod_id AND sm.enabled = 1
    ))
ORDER BY fe_target.relative_path;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `zip-extract` wrapper crate | `ZipArchive::extract()` built into zip crate | Deprecated in favor of built-in | One fewer dependency |
| `zip` v2.x with CVE-2025-29787 | `zip` v8.x with built-in path sanitization | Fixed in 2.3.0, now at 8.x | `enclosed_name()` is safe; no manual path checking needed |
| Tauri v1 `file-drop` event | Tauri v2 `TauriEvent.DRAG_DROP` with `paths` + `position` | Tauri 2.0 release | Different event names and payload shape |

## Open Questions

1. **Elevated file operations for protected game directories**
   - What we know: `requires_elevation` column exists on games; UAC helper process concept identified in Phase 1
   - What's unclear: Exact mechanism for spawning elevated helper to move files into Program Files directories
   - Recommendation: For Phase 2, implement toggle for non-elevated paths first. Add a TODO/error path for elevated games ("This game requires administrator access -- feature coming soon") or implement basic `runas` spawning of a helper binary. The architecture note in STATE.md flags this as needing deeper research.

2. **Duplicate drag-drop events (Tauri bug #14134)**
   - What we know: Known Tauri v2 bug where drag-drop events fire twice with different IDs
   - What's unclear: Whether this is fixed in the latest Tauri 2.x release
   - Recommendation: Add debounce/dedup guard on the drag-drop handler (ignore events for the same file path within 500ms)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` with `tokio::test` for async, `tempfile` for fixtures |
| Framework (Frontend) | Vitest 4.x + jsdom + @testing-library/react |
| Config file (Frontend) | `vitest.config.ts` |
| Quick run command | `npx vitest run --reporter=verbose` / `cargo test -p modtoggler` |
| Full suite command | `npx vitest run && cargo test -p modtoggler` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IMPORT-01 | Import mod from zip | unit (Rust) | `cargo test -p modtoggler import` | No - Wave 0 |
| IMPORT-02 | Extract + record manifest | unit (Rust) | `cargo test -p modtoggler extract` | No - Wave 0 |
| IMPORT-03 | Auto-group .pak/.ucas/.utoc | unit (Rust) | `cargo test -p modtoggler group` | No - Wave 0 |
| IMPORT-04 | Detect sub-mod Option_ folders | unit (Rust) | `cargo test -p modtoggler sub_mod` | No - Wave 0 |
| IMPORT-05 | Show file manifest in UI | unit (TS) | `npx vitest run src/components/ModCard` | No - Wave 0 |
| IMPORT-06 | ZipSlip protection | unit (Rust) | `cargo test -p modtoggler zipslip` | No - Wave 0 |
| TOGGLE-01 | Toggle on/off | unit (Rust) | `cargo test -p modtoggler toggle` | No - Wave 0 |
| TOGGLE-02 | Disable moves files to staging | unit (Rust) | `cargo test -p modtoggler disable` | No - Wave 0 |
| TOGGLE-03 | Enable moves files to game dir | unit (Rust) | `cargo test -p modtoggler enable` | No - Wave 0 |
| TOGGLE-05 | Sub-mod independent toggle | unit (Rust) | `cargo test -p modtoggler sub_mod_toggle` | No - Wave 0 |
| TOGGLE-07 | Delete mod + files | unit (Rust) | `cargo test -p modtoggler delete` | No - Wave 0 |
| CONFLICT-01 | Detect overlapping files | unit (Rust) | `cargo test -p modtoggler conflict` | No - Wave 0 |
| CONFLICT-02 | Display conflicting mods + files | unit (TS) | `npx vitest run src/components/ConflictDialog` | No - Wave 0 |
| CONFLICT-03 | Warning on enable with conflict | integration | Manual - requires Tauri runtime | No |

### Sampling Rate
- **Per task commit:** `cargo test -p modtoggler` + `npx vitest run`
- **Per wave merge:** Full suite: `cargo test -p modtoggler && npx vitest run`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/services/import.rs` -- extraction + manifest tests (IMPORT-01 through IMPORT-06)
- [ ] `src-tauri/src/services/toggle.rs` -- toggle logic tests (TOGGLE-01 through TOGGLE-05)
- [ ] `src/components/ModCard.test.tsx` -- mod card rendering tests
- [ ] `src/components/ConflictDialog.test.tsx` -- conflict dialog tests
- [ ] `src/hooks/useMods.test.ts` -- hook tests following useGames.test.ts pattern
- [ ] Test zip fixtures: create small test .zip files with known structure for Rust tests

## Sources

### Primary (HIGH confidence)
- [zip crate v8.x docs](https://docs.rs/zip/latest/zip/) -- ZipArchive API, extract method, enclosed_name
- [ZipFile::enclosed_name](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html) -- ZipSlip protection API
- [Tauri v2 event API](https://v2.tauri.app/reference/javascript/api/namespaceevent/) -- DragDrop event types
- Existing codebase: `file_ops.rs`, `journal.rs`, `queries.rs`, `useGames.ts` -- established patterns

### Secondary (MEDIUM confidence)
- [CVE-2025-29787](https://nvd.nist.gov/vuln/detail/CVE-2025-29787) -- Zip path traversal fixed in 2.3.0; current v8.x is safe
- [Tauri drag-drop bug #14134](https://github.com/tauri-apps/tauri/issues/14134) -- Duplicate events issue; may need workaround

### Tertiary (LOW confidence)
- Elevated helper binary pattern -- Not yet researched deeply; flagged as open question

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- using existing project dependencies plus well-established `zip` crate
- Architecture: HIGH -- following established patterns from Phase 1 (commands/services/queries split, React Query hooks)
- Pitfalls: HIGH -- common zip extraction and file management issues well-documented
- Sub-mod design: MEDIUM -- DB schema design is sound but the `Option_` prefix detection pattern is domain-specific
- Elevated operations: LOW -- UAC helper mechanism not yet designed

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable domain, no fast-moving dependencies)
