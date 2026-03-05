# Phase 2: Core Mod Loop - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can import mods from .zip archives, toggle them on/off with one click, and receive conflict warnings when two mods share overlapping files. Sub-mod option folders are detected at import and toggleable independently. Profiles and loose-file game support are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Zip Import Flow
- Drag-and-drop onto mod list area OR click "Import" button to open file picker — both paths available
- Multi-mod zips (multiple .pak groups) imported as a single mod — no auto-splitting
- Mod name pre-filled from zip filename, editable by user before confirming import
- Unexpected zip structures (no .pak files, random files): import with warning ("No recognized mod files found"), don't block
- ZipSlip protection required (IMPORT-06)

### Mod List & Toggle UX
- Card-based layout with toggle switch per mod (reuses existing Card component from Phase 1)
- Each card shows: mod name, toggle switch, file count, enabled/disabled status
- Clicking a card (not the toggle) expands it inline to show file manifest and sub-mod options
- Import button positioned top-right in the header area
- Delete button appears inside the expanded card details
- Sort order: enabled mods first, then alphabetical within each group

### Conflict Detection & Warnings
- Conflict detected by exact file path match (relative_path in mod_files table)
- When enabling a mod that conflicts: warning dialog shows conflicting mod names + overlapping file paths
- Dialog offers three actions: "Enable Anyway", "Disable Other", "Cancel"
- Last-enabled wins on overlap — newly enabled mod's files overwrite in game directory; overwritten files remain in the other mod's staging
- Persistent warning badge on mod cards that are currently in conflict — clicking badge shows conflict details

### Sub-mod Options
- Detected at import by `Option_` or `option_` prefix pattern (e.g. `Option_ExampleMod_ColorTexture/`)
- Displayed nested inside expanded parent card under "Options" section with individual toggles
- Multiple options can be enabled simultaneously (no radio/exclusive behavior)
- When parent mod is toggled OFF, all options are disabled too — individual option state is remembered and restored when parent is re-enabled

### Claude's Discretion
- Import progress indicator design (progress bar, spinner, etc.)
- Exact expanded card layout and spacing
- Empty state for new game with no mods (carries over EmptyModView pattern from Phase 1)
- Toggle animation/transition details
- Error handling for corrupted zips, permission failures during extract
- Elevated helper binary integration (UAC detection hooks from Phase 1 ready)

</decisions>

<specifics>
## Specific Ideas

- Import should feel quick — name, pick zip, confirm, done. Same "don't over-complicate the first interaction" principle from Phase 1 game setup
- Conflict dialog should name both mods and list the exact overlapping files — no vague "there's a conflict" message
- Expanded card view with nested options keeps everything in one place without context switching

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Card` component (src/components/ui/card.tsx): Shadow/rounded variants, reuse for mod cards
- `Dialog` component (src/components/ui/dialog.tsx): Reuse for conflict warning dialog and import confirmation
- `Sonner` toasts (src/components/ui/sonner.tsx): Use for import success/failure notifications
- `EmptyModView` (src/components/EmptyModView.tsx): Existing empty state for when no mods are imported
- `useGames` hook pattern (src/hooks/useGames.ts): Follow same React Query + unwrap pattern for useMods, useImportMod, etc.

### Established Patterns
- Tauri IPC via tauri-specta typed bindings (src/lib/tauri.ts)
- SQLite via sqlx::SqlitePool — direct pool access in Rust commands
- `AppError` tagged union with `{kind, message}` shape for frontend error handling
- `file_ops::move_file()` with cross-drive fallback and progress events
- Journal service (`FilePair`, `IncompleteJournalEntry`) for crash-safe toggle operations
- Dark theme only, set on HTML element statically

### Integration Points
- `ModRecord` and `FileEntry` types already defined in db/queries.rs — need new queries for CRUD
- `toggle_journal` table ready for toggle operations (begin_toggle, mark_file_done, complete)
- `commands/` module needs new files: mods.rs (import/toggle/delete), conflicts.rs
- Game selector already passes selected game ID — mod list hooks receive game_id as parameter
- `requires_elevation` column on games table — Phase 2 needs to implement elevated file ops

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-core-mod-loop*
*Context gathered: 2026-03-05*
