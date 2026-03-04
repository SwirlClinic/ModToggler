# Project Research Summary

**Project:** ModToggler
**Domain:** Desktop game mod manager — file-toggle pattern, Unreal Engine .pak/.ucas/.utoc, Windows
**Researched:** 2026-03-04
**Confidence:** HIGH (stack and architecture), MEDIUM-HIGH (features), HIGH (pitfalls)

## Executive Summary

ModToggler is a local-only desktop mod manager built on Tauri v2 + React + TypeScript. The expert approach for this domain is a file-move toggle pattern: disabled mods live in an app-managed staging folder (`~/.modtoggler/disabled/[game]/`) and are moved atomically into the game's live mod directory when enabled. All file operations must run in async Rust commands — never synchronously, never from the frontend — with SQLite as the authoritative source of truth and Zustand as a display cache only. The architectural throughline across all four research areas is that the Rust backend owns the filesystem and database, and the React frontend renders what Rust tells it is true after each command completes.

The recommended stack is well-established: Tauri v2.10.3, React 19, TypeScript 5, Vite 7, Zustand 5 for UI state, TanStack Query 5 for async data over Tauri IPC, Tailwind CSS 4 + shadcn/ui for components, and SQLite via tauri-plugin-sql for persistence. The tauri-specta library generates TypeScript bindings from Rust command signatures, eliminating the most common source of IPC type drift. The `zip` Rust crate handles archive extraction in custom async Tauri commands — the tauri-plugin-extract alternative has insufficient maturity for production use.

The top risks are operational: half-toggled mods from crashes leave the game broken (transaction journaling is non-negotiable from Phase 1), UAC elevation for games in Program Files requires a separate helper process (running the whole Tauri app elevated breaks WebView2 on Windows 11), and cross-drive staging paths silently turn fast renames into slow copy-then-delete operations. ZipSlip path traversal during extraction is a security requirement, not optional hardening. These are all preventable if addressed in the right phase — they become very costly to retrofit.

## Key Findings

### Recommended Stack

The stack is tightly integrated and each choice reinforces the others. Tauri v2 provides the app shell with a Rust backend that handles all filesystem operations safely; tauri-specta provides compile-time type safety across the IPC boundary so Rust struct changes break the TypeScript build immediately instead of at runtime. TanStack Query wraps all `invoke()` calls with loading, error, and cache semantics — this eliminates an entire category of manual state management bugs that arise from raw `useState` + `invoke` patterns.

For the UI layer, shadcn/ui components are owned code (copied into the codebase, not an external dependency), which aligns with the project's local-only philosophy. Tailwind v4 is the current stable release with a significantly improved build pipeline. The only RC-status dependency is tauri-specta 2.0.0-rc.21, which is widely deployed in production Tauri projects despite the label.

**Core technologies:**
- Tauri 2.10.3: App shell + Rust backend — smaller binary than Electron, Rust handles large file moves safely
- React 19 + TypeScript 5: Frontend UI — required for tauri-specta's type-safe IPC binding generation
- Vite 7: Build tool — default for Tauri's React template, near-instant HMR
- Zustand 5: UI state cache — minimal boilerplate, display cache only (Rust DB is source of truth)
- TanStack Query 5: Async data layer over invoke() — loading/error/stale states without manual management
- TanStack Router 1.x: Client routing — type-safe route params, designed for Vite SPAs
- Tailwind CSS 4 + shadcn/ui: Styling + components — accessible primitives, toggle/switch components match mod manager UI directly
- tauri-plugin-sql 2.x + SQLite: Persistence — single-file DB, relational data for mods/games/profiles/conflicts
- tauri-specta 2.0.0-rc.21: IPC type generation — eliminates hand-maintained `invoke<T>` type casts
- zip crate 2.x (Rust): Archive extraction — 107M downloads, battle-tested; do not use tauri-plugin-extract

### Expected Features

Research from competitive analysis of Vortex, MO2, Fluffy Mod Manager, and Tekken 8-specific tools (Mod Manager Plus, Another Mod Manager V2) confirms a clear MVP boundary. No existing free tool handles PAK triple auto-grouping or sub-mod option toggling well — these are genuine differentiators.

**Must have (table stakes — v1):**
- Toggle mod on/off with one click — core product promise; failure here is catastrophic
- Import mod from .zip archive — standard delivery format; all operations depend on this
- Game configuration with configurable mod path — prerequisite for everything else
- Persistent mod state across restarts — users expect this; SQLite satisfies it
- Conflict detection (file stem overlap warning) — required trust signal; two mods claiming same UE4 stem
- UE4 PAK triple auto-grouping (.pak/.ucas/.utoc by base stem) — distinguishes from competitors at import time
- Per-game mod lists with enabled/disabled status — multi-game management is expected

**Should have (competitive — v1.x, after core validation):**
- Named profiles per game — power users need "online-safe" vs "full mod" presets
- Mod options / sub-folder toggling — Tekken 8 costume color variants; no existing free tool handles this
- Conflict visualization (which mods conflict, not just "conflict exists") — natural follow-on to detection
- Per-mod notes — low cost, high quality-of-life

**Defer (v2+):**
- Mishmash / loose-file game support — high complexity, niche secondary use case
- Export/import profiles as JSON — useful for sharing, not needed until profiles ship
- Multi-platform (macOS/Linux) — Tauri enables it, no validated demand outside Windows gaming

**Anti-features to avoid entirely:**
- Mod downloading from Nexus/TekkenMods — API keys, rate limits, legal review, scope explosion
- Auto-update mods — silent breakage risk, out of scope for a file manager
- Virtual file system (VFS) — kernel-level complexity, defeats the simplicity goal
- Load order / priority numbers — meaningless for PAK-based games where last writer wins

### Architecture Approach

The architecture follows a strict layering: React frontend renders from Zustand (display cache), Zustand is populated by reading Rust state after each IPC command, all Rust commands are thin adapters over service functions, services do all file I/O and DB access, and SQLite is the only authoritative source of state. The key invariant is that Zustand is never treated as truth — after any mutation command, the frontend calls a refresh that re-reads canonical state from Rust. The file manifest (captured at import into SQLite's `file_entries` table) is what makes conflict detection, toggle operations, and profile switching all fast index queries rather than live filesystem scans.

**Major components:**
1. React Frontend (Webview) — Game View, Mod List, Profile Panel, Settings Panel; renders from Zustand
2. Zustand Store — display cache for games[], activeMods[], profiles[]; invalidated and repopulated from Rust after mutations
3. IPC Layer (tauri-specta typed invoke()) — JSON over ipc://; all calls go through lib/tauri.ts wrappers
4. Rust Command Handlers (thin) — accept raw inputs + State refs, delegate to services; no file I/O directly
5. File Ops Service — atomic file moves between staging and game directory; rollback on failure
6. Archive Extractor — zip extraction to staging, ZipSlip validation, file manifest recording
7. Conflict Detector — pure DB query: cross-join file_entries for enabled mods; no filesystem scans
8. SQLite (db.sqlite) — source of truth: games, mods, file_entries, profiles
9. Staging Area (~/.modtoggler/disabled/[game]/) — disabled mod files at rest; plain filesystem

### Critical Pitfalls

1. **No crash recovery leaves mods half-toggled** — Write a transaction journal before every toggle: record op, files, and status. On startup, scan for incomplete journals and offer complete-or-rollback. Cannot be retrofitted; must be designed in Phase 1 file ops.

2. **UAC / elevated permissions not planned for** — Games in `C:\Program Files (x86)\Steam\` require admin access. Running the whole Tauri app elevated breaks WebView2 on Windows 11. Use a separate small Rust CLI helper for elevated file ops. Add a write-access probe when the user configures any game path.

3. **Cross-drive staging turns instant rename into slow copy-then-delete** — `std::fs::rename()` is atomic on same volume; cross-volume is copy+delete and can take 30+ seconds for large .pak files. Detect volume mismatch at path configuration. Offer a game-adjacent staging option. Always show progress for toggle operations — never assume a move is instant.

4. **State desync: DB says enabled, files say otherwise** — Antivirus quarantine, manual file moves, or incomplete past operations can corrupt DB/filesystem agreement. Run a startup integrity scan: verify each mod marked enabled has files in the game directory, each disabled mod has files in staging. Surface discrepancies with recovery options.

5. **ZipSlip path traversal in archive extraction** — A crafted .zip with `../../` entries can write files anywhere on the filesystem. Always canonicalize each extraction target path and assert it starts with the destination staging directory. Reject any entry with `..` components. This is a security requirement, not optional.

## Implications for Roadmap

Based on feature dependencies, architecture layering, and pitfall phase mappings, a 4-phase structure is recommended:

### Phase 1: Foundation — Game Config + File Operations Infrastructure
**Rationale:** Every feature in the product depends on two things: knowing the game's mod path, and having a reliable atomic file-move mechanism. These must be built first and must be correct. The transaction journal (crash recovery) and startup integrity check belong here — they cannot be retrofitted after toggle is built. The SQLite schema also locks in here.
**Delivers:** Game configuration UI (add/remove games, set mod path with write-access probe), staging directory setup, async file move service with transaction journal, startup integrity scan, SQLite schema with games + mods + file_entries tables.
**Addresses features:** Add/remove games, configurable mod path, disabled mods in staging folder.
**Avoids:** Crash recovery pitfall, cross-drive performance pitfall, UAC planning (helper process architecture decision here even if not fully implemented), state desync pitfall.
**Research flag:** Needs research-phase — UAC helper process architecture on Windows is non-trivial; Tauri v2 capabilities scoping for protected paths needs validation against a real Program Files path.

### Phase 2: Core Mod Lifecycle — Import, Toggle, Conflict Detection
**Rationale:** With file operations proven and the schema in place, the core mod management loop can be built. Import must come before toggle (you need mods to toggle). Conflict detection logic is a pure DB query on the file_entries table established in Phase 1, so it is nearly free to add here. UE4 PAK triple auto-grouping is a detection pass at import time — implement it here, not as a separate phase.
**Delivers:** Import from .zip with ZipSlip validation, PAK triple auto-detection at import, file manifest recording into file_entries, mod list per game with enabled/disabled display, toggle on/off with optimistic UI + rollback, conflict detection warnings (stem overlap).
**Uses:** zip crate, tauri-specta typed commands (toggle_mod, import_mod, list_mods), TanStack Query mutations for toggle, Zustand modStore/gameStore.
**Implements:** Archive Extractor, File Ops Service (toggle path), Conflict Detector, mod_detector service.
**Avoids:** Import overwrite pitfall (duplicate name check at import), ZipSlip, synchronous file moves.
**Research flag:** Standard patterns — well-documented Tauri async command + Zustand refresh loop. No research-phase needed unless UE4 pak triple detection edge cases emerge during implementation.

### Phase 3: Reliability and UX Polish
**Rationale:** Phase 2 delivers a working product, but research shows that production-quality mod management requires surfacing errors clearly, progress feedback for large files, and conflict visualization. These turn a "works in testing" app into one users trust with their game installations.
**Delivers:** Per-file progress events during toggle (Tauri events from Rust to frontend), progress bar UI (shadcn Progress component), conflict badges on mod list (which mods conflict, not just "conflict exists"), improved error messages for PermissionDenied distinguishing UAC from other errors, per-mod notes field.
**Implements:** Tauri event emission from file_ops service, ConflictBadge component, conflict visualization query.
**Avoids:** No progress feedback UX pitfall, silent failure on permission denied.
**Research flag:** Standard patterns — Tauri event system is well-documented. Skip research-phase.

### Phase 4: Power User Features — Profiles + Mod Options
**Rationale:** Named profiles and sub-mod option toggling are explicitly validated differentiators from competitive analysis. They both depend on stable toggle (Phase 2) and are high-complexity enough to deserve their own phase. Profiles require many sequential toggles — if individual toggle is unreliable, profile switching amplifies failures. Sub-mod toggling requires the PAK triple grouping logic from Phase 2 to operate recursively at the sub-folder level.
**Delivers:** Named profiles per game (save/load sets of enabled mod IDs), profile switching (sequential toggle of delta between current state and target profile), mod options / sub-folder toggling (recursive PAK grouping within sub-folders, independent enable/disable per option).
**Addresses features:** Named profiles, mod options / sub-mod toggling (FEATURES.md P2 items).
**Avoids:** Profile load failure with missing mods must show clear error rather than silent partial application.
**Research flag:** Mod options sub-folder structure needs research-phase during planning — the interaction between top-level PAK triple grouping and sub-folder-scoped grouping is complex and not well-documented in existing tools.

### Phase Ordering Rationale

- **File infrastructure before UI:** The feature dependency graph is explicit — game config is required by everything, import is required by all mod operations, and toggle is required by profiles. Building UI before the backend is solid creates rework.
- **Pitfalls dictate phase 1 scope:** Three critical pitfalls (crash recovery, state desync, cross-drive detection) must be addressed before toggle is considered "done." Research makes explicit that these cannot be retrofitted.
- **Journal and integrity scan in Phase 1, not Phase 3:** These are often treated as polish features but are foundational reliability requirements. Placing them in Phase 1 avoids user data loss during early testing.
- **PAK grouping in Phase 2 (not Phase 4):** The mod_detector logic is needed at import time and for conflict detection. Building it late would require retrofitting the data model.
- **Security requirements (ZipSlip) in Phase 2:** Non-negotiable at import implementation time. Not deferrable.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1:** UAC helper process architecture — Tauri v2 capabilities vs. OS ACLs interaction on Windows, correct approach for a separate elevated binary, testing methodology against Program Files paths with a standard user account.
- **Phase 4:** Mod options sub-folder structure — recursive PAK triple grouping at sub-folder level, data model for parent mod + child options, UI for expandable option rows within a mod card.

Phases with standard patterns (can skip research-phase):
- **Phase 2:** Import + toggle + conflict detection — all follow the established Tauri async command + service + DB + Zustand refresh pattern. Tauri docs and tauri-specta cover the IPC typing fully.
- **Phase 3:** Progress events + conflict visualization — Tauri's event system (emit/listen) is well-documented. Conflict badge is a derived query over existing data.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core technologies verified against official Tauri v2 docs, npm, crates.io. Only exception: tauri-specta is RC status (2.0.0-rc.21) but widely production-deployed per community reports |
| Features | MEDIUM-HIGH | Table stakes confirmed by competitive analysis of 4 tools. Differentiator demand inferred from community pain-point articles rather than direct user research — validate with early users |
| Architecture | HIGH | Patterns sourced from official Tauri docs + Vortex open-source reference implementation. Command-service separation, optimistic UI with rollback, and file manifest as source of truth are established patterns with working examples |
| Pitfalls | HIGH | Critical pitfalls backed by official sources: Tauri GitHub issues (WebView2 elevation bug), Microsoft Learn (MoveFile atomicity, Windows permissions), Snyk security advisories (ZipSlip CVE-2025-29787) |

**Overall confidence:** HIGH

### Gaps to Address

- **UAC helper process implementation details:** Research confirms the need for a separate elevated binary but the exact Tauri v2 integration pattern (process spawning, IPC between helper and main app) needs validation against a real Program Files game path. Address in Phase 1 planning research.
- **tauri-specta RC stability:** Version 2.0.0-rc.21 is widely used but RC status means APIs could change before stable release. Pin to an exact version in Cargo.toml and monitor the tauri-specta GitHub for breaking changes between phases.
- **Sub-mod option data model:** The interaction between top-level mod records, PAK triple grouping, and sub-folder option toggling is the most architecturally complex part of the system. The schema implications (how to represent parent mod + child options in SQLite) are not fully resolved in research. Address in Phase 4 planning research.
- **Cross-drive staging path UX:** Research confirms the problem and recommends a game-adjacent staging option, but the exact onboarding flow (when to prompt, how to migrate an existing staging path) is not designed. Address in Phase 1 planning.

## Sources

### Primary (HIGH confidence)
- https://v2.tauri.app/ — Tauri v2 official docs: architecture, IPC, plugins, capabilities, version 2.10.3
- https://v2.tauri.app/plugin/file-system/ — File system plugin: scope config, rename/copy APIs, path traversal prevention
- https://v2.tauri.app/plugin/sql/ — SQL plugin: SQLite setup, migration support, Rust 1.77.2+ requirement
- https://github.com/tauri-apps/tauri/issues/13926 — WebView2 elevation bug: running whole Tauri app as admin breaks WebView2 on Windows 11 Admin Protection
- https://github.com/tauri-apps/tauri/discussions/4201 — UAC elevation approach: separate helper process pattern
- https://security.snyk.io/vuln/SNYK-RUST-ZIP-9460813 — CVE-2025-29787: directory traversal in Rust zip crate (ZipSlip)
- https://security.snyk.io/research/zip-slip-vulnerability — ZipSlip vulnerability: extraction path validation requirement
- https://learn.microsoft.com/en-us/archive/msdn-technet-forums/449bb49d-8acc-48dc-a46f-0760ceddbfc3 — MoveFile atomicity: same-volume vs cross-volume behavior on Windows
- https://learn.microsoft.com/en-us/troubleshoot/windows-client/windows-security/permissions-on-copying-moving-files — Windows file permissions on move operations
- https://github.com/pmndrs/zustand — Zustand 5.0.6: useSyncExternalStore, React 18+ compatibility
- https://www.npmjs.com/package/@tanstack/react-query — TanStack Query 5.90.21
- https://tailwindcss.com/blog/tailwindcss-v4 — Tailwind CSS v4.0 released January 2025
- https://ui.shadcn.com/docs/tailwind-v4 — shadcn/ui React 19 + Tailwind v4 support

### Secondary (MEDIUM confidence)
- https://github.com/specta-rs/tauri-specta — tauri-specta 2.0.0-rc.21: Tauri v2 requirement, TypeScript binding generation
- https://tanstack.com/router/latest — TanStack Router v1.x: Tauri desktop app pattern confirmed by community
- https://github.com/dannysmith/tauri-template — Production Tauri v2 + React 19 + shadcn + Zustand + TanStack Query template: cross-validates recommended stack
- https://www.nexusmods.com/tekken8/mods/231 — Mod Manager Plus: Tekken 8 feature comparison
- https://tekkenmods.com/mod/4086/another-mod-manager-v2-now-with-presets — Another Mod Manager V2: feature comparison
- https://www.patreon.com/posts/fluffy-mod-v3-151635881 — Fluffy Mod Manager v3.070: mixed PAK/loose file support, competitor features
- https://github.com/ModOrganizer2/modorganizer/wiki/Mod-Managers-Comparison — MO2 mod manager comparison wiki
- https://buckminsterfullerene02.github.io/dev-guide/Basis/DealingWithPaks.html — UE4 pak/ucas/utoc structure: community guide
- https://wiki.nexusmods.com/index.php/Managing_File_Conflicts — Managing file conflicts: Nexus Mods wiki
- https://dev.to/subtixx/why-is-game-mod-management-still-so-cumbersome-in-2025-5c0 — Community analysis of mod manager pain points

### Tertiary (LOW confidence)
- https://docs.rs/crate/zip/latest — zip crate version sourcing had discrepancies across search results; 8.1.0 referenced but verify current version at implementation time

---
*Research completed: 2026-03-04*
*Ready for roadmap: yes*
