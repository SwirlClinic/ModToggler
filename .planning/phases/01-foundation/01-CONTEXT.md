# Phase 1: Foundation - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Game configuration UI, app-managed staging setup, atomic file operations infrastructure (transaction journal, crash recovery), and reliability primitives (startup integrity scan, cross-drive handling, permission management). No mod import or toggle UI in this phase — just the infrastructure they depend on.

</domain>

<decisions>
## Implementation Decisions

### Game Setup Flow
- Folder picker dialog OR paste-path text field — both options available (picker button next to text input)
- Game entry captures: name, mod directory path, mod structure type (structured vs loose-file)
- No auto-detection of Steam library games — manual add only (auto-detect deferred to future)
- When adding a game with an existing mod directory, offer to scan for existing mods (auto-detect .pak/.ucas/.utoc groups)

### Staging Location
- Default: each game gets its own staging folder inside an app-managed games directory (e.g. ~/.modtoggler/games/[game-name]/staging/)
- User can override and choose a custom staging folder per game
- Same-drive staging preferred by default to enable instant `rename()` — app should detect drive mismatch and warn
- Flat per-mod layout inside staging: staging/[modname]/ contains that mod's disabled files

### Permission Strategy
- UAC elevation prompted once per session — not per operation
- Elevated helper process spawned at app start when games are configured in protected paths (Program Files)
- Helper stays running for the session, handles all file moves in protected directories
- If no games are in protected paths, no elevation needed — helper not spawned

### App Shell / Layout
- Single game view — select a game first, then full-screen mod view (confirmed during project questioning)
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

</decisions>

<specifics>
## Specific Ideas

- Staging folder structure: ~/.modtoggler/games/[game-name]/staging/[mod-name]/ — keeps disabled mods organized per game with clear mod boundaries
- UAC should feel seamless — one prompt at launch, then forget about it
- Game setup should be quick — name, pick folder, done. Don't over-complicate the first interaction.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None yet — Phase 1 establishes the patterns all subsequent phases follow

### Integration Points
- Tauri v2 IPC layer: all file operations happen in Rust backend, React frontend calls via invoke()
- SQLite database: game config, staging paths, transaction journal all stored here
- tauri-specta: typed IPC bindings established in this phase carry through entire project

</code_context>

<deferred>
## Deferred Ideas

- Auto-detect Steam library games — future enhancement to game setup
- Custom game icons/thumbnails — nice to have, not v1 priority

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-04*
