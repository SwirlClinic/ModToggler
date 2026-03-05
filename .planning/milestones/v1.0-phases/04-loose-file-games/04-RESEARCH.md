# Phase 4: Loose-File Games - Research

**Researched:** 2026-03-05
**Domain:** Extending existing Tauri/Rust + React mod manager to support loose-file (non-PAK) game mods
**Confidence:** HIGH

## Summary

Phase 4 extends ModToggler to support games where mods are individual files scattered across the game directory (not structured PAK/UCAS/UTOC groups). The codebase is well-prepared: `mod_structure` column already exists on `games` table with CHECK constraint for "structured"|"loose", GameForm already has a mod structure selector, and the toggle/journal infrastructure is fully reusable. The core work is: (1) adding a `destination_path` column to `file_entries` so each file knows where it goes relative to game root, (2) creating new import flows (manual file picker + zip with file selection), (3) adapting toggle logic to use per-file destination paths instead of uniform mod_dir, and (4) frontend components for file mapping and loose-file management.

This is primarily a feature extension, not new architecture. Every layer (DB, services, commands, hooks, components) needs additions, but the patterns are established and consistent across 12 completed plans.

**Primary recommendation:** Add `destination_path` column to `file_entries` table (nullable, NULL = structured mod behavior), then branch import/toggle logic based on game.mod_structure. Keep all existing structured-mod code paths unchanged.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Two import methods for loose-file games: manual file picker (multi-select) AND zip import
- Manual file picker: standard OS multi-select dialog, all selected files added to the same mod
- Zip import: extracts files, shows all files in a list, user selectively checks which files to include
- Mod name pre-filled from zip filename (for zip) or user-entered (for manual), editable before confirming
- Files are copied to staging/[modname]/ on import (consistent with structured mods)
- Users can add more files to an existing loose-file mod after initial import
- Users can remove individual files from a loose-file mod without deleting the whole mod
- Table with filename | destination path columns -- user types or pastes the relative path
- Default destination path is "/" (game root) -- user edits to add subdirectory if needed
- Multi-select files + bulk set destination path supported (check multiple rows, set path for all)
- File mapping table shown during import flow (before confirming mod creation)
- mod_structure setting lives in GameForm (add/edit game) -- toggle or dropdown for "structured" vs "loose" (ALREADY DONE)
- Switchable anytime -- not locked at creation
- When switching mode with existing mods: existing mods keep their original type, new imports use the new mode (mixed mod types per game)
- Subtle "Loose" badge/label near the game name when viewing a loose-file game
- Toggle ON: move files from staging to their mapped destination paths (same journal safety as structured)
- Auto-create missing destination directories when enabling a loose-file mod
- Toggle OFF: move files back to staging, leave empty directories in place (don't clean up)
- No sub-mods for loose-file mods -- all files toggle together as one unit
- Conflict detection by exact destination path match (two mods conflict if any files map to the same destination)

### Claude's Discretion
- File mapping table component design and editing interaction
- How "add files to existing mod" is accessed in the UI (button in expanded card, etc.)
- How "remove file from mod" works in the UI (delete icon per row, etc.)
- Import dialog layout adaptation for loose-file mode
- Badge/label styling for loose-file games

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOOSE-01 | User can add a game configured for loose-file mod structure | GameForm already has mod_structure selector; games table already has mod_structure column. Needs "Loose" badge on GameSelector. |
| LOOSE-02 | User can manually tag which files belong to a mod when importing for loose-file games | New import flows (file picker + zip with checkboxes), new import_loose_mod command, file mapping table component |
| LOOSE-03 | User can specify destination paths for each file relative to the game root | destination_path column on file_entries, file mapping table with editable paths, bulk path editing |
| LOOSE-04 | Toggling works the same way for loose-file mods (move to/from staging) | Extended toggle_mod to use destination_path per file instead of uniform mod_dir join, journal infrastructure unchanged |

</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 | 2.x | Desktop app framework | Already in use |
| sqlx | rc.22 | SQLite database | Already in use, migration system established |
| zip | 8.x | Zip extraction | Already in use for structured imports |
| React | 18/19 | UI framework | Already in use |
| TanStack React Query | 5.x | Server state management | Already in use, unwrap pattern established |
| tauri-specta | rc.21 | Typed command bindings | Already in use, auto-generates bindings.ts |
| shadcn/ui | latest | UI components | Already in use (Dialog, Button, Input, Select, Card, etc.) |
| @tauri-apps/plugin-dialog | 2.x | File/folder picker dialogs | Already in use for folder picking |

### New Dependencies Needed
| Library | Purpose | Notes |
|---------|---------|-------|
| shadcn/ui checkbox | Checkbox for file selection in zip import | May already be available, check components/ui/ |

### No New External Dependencies Required
The entire phase can be built with existing libraries. The `@tauri-apps/plugin-dialog` already supports `open({ multiple: true })` for multi-file selection. Shadcn/ui provides all needed UI primitives.

## Architecture Patterns

### Database Schema Extension

**Migration 8: Add destination_path to file_entries**
```sql
ALTER TABLE file_entries ADD COLUMN destination_path TEXT;
```

- `NULL` for structured mods (relative_path is used directly under mod_dir, existing behavior unchanged)
- Non-NULL for loose-file mods (e.g., `"bin/scripts"`, `"data/textures"`, `"/"` for game root)
- No need for a separate table -- extending file_entries keeps the schema simple and queries unified

**Why nullable:** Existing structured mods have no destination_path. Using NULL preserves backward compatibility with zero data migration needed. Toggle logic branches: if destination_path is NULL, use existing mod_dir join; if non-NULL, use game root + destination_path + filename.

### Mod Type Tracking

The `mod_structure` field on GameRecord controls which import flow to show. However, per the user decision, existing mods keep their type when the game mode switches. This means we need per-mod type awareness.

**Option A (recommended):** Add `mod_type` column to `mods` table ("structured"|"loose"). Set at import time based on current game.mod_structure. This lets toggle and delete know how to handle each mod regardless of current game setting.

**Option B:** Infer from file_entries -- if any file has non-NULL destination_path, it's a loose mod. Simpler schema but fragile inference.

**Migration 8 should include both:**
```sql
ALTER TABLE file_entries ADD COLUMN destination_path TEXT;
ALTER TABLE mods ADD COLUMN mod_type TEXT NOT NULL DEFAULT 'structured' CHECK(mod_type IN ('structured', 'loose'));
```

### Toggle Logic Extension

Current `build_file_pairs` joins `relative_path` to `src_base`/`dst_base`. For loose-file mods:
- **Staging side:** Files stored flat in `staging/[modname]/` with their original filename. The `relative_path` in file_entries is just the filename.
- **Game side:** Files go to `game_root + destination_path + filename`.

New function needed: `build_loose_file_pairs(entries, staging_base, game_root)` where each entry uses its `destination_path` to compute the game-side path.

```rust
pub fn build_loose_file_pairs(
    entries: &[FileEntry],  // with destination_path populated
    staging_base: &Path,    // staging/[modname]/
    game_root: &Path,       // game mod_dir (actually game root for loose files)
) -> Vec<FilePair> {
    entries.iter().map(|entry| {
        let filename = Path::new(&entry.relative_path)
            .file_name()
            .unwrap_or_default();
        let src = staging_base.join(&entry.relative_path);
        let dest_dir = entry.destination_path.as_deref().unwrap_or("/");
        let dest_dir = dest_dir.trim_start_matches('/');
        let dst = game_root.join(dest_dir).join(filename);
        FilePair { src: src.display().to_string(), dst: dst.display().to_string(), done: false }
    }).collect()
}
```

### Import Flow Architecture

**Manual file picker flow:**
1. User clicks "Import" on a loose-file game
2. OS multi-file dialog opens (`@tauri-apps/plugin-dialog` with `multiple: true`)
3. Selected files shown in file mapping table (filename | destination path)
4. User edits destination paths, enters mod name
5. On confirm: backend copies files to staging/[modname]/, creates mod + file_entries with destination_path

**Zip import flow for loose games:**
1. Same zip selection as structured (drag-drop or file picker)
2. Files extracted to temp location first (or staging), then shown in checklist
3. User checks which files to include, sets destination paths
4. On confirm: checked files copied to staging/[modname]/, unchecked files deleted

**New Tauri commands needed:**
- `import_loose_files(game_id, mod_name, files: Vec<LooseFileInput>)` where LooseFileInput = { source_path, destination_path }
- `import_loose_zip(game_id, zip_path, mod_name, selected_files: Vec<LooseFileInput>)`
- `add_files_to_mod(mod_id, files: Vec<LooseFileInput>)` -- add more files to existing loose mod
- `remove_file_from_mod(file_entry_id)` -- remove single file from a loose mod

### Frontend Component Structure

```
src/
  components/
    LooseImportDialog.tsx      # New: wraps both manual + zip import for loose games
    FileMapTable.tsx           # New: editable table (filename | destination path | checkbox)
    ModCard.tsx                # Extended: show destination paths, add/remove file buttons for loose mods
    GameSelector.tsx           # Extended: "Loose" badge
    ImportDialog.tsx           # Unchanged (structured-only)
    ModList.tsx                # Extended: branch import dialog based on game.mod_structure
  hooks/
    useMods.ts                 # Extended: new mutations for loose import, add/remove files
```

### Conflict Detection for Loose Mods

Current conflict detection matches on `relative_path`. For loose mods, the effective destination is `destination_path + filename`, not `relative_path`. Two approaches:

**Recommended:** Store a computed `effective_destination` in file_entries (or compute it in the conflict query). The conflict query for loose mods should compare `destination_path || '/' || filename` across mods.

**Simpler approach:** Store the full effective path as `relative_path` for loose mods (e.g., `"bin/scripts/config.ini"`) and use `destination_path` only for the directory part. Then the existing conflict detection on `relative_path` works for both structured and loose mods.

**Best approach:** For loose mods, store relative_path as just the filename within staging (for locating the file in staging), and add a query that computes the full destination for conflict checking. Or store the full destination in relative_path and keep destination_path as the directory-only component. Either way, conflict detection needs to compare on the effective game-side path.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multi-file selection | Custom file tree | `@tauri-apps/plugin-dialog` `open({ multiple: true })` | OS-native dialog handles permissions, UX, and platform differences |
| Zip extraction | Custom decompression | Existing `import::extract_zip_to_staging` | Already handles ZipSlip, forward-slash normalization, directory creation |
| File copy to staging | Manual fs operations | `file_ops::move_file` or `tokio::fs::copy` | Cross-drive handling already solved |
| Journal safety | Custom transaction log | Existing `journal_move_files` | Already crash-safe, tested, proven in 12 plans |

## Common Pitfalls

### Pitfall 1: Dual Path Semantics
**What goes wrong:** Confusing relative_path (location in staging) with destination_path (location in game directory). Structured mods use relative_path for both; loose mods use them differently.
**Why it happens:** The existing codebase assumes relative_path doubles as the game-side path.
**How to avoid:** Clear naming: `staging_relative_path` vs `game_destination_path`. Or keep relative_path as staging-relative and always consult destination_path for game-side location.
**Warning signs:** Files ending up in wrong directories during toggle.

### Pitfall 2: Conflict Detection Divergence
**What goes wrong:** Structured mod conflicts use relative_path comparison. Loose mod conflicts need destination_path + filename comparison. If the conflict query isn't updated, loose mods will never show conflicts.
**Why it happens:** The check_conflicts query only compares relative_path.
**How to avoid:** Update check_conflicts to compare effective game-side paths. For structured mods this is relative_path; for loose mods it's destination_path + filename.

### Pitfall 3: Mixed Mod Types Per Game
**What goes wrong:** A game can have both structured and loose mods simultaneously (user changed mode mid-stream). Toggle, delete, and conflict detection must handle both types correctly.
**Why it happens:** User decision explicitly allows switching modes with existing mods.
**How to avoid:** Always check mod_type per-mod, never assume all mods in a game are the same type.

### Pitfall 4: File Naming Collisions in Staging
**What goes wrong:** Two files with the same name but different source directories imported to the same loose mod will collide in staging.
**Why it happens:** Staging layout is flat: staging/[modname]/filename.
**How to avoid:** Use a unique staging path per file -- either preserve source directory structure in staging, or use a hash/counter suffix. Simplest: preserve the source filename but add numeric suffix on collision.

### Pitfall 5: Empty Destination Path Handling
**What goes wrong:** User leaves destination_path empty or as "/". Code needs to handle "/" as "game root" without creating a path like `/game_dir//filename`.
**Why it happens:** Path joining with empty or "/" strings varies by OS.
**How to avoid:** Trim leading "/" from destination_path, treat empty and "/" as "game root" (join directly to game_root).

### Pitfall 6: bindings.ts Must Be Regenerated
**What goes wrong:** New commands and types added in Rust but not reflected in TypeScript bindings.
**Why it happens:** tauri-specta generates bindings.ts at build time.
**How to avoid:** Run `cargo build` (which triggers specta export) after adding new commands, before writing frontend code. The established project pattern handles this.

## Code Examples

### Migration 8: Schema Extension
```sql
-- Migration version 8: add_loose_file_support
ALTER TABLE file_entries ADD COLUMN destination_path TEXT;
ALTER TABLE mods ADD COLUMN mod_type TEXT NOT NULL DEFAULT 'structured'
    CHECK(mod_type IN ('structured', 'loose'));
```

### New Record Types
```rust
// Input type for loose file import
#[derive(Debug, Deserialize, Type)]
pub struct LooseFileInput {
    pub source_path: String,       // absolute path to source file
    pub destination_path: String,  // relative path under game root (e.g., "bin/scripts")
    pub file_name: String,         // original filename
}
```

### Extended Toggle for Loose Mods
```rust
// In toggle_mod, after getting mod_rec:
let mod_type = &mod_rec.mod_type; // "structured" or "loose"
if mod_type == "loose" {
    // Use destination_path per file entry
    let pairs = build_loose_file_pairs(&all_entries, staging, game_dir);
    journal_move_files(app, pool, mod_id, operation, &pairs).await?;
} else {
    // Existing structured logic unchanged
    let parent = parent_entries(&all_entries);
    let pairs = build_file_pairs(&parent, staging, game_dir);
    journal_move_files(app, pool, mod_id, operation, &pairs).await?;
}
```

### File Mapping Table Component Pattern
```tsx
// FileMapTable.tsx - editable table for loose file destination paths
interface FileMapping {
  fileName: string
  sourcePath: string
  destinationPath: string
  selected: boolean
}

function FileMapTable({ files, onChange }: {
  files: FileMapping[]
  onChange: (files: FileMapping[]) => void
}) {
  // Table with: [checkbox] | filename | editable destination path input
  // Multi-select + bulk set destination path
}
```

### Loose Import Command
```rust
#[tauri::command]
#[specta::specta]
pub async fn import_loose_files(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    mod_name: String,
    files: Vec<LooseFileInput>,
) -> Result<ImportResult, AppError> {
    let game = queries::get_game(&pool, game_id).await?;
    let slug = slug_from_name(&mod_name);
    let mod_staging = PathBuf::from(&game.staging_dir).join(&slug);
    std::fs::create_dir_all(&mod_staging)?;

    // Copy each file to staging
    for file in &files {
        let src = PathBuf::from(&file.source_path);
        let dst = mod_staging.join(&file.file_name);
        tokio::fs::copy(&src, &dst).await?;
    }

    // Insert mod record with mod_type="loose"
    let mod_record = queries::insert_mod(&pool, game_id, &mod_name,
        &mod_staging.display().to_string()).await?;
    // TODO: also set mod_type on mod record

    // Insert file entries with destination_path
    for file in &files {
        queries::insert_file_entry_with_destination(
            &pool, mod_record.id, &file.file_name,
            None, Some(&file.destination_path)
        ).await?;
    }

    Ok(ImportResult { /* ... */ })
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| mod_structure on game only | mod_type on individual mods | Phase 4 | Mixed mod types per game supported |
| uniform mod_dir for all files | per-file destination_path | Phase 4 | Files can go to arbitrary game subdirectories |
| Single import flow | Branched import (structured vs loose) | Phase 4 | Different UX per game mode |

## Open Questions

1. **Staging layout for loose files with name collisions**
   - What we know: Files are copied to staging/[modname]/. If two files have the same name but different source paths, they collide.
   - What's unclear: How common this is in practice.
   - Recommendation: Use relative_path in staging as `{counter}_{filename}` on collision, or preserve source directory structure. Simplest: detect collision and append numeric suffix.

2. **Integrity scan for loose mods**
   - What we know: Existing integrity scan checks if files exist at mod_dir/relative_path (for enabled mods) or staging/relative_path (for disabled mods).
   - What's unclear: Need to verify integrity scan handles destination_path correctly.
   - Recommendation: Update integrity scan to use destination_path for loose mods when checking game-side file existence.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust unit tests (cargo test) + Vitest (frontend) |
| Config file | src-tauri/Cargo.toml (Rust), vitest not yet configured for frontend |
| Quick run command | `cd src-tauri && cargo test` |
| Full suite command | `cd src-tauri && cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOOSE-01 | Game with mod_structure="loose" can be created/edited | unit | `cd src-tauri && cargo test queries::tests::test_insert_game_loose -x` | Partially (game CRUD tests exist, need loose-specific) |
| LOOSE-02 | Loose file import creates mod + file_entries with destination_path | unit | `cd src-tauri && cargo test commands::mods::tests::test_import_loose -x` | No - Wave 0 |
| LOOSE-03 | File entries store destination_path correctly | unit | `cd src-tauri && cargo test queries::tests::test_file_entry_destination_path -x` | No - Wave 0 |
| LOOSE-04 | Toggle builds correct file pairs using destination_path | unit | `cd src-tauri && cargo test toggle::tests::test_build_loose_file_pairs -x` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test`
- **Per wave merge:** `cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/services/toggle.rs` -- add test_build_loose_file_pairs test
- [ ] `src-tauri/src/db/queries.rs` -- add test for insert_file_entry with destination_path
- [ ] Migration version 8 test in `src-tauri/src/db/migrations.rs` -- verify migration count and version uniqueness updated

## Sources

### Primary (HIGH confidence)
- Project codebase analysis -- direct reading of all relevant source files
- CONTEXT.md user decisions -- locked implementation decisions
- Existing migration history (versions 1-7) -- schema evolution pattern

### Secondary (MEDIUM confidence)
- @tauri-apps/plugin-dialog API -- `open({ multiple: true })` for multi-file selection (verified from existing usage patterns in codebase)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all patterns established in prior phases
- Architecture: HIGH - clear extension points identified in every layer, schema change is minimal
- Pitfalls: HIGH - based on direct code analysis of toggle/import/conflict code paths
- Validation: MEDIUM - test infrastructure exists for Rust, frontend tests sparse

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable -- internal project, no external dependency changes)
