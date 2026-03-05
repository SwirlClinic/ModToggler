# Phase 4: Loose-File Games - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can manage mods for games where mod files are scattered across the game root (no standard .pak structure). Users manually pick files, tag which belong to a mod, and specify destination paths relative to the game root. Toggling moves files to/from staging using the same journal-safe infrastructure as structured mods.

</domain>

<decisions>
## Implementation Decisions

### Import Flow
- Two import methods for loose-file games: manual file picker (multi-select) AND zip import
- Manual file picker: standard OS multi-select dialog, all selected files added to the same mod
- Zip import: extracts files, shows all files in a list, user selectively checks which files to include
- Mod name pre-filled from zip filename (for zip) or user-entered (for manual), editable before confirming
- Files are copied to staging/[modname]/ on import (consistent with structured mods)
- Users can add more files to an existing loose-file mod after initial import
- Users can remove individual files from a loose-file mod without deleting the whole mod

### File Mapping UX
- Table with filename | destination path columns — user types or pastes the relative path
- Default destination path is "/" (game root) — user edits to add subdirectory if needed
- Multi-select files + bulk set destination path supported (check multiple rows, set path for all)
- File mapping table shown during import flow (before confirming mod creation)

### Game Mode Switch
- mod_structure setting lives in GameForm (add/edit game) — toggle or dropdown for "structured" vs "loose"
- Switchable anytime — not locked at creation
- When switching mode with existing mods: existing mods keep their original type, new imports use the new mode (mixed mod types per game)
- Subtle "Loose" badge/label near the game name when viewing a loose-file game

### Toggle Behavior
- Toggle ON: move files from staging to their mapped destination paths (same journal safety as structured)
- Auto-create missing destination directories when enabling a loose-file mod
- Toggle OFF: move files back to staging, leave empty directories in place (don't clean up)
- No sub-mods for loose-file mods — all files toggle together as one unit
- Conflict detection by exact destination path match (two mods conflict if any files map to the same destination)

### Claude's Discretion
- File mapping table component design and editing interaction
- How "add files to existing mod" is accessed in the UI (button in expanded card, etc.)
- How "remove file from mod" works in the UI (delete icon per row, etc.)
- Import dialog layout adaptation for loose-file mode
- Badge/label styling for loose-file games

</decisions>

<specifics>
## Specific Ideas

- Same "quick, don't over-complicate" principle — pick files, set paths, done
- File mapping during import should feel like a simple spreadsheet: filename on the left, editable path on the right
- Mixed mod types per game means the mod list can show both structured and loose mods side by side

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GameForm` (src/components/GameForm.tsx): Already captures game name/paths — add mod_structure field here
- `ImportDialog` (src/components/ImportDialog.tsx): Existing zip import flow — adapt for loose-file variant
- `ModCard` (src/components/ModCard.tsx): Expanded view shows file manifest — extend for destination paths
- `file_ops::move_file()`: Cross-drive safe file moves with journal — reuse for loose-file toggle
- `services/toggle.rs`: Toggle infrastructure — extend to handle destination path per file
- `services/import.rs`: Zip extraction — reuse for loose-file zip import path

### Established Patterns
- `mod_structure` column already exists on `GameRecord` ("structured" | "loose") — Phase 1 established this
- `file_entries` table stores relative_path per file — extend with destination_path column for loose files
- Flat staging layout: staging/[modname]/ — same for loose-file mods
- React Query + tauri-specta typed bindings for all data fetching

### Integration Points
- `GameForm.tsx`: Add mod_structure selector (toggle/dropdown)
- `GameSelector.tsx`: Show "Loose" badge when game.mod_structure === "loose"
- `ImportDialog.tsx`: Branch flow based on game.mod_structure
- `ModCard.tsx`: Show destination paths in expanded file manifest for loose mods
- `db/queries.rs`: Add destination_path to file_entries or new loose_file_entries table
- `commands/mods.rs`: New command for adding/removing individual files from a mod

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-loose-file-games*
*Context gathered: 2026-03-05*
