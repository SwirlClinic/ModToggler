# Phase 3: Profiles - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can save the current mod enabled/disabled state as a named profile, load a saved profile to restore that state, and delete profiles. Profiles are scoped per game. No profile import/export (that's v2).

</domain>

<decisions>
## Implementation Decisions

### Profile Save/Load UX
- Dropdown in the ModList header bar (next to Import button) — labeled with last-loaded profile name
- Dropdown shows: saved profile names, separator, "Save Current...", "Manage Profiles"
- Clicking a profile name = immediate load (no confirmation dialog)
- "Save Current..." opens a small dialog with name input, Cancel/Save buttons
- Toast confirms save and load actions
- Current state is NOT auto-saved — user saves explicitly if they want to keep it

### Profile Apply Behavior
- Loading a profile batch-toggles all mods to match the saved state
- If a profile references mods that were deleted: skip missing mods, apply the rest, show toast listing which mods were skipped
- Profile data stays saved as-is even if some mods no longer exist

### Profile Naming & Management
- Names must be unique per game — saving with an existing name overwrites (with confirmation: "Overwrite existing profile 'X'?")
- "Manage Profiles" opens a dialog/panel listing all profiles for the current game with delete buttons
- Delete requires confirmation dialog
- Dropdown label shows last-loaded profile name; no "modified" tracking if user manually toggles mods after loading

### Active Profile Indicator
- Dropdown button shows the last-loaded profile name (e.g. "Tournament Setup")
- If no profile has been loaded this session, shows "Profiles"
- No modified/dirty state tracking — keeps it simple

### Claude's Discretion
- Exact dropdown component implementation (Radix popover, custom, etc.)
- Profile data storage schema (DB table structure)
- Batch toggle optimization (whether to use a single transaction or per-mod toggles)
- Manage Profiles dialog layout
- Animation/transition for profile apply

</decisions>

<specifics>
## Specific Ideas

- Same "quick, don't over-complicate" principle — save is one dialog, load is one click
- Profiles dropdown in the header keeps it visible and accessible without navigating away from the mod list

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Dialog` component (src/components/ui/dialog.tsx): Reuse for save and manage dialogs
- `Select` component (src/components/ui/select.tsx): Possible base for profile dropdown
- `Sonner` toasts (src/components/ui/sonner.tsx): Confirm save/load/delete actions
- `Button`, `Input`, `Label` UI primitives all available
- `useMods` hook (src/hooks/useMods.ts): Returns mod list with enabled state — profile save reads this
- `useToggleMod` mutation: Profile load calls this per mod to apply state

### Established Patterns
- React Query + tauri-specta typed bindings (useGames, useMods pattern)
- SQLite via sqlx::SqlitePool with AppError handling
- Commands in src-tauri/src/commands/, services in src-tauri/src/services/
- ModList header bar already has game name + Import button — add Profiles dropdown alongside

### Integration Points
- `ModList.tsx` header: add Profiles dropdown next to Import button
- `db/queries.rs`: add profile CRUD queries
- `commands/`: new profiles.rs for Tauri commands
- `hooks/`: new useProfiles.ts following useMods pattern
- Game ID scoping: profiles table FK to games(id), same pattern as mods table

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-profiles*
*Context gathered: 2026-03-05*
