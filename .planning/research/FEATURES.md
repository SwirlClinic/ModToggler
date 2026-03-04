# Feature Research

**Domain:** Game mod manager desktop application (PAK/file-based, multi-game)
**Researched:** 2026-03-04
**Confidence:** HIGH (competitive analysis from Vortex, MO2, Fluffy Mod Manager, Tekken 8-specific tools; MEDIUM on differentiators — user demand inferred from pain-point articles)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Toggle mod on/off with one click | Core reason to use a manager over manual file moves | LOW | Must be instant and reliable; failure here is catastrophic trust loss |
| Import mod from .zip archive | Standard delivery format for all Tekken 8 mods on Nexus/TekkenMods | MEDIUM | Must extract, introspect file structure, and register mod metadata on import |
| Persistent mod state across app restarts | Users expect their enabled/disabled state to be remembered | LOW | Requires a local config/DB; JSON or SQLite both work |
| Per-game mod lists | Users manage mods for multiple games; mixing them is confusing | LOW | Game selection screen drives which mod list is shown |
| Add/remove games with configurable mod path | Each game has a different Paks or Mods directory | LOW | Simple settings form with path picker |
| Conflict detection — warn on overlapping files | Two mods touching the same .pak stem means one silently overwrites the other | MEDIUM | File-stem comparison for UE4 PAK triples (.pak/.ucas/.utoc); straightforward hash/name scan |
| Clean removal (delete mod + its files from staging) | Users need to get rid of mods they no longer want | LOW | Must remove from staging area only; never touch game directory for disabled mods |
| Show which files belong to which mod | Users need confidence the app knows what it owns | LOW | Display on mod detail/expand view; data captured at import |
| Disabled mods persisted in app-managed folder | Files must not disappear when toggled off | LOW | App moves files to ~/.modtoggler/disabled/[game]/ on disable |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Named profiles per game (save/load mod configurations) | Power users run "online-safe" vs "full mod" setups; switching is otherwise tedious | MEDIUM | Store profile as a named set of enabled mod IDs; switching moves files accordingly |
| Mod options / sub-mod toggling | Tekken 8 costume mods often ship with color variants in sub-folders; no existing free tool handles this well | HIGH | Sub-folder structure within a mod; each sub-folder is independently toggleable; requires recursive introspection at import |
| Unreal Engine PAK triple auto-grouping | Users hate manually associating .pak/.ucas/.utoc files; current tools are inconsistent | MEDIUM | Detect files sharing a base stem (e.g., `ExampleMod.pak`, `ExampleMod.ucas`, `ExampleMod.utoc`) and present them as one logical mod |
| Mishmash / loose-file game support | Games that scatter mod files across the root dir have no dedicated manager | HIGH | Requires manual file tagging UI at import time — user maps files to arbitrary destination paths; app tracks the mapping |
| Conflict visualization showing which specific mods conflict | Most tools say "conflict exists" but don't clearly show which two mods are fighting over what | MEDIUM | Render a conflict matrix or inline badge on each mod card showing the names of conflicting mods |
| Per-mod notes field | Users want to annotate mods ("breaks online", "wait for update") | LOW | Simple text field stored in mod metadata |
| Lightweight, no account required, no Nexus integration | Community frustration: Vortex forces NexusMods login and ecosystem lock-in | LOW | No auth, no download integration — pure local file management |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Mod downloading from Nexus/TekkenMods | "One app to rule them all" appeal | Requires API keys, rate limits, login flows, legal review, and constant maintenance as sites change; massively expands scope beyond file management | Keep scope at import from local .zip — user downloads manually, then imports |
| Auto-update mods | Convenience when mods ship fixes | Mod updates can break working setups silently; requires tracking mod IDs against external registries; outside scope of a file manager | Show "last imported" date on mod card so user knows when they last updated manually |
| Game launching / injection | Users want one-click "launch with mods" | Game launch logic varies wildly per game/platform; anti-cheat interactions; permission complexity; completely separate problem domain | Out of scope per PROJECT.md; link to existing launchers via shell open if needed |
| Virtual file system (VFS) like MO2 | No file moves = no risk of file loss | VFS requires kernel-level or injected hooking (USVFS, hardlinks, or fuse); enormous complexity on Windows with UAC and Program Files paths; hard to debug when it breaks | File-move approach is simpler, auditable, and adequate for the target use case |
| Load order management / priority numbers | Power users want fine-grained control | Meaningful only when two mods touch the same file AND both should partially apply — not possible with PAK files (last writer wins); adds UI complexity for no benefit in PAK-based games | Surface conflict warnings instead; let user choose which conflicting mod to disable |
| Mod merging | "Apply both mods at once" | Requires understanding internal file formats (PAK internals, mesh files, textures); deeply game-specific; easily creates corrupted output | Out of scope; direct users to game-specific merge tools |
| Cloud sync of mod configurations | Share setups between machines | Requires auth, cloud storage integration, sync conflict resolution; high complexity for a niche use case | Export/import profile as a JSON file for manual sharing |

---

## Feature Dependencies

```
[Game Configuration (add game + path)]
    └──required by──> [Per-Game Mod List]
                          └──required by──> [Toggle On/Off]
                          └──required by──> [Conflict Detection]
                          └──required by──> [Named Profiles]

[Import from .zip]
    └──required by──> [All mod operations]
    └──enables──> [UE4 PAK Triple Auto-Grouping]
    └──enables──> [Mod Options / Sub-Mod Toggling]

[File Move (toggle mechanism)]
    └──required by──> [Disabled Mods in Staging Folder]
    └──required by──> [Clean Removal]

[Conflict Detection]
    └──enhances──> [Conflict Visualization]

[Named Profiles]
    └──requires──> [Toggle On/Off]  (profiles are just saved sets of toggle states)

[Mod Options / Sub-Mod Toggling]
    └──requires──> [Import from .zip]  (sub-folder structure discovered at import)
    └──conflicts with──> [UE4 PAK Triple Auto-Grouping at top level]
                         (sub-mods contain their own PAK triples; grouping must operate
                          at sub-mod level, not root mod level)
```

### Dependency Notes

- **Game Configuration required by everything:** Without knowing the game's mod path, the app cannot move files or detect conflicts. This is the first feature to build.
- **Import required by all mod operations:** The app's entire model is built on tracked imports. No import = no managed mods.
- **Toggle requires file-move infrastructure:** The Rust backend must handle large files, UAC-elevated paths in Program Files, and atomic moves (no partial state). Build this carefully before building UI on top.
- **Profiles require stable toggle:** Switching a profile is many toggles in sequence. If individual toggle is unreliable, profile switching amplifies the failure.
- **Sub-mod toggling complicates PAK grouping:** At the top mod level, PAK triples group by base stem. Inside a sub-mod folder, the same grouping applies but is scoped to that sub-folder. The grouping logic must be recursive-aware.

---

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the core use case (Tekken 8 on Windows).

- [ ] Add game with configured Paks path
- [ ] Import mod from .zip — extract, detect PAK triples, register file manifest
- [ ] Toggle mod on/off (move files between game directory and ~/.modtoggler/disabled/)
- [ ] Persist mod state across restarts (local config)
- [ ] Conflict detection — warn when two enabled mods share a file stem
- [ ] Show mod list per game with enabled/disabled status

### Add After Validation (v1.x)

Features to add once core file-move and toggle are proven stable.

- [ ] Named profiles — trigger: users report manually re-enabling the same sets repeatedly
- [ ] Mod options / sub-folder toggling — trigger: user feedback on costume color variant use case
- [ ] Per-mod notes — trigger: cheap to add, high quality-of-life signal once core is solid
- [ ] Conflict visualization (which mods conflict) — trigger: conflict warnings ship in v1; visualization is the natural follow-up

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Mishmash / loose-file game support — high complexity, niche secondary use case; defer until primary use case is mature
- [ ] Export/import profiles as JSON — useful for sharing, but not needed until profiles ship
- [ ] Multi-platform (macOS/Linux) — Tauri enables it, but no validated demand outside Windows gaming

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Toggle mod on/off | HIGH | MEDIUM | P1 |
| Import from .zip + file manifest | HIGH | MEDIUM | P1 |
| Game configuration (add game + path) | HIGH | LOW | P1 |
| Persist state across restarts | HIGH | LOW | P1 |
| Conflict detection (file stem overlap) | HIGH | LOW | P1 |
| UE4 PAK triple auto-grouping | HIGH | MEDIUM | P1 |
| Named profiles | MEDIUM | MEDIUM | P2 |
| Mod options / sub-mod toggling | MEDIUM | HIGH | P2 |
| Conflict visualization (which mods) | MEDIUM | LOW | P2 |
| Per-mod notes | LOW | LOW | P2 |
| Mishmash / loose-file game support | MEDIUM | HIGH | P3 |
| Export/import profiles as JSON | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | Fluffy Mod Manager | Mod Manager Plus (T8) | Another Mod Manager v2 (T8) | Our Approach |
|---------|--------------------|-----------------------|------------------------------|--------------|
| Toggle on/off | Yes | Yes | Yes | Yes — file move to ~/.modtoggler/ |
| Import from .zip | Yes | Unknown | Unknown | Yes — first-class import flow |
| PAK triple (.pak/.ucas/.utoc) grouping | Partial (deletes UCAS/UTOC on remove; inconsistent) | Unknown | Unknown | Auto-group by base stem at import |
| Sub-mod / mod options | No | No | No | Yes (P2) — sub-folder toggling |
| Conflict detection | No | Yes (same item slot warning) | Yes (same item slot warning) | Yes — file stem overlap detection |
| Named profiles / presets | Yes (mod presets) | Yes | Yes (V2) | Yes (P2) |
| Multi-game support | Yes (Capcom-focused) | No (T8 only) | No (T8 only) | Yes — per-game configuration |
| Mishmash / loose files | Partial (v3.070 WIP) | No | No | Planned (P3) |
| No account required | Yes | Yes | Yes | Yes — local only |
| Per-mod notes | No | No | No | Yes (P2) |

---

## Sources

- [Vortex Mod Manager — Nexus Mods](https://www.nexusmods.com/about/vortex) — MEDIUM confidence (marketing page)
- [Mod Managers Comparison — ModOrganizer2 GitHub Wiki](https://github.com/ModOrganizer2/modorganizer/wiki/Mod-Managers-Comparison) — HIGH confidence (maintained by MO2 team)
- [Mod Manager Plus at Tekken 8 Nexus](https://www.nexusmods.com/tekken8/mods/231) — MEDIUM confidence (mod page, features described by author)
- [Another Mod Manager V2 — TekkenMods](https://tekkenmods.com/mod/4086/another-mod-manager-v2-now-with-presets) — MEDIUM confidence (mod page)
- [Fluffy Mod Manager v3.070 — Mixed PAK/Loose File Support](https://www.patreon.com/posts/fluffy-mod-v3-151635881) — MEDIUM confidence (developer release notes)
- [Why Is Game Mod Management Still So Cumbersome in 2025? — DEV Community](https://dev.to/subtixx/why-is-game-mod-management-still-so-cumbersome-in-2025-5c0) — MEDIUM confidence (community analysis, not official)

---

*Feature research for: ModToggler — game mod manager desktop app*
*Researched: 2026-03-04*
