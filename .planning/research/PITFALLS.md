# Pitfalls Research

**Domain:** Desktop mod manager — Windows file system, Unreal Engine .pak/.ucas/.utoc, Tauri v2
**Researched:** 2026-03-04
**Confidence:** HIGH (Windows/UE file behavior), MEDIUM (Tauri-specific elevation), HIGH (state management)

---

## Critical Pitfalls

### Pitfall 1: No Crash Recovery — Half-Toggled Mods Leave Game Broken

**What goes wrong:**
A toggle operation (enable or disable a mod) is interrupted mid-flight — power loss, app crash, process kill — leaving some files in the game directory and some still in staging. The game may fail to load, crash on startup, or exhibit unpredictable behavior. The app's database says the mod is enabled or disabled but the file system disagrees.

**Why it happens:**
File-by-file moves are not atomic. A .pak/.ucas/.utoc group requires moving three files. If the process dies after moving .pak but before .ucas, the game directory is in an inconsistent state. Most early implementations skip journaling because "it's just moving files."

**How to avoid:**
Design a transaction log from Phase 1. Before any toggle operation, write a journal entry: `{ op: "enable", mod_id: "...", files: [...], status: "in_progress" }`. After each file move, update the journal. On completion, mark `status: "done"`. On app startup, always check for incomplete journal entries and offer recovery (complete the operation or roll it back).

For rollback: keep a record of where each file came from. If enabling fails halfway, move already-transferred files back to staging.

**Warning signs:**
- Toggle operations have no pre/post consistency check
- App opens directly to mod list with no startup integrity scan
- No "pending operations" concept in the data model
- File operations happen in a loop with no intermediate state writes

**Phase to address:**
Core file operations phase (Phase 1 or 2 — wherever toggle logic is implemented). Cannot be retrofitted easily.

---

### Pitfall 2: UAC / Elevated Permissions Not Planned For

**What goes wrong:**
Steam installs games to `C:\Program Files (x86)\...` which is ACL-protected. Moving files into or out of this directory requires administrator privileges. An app without an elevation strategy will silently fail or throw cryptic OS errors when users have games in the default Steam location.

**Why it happens:**
Developers test with games installed outside Program Files (e.g. `D:\Games\`) where standard user permissions work. The app appears to work. UAC issues only surface when users have Steam in the default location.

**Additional Tauri-specific trap:**
Running the entire Tauri app elevated (via embedded manifest) causes a WebView2 bug on Windows 11 with Administrator Protection enabled — WebView2 fails to start because it cannot access the elevated user's AppData directory. Running the whole app as admin is therefore unreliable.

**How to avoid:**
- Design for a helper process model from the start: a small Rust CLI binary that requests elevation via `runas` and handles only file move operations. The main Tauri app stays non-elevated and communicates with the helper when needed.
- Alternatively, detect at startup whether the game path is inside a protected directory and warn the user before any operation fails.
- Include a permission check in the game path configuration step — test write access with a probe file before accepting the path.

**Warning signs:**
- No write-access check when user configures a game directory
- File operations are all done inline in the main app process
- No error handling for `ERROR_ACCESS_DENIED` (OS error 5) that distinguishes it from other IO errors
- Testing only on non-Program-Files paths

**Phase to address:**
Game configuration / path setup phase. The helper process architecture must be decided before file operation code is written.

---

### Pitfall 3: Cross-Drive Staging Turns Move Into Slow Copy-Then-Delete

**What goes wrong:**
A file system `rename()` / `MoveFile()` is instant when source and destination are on the same volume — the OS just updates directory entries. When source and destination are on different volumes, the OS must physically copy bytes then delete the original. A 500 MB .ucas file that should toggle in milliseconds now takes 30+ seconds. Users think the app is frozen.

**Why it happens:**
The staging folder defaults to `~/.modtoggler/` (usually on C:). If the game is installed on a different drive (D:, E:, etc.), every toggle becomes a multi-hundred-megabyte copy. Most developers test with game and home directory on the same drive.

**How to avoid:**
- Detect at startup whether the game path and staging path are on the same volume using volume serial numbers (Windows `GetVolumeInformation`).
- If cross-drive: warn the user and offer to set a game-adjacent staging location (e.g. `[game_dir]/../.modtoggler_staging/`).
- Show a progress indicator with file size and estimated time for all toggle operations — never assume a move is instant.
- Consider offering a symlink/junction-based approach as an alternative for cross-drive setups (though this adds complexity).

**Warning signs:**
- Staging path is hardcoded to `~/.modtoggler/` with no override
- No progress feedback during toggle — operations assumed to be instant
- No volume comparison logic anywhere in the codebase
- User reports of the app "hanging" on toggle

**Phase to address:**
Core file operations phase, and also the game/path configuration phase where the staging location is established.

---

### Pitfall 4: State Desync — Database Says Enabled, Files Say Otherwise

**What goes wrong:**
The app's internal state (SQLite / JSON) tracks which mods are enabled. A user manually moves files, the game crashes and reverts changes, an antivirus quarantines a file, or a previous incomplete operation was never recovered. Now the database is wrong and the app presents false information. Toggling an "enabled" mod fails because the files aren't where the app expects them.

**Why it happens:**
State is written to the database and file operations happen separately, with no verification loop. The two sources of truth (database + file system) drift apart whenever anything outside the app touches the files.

**How to avoid:**
- On every app startup (and optionally on each toggle), verify that the file system state matches the database. For each mod marked "enabled", check that all its files exist in the game directory. For each mod marked "disabled", check that all its files exist in the staging area.
- Report discrepancies to the user with actionable options: "Mod X files missing from game directory — re-enable from staging?" or "Mod X files found in game directory but marked disabled — mark as enabled?"
- Store file checksums or at minimum file sizes at import time to detect file replacement.

**Warning signs:**
- No startup integrity check
- Toggle errors are shown as generic IO errors without explaining the likely cause
- No "rescan" or "repair" function in the UI
- File existence is only checked at toggle time, not on load

**Phase to address:**
Core file operations phase, alongside or immediately after toggle implementation.

---

### Pitfall 5: Conflict Detection Ignores the .pak/.ucas/.utoc Triplet

**What goes wrong:**
Conflict detection compares filenames to detect when two mods claim the same file. For Unreal Engine mods, the meaningful unit is the base name (e.g. `ExampleMod` — covering `ExampleMod.pak`, `ExampleMod.ucas`, `ExampleMod.utoc`). A naive implementation detects conflicts at the extension level and either misses real conflicts (different stems with different extensions that don't share a stem) or triggers false positives.

More critically, Unreal Engine uses filename-based load ordering. Mods loaded later (alphabetically later filenames) override earlier mods for the same asset. The conflict is not just "two mods touch the same file" — it's "two mods modify the same in-engine asset path." Without pak inspection, the app cannot know this.

**Why it happens:**
Conflict detection seems simple — compare file lists. Developers implement filename comparison without understanding that UE asset conflicts happen at the pak-internal path level, not at the pak filename level.

**How to avoid:**
- Group files by base stem for display and conflict reporting purposes (the triplet `ExampleMod.*` is one logical unit).
- For Phase 1 conflict detection: warn when two mods have the same base stem in the same directory — this is a definite file-level conflict.
- Document clearly that true asset-level conflict detection (which requires reading pak internals) is out of scope — don't promise what you can't deliver without pak introspection tooling.
- Expose load ordering to the user: show mods sorted by how UE will load them (alphabetical within the Mods folder) and let users rename mods to control priority.

**Warning signs:**
- Conflict detection works on full filenames including extension, not stems
- No concept of "mod load order" in the data model
- Users report that "conflict detection says no conflict but mods are fighting"

**Phase to address:**
Conflict detection feature phase. The stem-grouping concept must be in the data model from the start (Phase 1).

---

### Pitfall 6: Import Overwrites Existing Mod Without Warning

**What goes wrong:**
A user imports a mod that has the same name as an existing mod. The app silently replaces the existing mod's tracked files, potentially orphaning files already deployed to the game directory. The old mod's files remain in the game directory but are no longer tracked — invisible ghost files.

**Why it happens:**
Import logic checks mod name/ID uniqueness but doesn't reconcile with already-deployed files. The "happy path" of importing a fresh mod is tested; the update/replace scenario is not.

**How to avoid:**
- On import, check if a mod with the same name (or same file stems) already exists.
- If it does: present options — update (replace and redeploy), create as new (allow duplicate names), or cancel.
- If updating a currently-enabled mod: disable the old version first (move files back to staging), delete or archive the old staging files, then import the new version.

**Warning signs:**
- Import logic has no duplicate-name check
- "Currently enabled" state is not checked before import processing
- No concept of mod versioning or update path

**Phase to address:**
Mod import phase.

---

### Pitfall 7: Zip Extraction Path Traversal (ZipSlip)

**What goes wrong:**
A maliciously crafted .zip file contains entries with paths like `../../Windows/System32/evil.dll`. If the extraction code does not validate that each extracted path remains within the intended destination, files can be written anywhere on the filesystem — including outside the staging area.

**Why it happens:**
Rust's `zip` crate (and similar) does not automatically prevent path traversal. The developer uses the entry filename directly to construct the output path without validation.

**How to avoid:**
- Before extracting each entry, canonicalize the target path and assert it starts with the destination directory prefix.
- Reject any entry with `..` components or absolute paths.
- Use `std::path::Path::starts_with()` after resolving the full path to verify containment.
- Consider using a crate that handles this automatically, or add a pre-extraction validation pass.

**Warning signs:**
- Extraction code does `dest_dir.join(entry.name())` without path sanitization
- No test for malicious zip entries in the test suite
- Zip entries are trusted as safe user content

**Phase to address:**
Mod import phase (zip extraction implementation).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Store mod state as JSON flat file | Simple, no migrations | Concurrent write corruption, no transactions, hard to query | Never for production — use SQLite from day one |
| Skip transaction journal for file moves | Faster to implement | Unrecoverable corruption on crash, support nightmare | Never — the complexity is low, the payoff is high |
| Hardcode staging to `~/.modtoggler/` | One less config option | Cross-drive performance cliff, unresolvable for some users | MVP only, with a clear upgrade path |
| Move files synchronously in main thread | Simple code | UI freezes on large mods, perceived as crash | Never — async from the start |
| Trust mod filenames from zip as-is | No sanitization code | ZipSlip path traversal vulnerability | Never |
| Skip startup integrity check | Faster startup | State desync becomes permanent and confusing | Never |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Windows file move API | Assuming `fs::rename()` works across drives | Detect same-volume vs. cross-volume; use copy+delete for cross-volume with progress |
| Windows UAC / Program Files | Testing only on non-protected directories | Probe write access at path config time; design helper process for elevation |
| Tauri v2 file system plugin | Assuming Tauri permissions cover OS-level access | OS ACLs are enforced independently of Tauri's permission system; handle `PermissionDenied` explicitly |
| Tauri v2 elevated app + WebView2 | Running whole app as admin via manifest | WebView2 fails under Administrator Protection; use a separate elevated helper binary instead |
| Rust `zip` crate | Using entry names directly as output paths | Always validate output path is within destination directory (ZipSlip prevention) |
| Windows Defender / AV | Ignoring false positive risk for moved .pak files | Document in app help; consider adding a note to configure game folder as AV exclusion |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Synchronous file moves in UI thread | App appears frozen during toggle | Run all file I/O in async Rust task, send progress events to frontend | Every large mod (100+ MB) |
| Cross-drive staging path | Toggle of a large mod takes 30+ seconds | Detect volume mismatch at setup; offer game-adjacent staging | Any user with game on non-C drive |
| Reading all mod metadata into memory at startup | Slow startup with large mod libraries | Lazy-load mod file lists; only load what's needed for display | 50+ mods, especially with large file lists |
| Recalculating conflict state on every toggle | UI sluggish with many mods | Cache conflict graph; invalidate only affected mods on toggle | 20+ mods with overlapping file stems |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Extracting zip without path validation | Arbitrary file write anywhere on filesystem (ZipSlip) | Canonicalize each extraction path; assert containment in dest dir |
| Running full app elevated unnecessarily | Larger attack surface; WebView2 failure on Win11 Admin Protection | Elevate only the helper process that does privileged file ops |
| Trusting mod zip filenames for stored paths | Path confusion, potential overwrite of staging metadata | Sanitize and normalize all paths at import; store canonical paths only |
| Writing staging files as world-readable | Other apps can read potentially licensed mod content | Use default NTFS ACLs (user-private) for staging directory |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No progress feedback during toggle | User thinks app froze; force-quits mid-operation causing corruption | Show per-file progress bar with file name, size, and count (e.g., "Moving file 2 of 3 — ExampleMod.ucas") |
| Silent failure on permission denied | User sees nothing happen; mod stays in wrong state | Detect `PermissionDenied`, explain the cause ("Game is in Program Files — administrator access needed"), offer actionable next step |
| Conflict detection warning is too noisy | Users dismiss all warnings and ignore real conflicts | Only warn when files with same stem exist in same directory — true file-level conflicts. Don't warn for asset-level UE conflicts you can't resolve without pak introspection |
| No "undo last toggle" | User accidentally disables a mod, can't easily re-enable if they don't remember which one | Show recent toggle activity in sidebar; make the last toggle easily reversible |
| Mod names derived from zip filename | Zip names like `T8_ExampleMod_v2_FINAL_USE_THIS_ONE.zip` become the displayed mod name | Prompt user to set a clean display name at import; pre-fill from zip name but allow editing |
| No visual indicator of which mods conflict | User must toggle-test to discover conflicts | Show conflict badges on mod list; clicking a badge shows which other mods conflict and why |

---

## "Looks Done But Isn't" Checklist

- [ ] **Toggle operation:** Appears to work on same-drive setup — verify behavior when game is on a different drive from staging
- [ ] **Import:** Works for fresh imports — verify update flow when a mod with the same name already exists and is currently enabled
- [ ] **Conflict detection:** Detects obvious same-stem conflicts — verify it does NOT fire false positives for unrelated mods that happen to share a partial filename
- [ ] **Crash recovery:** Journal records are written — verify app correctly detects and offers recovery for interrupted operations on next startup
- [ ] **State integrity:** Database reflects current state — verify startup scan catches files that were manually moved or deleted outside the app
- [ ] **Permissions:** Works on dev machine — verify against a game installed in `C:\Program Files (x86)\` with standard user account
- [ ] **Zip extraction:** Extracts correctly — verify that a zip with `../../` path entries is rejected, not silently extracted outside staging
- [ ] **Mod options (sub-folders):** Top-level mod toggles work — verify option sub-folders are correctly included/excluded in file tracking and toggle operations
- [ ] **Profile save/load:** Profile saves correctly — verify that loading a profile when some mods in the profile no longer exist shows a clear error rather than silently partial-applying

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Half-toggled mod from crash | MEDIUM | Startup journal scan detects it; offer "complete operation" or "roll back" — implementation must be in place before first crash |
| State desync from external file change | LOW | Startup integrity scan detects it; user picks "accept file system state" or "restore from database" |
| Cross-drive performance complaint | LOW (config change) | Add staging path config option pointing to game-adjacent directory; document in onboarding |
| ZipSlip extraction into wrong path | HIGH | Cannot undo arbitrary file writes; prevent at import time — no recovery strategy makes this acceptable |
| Elevation failure for Program Files game | MEDIUM | Implement helper process; interim: document workaround (grant user write access to Mods subfolder only) |
| User overwrote existing mod unknowingly | HIGH | If old files were deployed and staging was overwritten, old version is gone; prevent with import duplicate detection |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| No crash recovery / half-toggled state | Core file operations (early) | Write a test that kills the process mid-toggle and verifies startup recovery |
| UAC / elevated permissions | Game path configuration phase | Test against a path in `C:\Program Files (x86)\` with a standard user account |
| Cross-drive slow moves | Core file operations + path config | Benchmark toggle time with game and staging on different drives; ensure progress UI shown |
| State desync | Core file operations (alongside toggle) | Test: manually delete a "enabled" mod's file from game dir; restart app; verify detection |
| Conflict detection ignores triplets | Data model (before conflict feature) | Unit test: two mods with same stem in same dir → conflict detected; different stems → no conflict |
| Import overwrites enabled mod | Mod import phase | Test: import mod with same name as currently-enabled mod; verify old files not orphaned |
| ZipSlip path traversal | Mod import phase (zip extraction) | Test: craft a zip with `../../evil.txt` entry; verify it is rejected, not extracted |
| Async / progress feedback | Core file operations | Test: toggle a 500 MB mod; verify UI remains responsive and shows progress |

---

## Sources

- [Why Is Game Mod Management Still So Cumbersome in 2025?](https://dev.to/subtixx/why-is-game-mod-management-still-so-cumbersome-in-2025-5c0) — MEDIUM confidence, community article
- [Tauri UAC elevation discussion](https://github.com/tauri-apps/tauri/discussions/4201) — HIGH confidence, official Tauri GitHub
- [Tauri WebView2 elevation bug](https://github.com/tauri-apps/tauri/issues/13926) — HIGH confidence, official Tauri GitHub issue
- [MoveFileEx atomicity on Windows](https://learn.microsoft.com/en-us/archive/msdn-technet-forums/449bb49d-8acc-48dc-a46f-0760ceddbfc3) — HIGH confidence, Microsoft Learn
- [Zip Slip Vulnerability](https://security.snyk.io/research/zip-slip-vulnerability) — HIGH confidence, Snyk security research
- [CVE-2025-29787: Directory Traversal in Rust zip crate](https://security.snyk.io/vuln/SNYK-RUST-ZIP-9460813) — HIGH confidence, Snyk advisory
- [Windows file permissions on copy/move](https://learn.microsoft.com/en-us/troubleshoot/windows-client/windows-security/permissions-on-copying-moving-files) — HIGH confidence, Microsoft Learn
- [Managing File Conflicts - Nexus Mods Wiki](https://wiki.nexusmods.com/index.php/Managing_File_Conflicts) — HIGH confidence, official Nexus docs
- [Unreal Engine pak/ucas/utoc structure](https://buckminsterfullerene02.github.io/dev-guide/Basis/DealingWithPaks.html) — MEDIUM confidence, community guide
- [Tekken 8 Mod Manager Plus docs](https://www.nexusmods.com/tekken8/mods/231?tab=docs) — MEDIUM confidence, community mod tool
- [Fluffy Mod Manager Tekken 8 support](https://www.patreon.com/posts/fluffy-mod-v3-8-111380741) — MEDIUM confidence, developer announcement
- [MO2 virtual filesystem architecture](https://thebestoftimes.moddinglinked.com/mo2.html) — MEDIUM confidence, community documentation

---
*Pitfalls research for: Desktop mod manager (file-move toggle pattern, Unreal Engine .pak/.ucas/.utoc, Tauri v2, Windows)*
*Researched: 2026-03-04*
