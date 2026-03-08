# Project Research Summary

**Project:** ModToggler v1.1 -- Auto-Update Releases
**Domain:** Auto-update and CI/CD for Tauri v2 desktop application
**Researched:** 2026-03-08
**Confidence:** HIGH

## Executive Summary

ModToggler v1.1 adds auto-update capability to the existing Tauri v2 desktop mod manager. The update system follows a single established pattern: the official `tauri-plugin-updater` checks a `latest.json` manifest hosted on GitHub Releases, verifies Ed25519 signatures, downloads the NSIS installer, and restarts the app. No custom Rust commands are needed -- the frontend drives the entire flow through the plugin's JavaScript API. This is a straightforward, additive feature with no changes to the existing mod management architecture, database schema, or IPC surface.

The recommended approach is a three-phase build: (1) generate the signing keypair and configure the updater in `tauri.conf.json` plus Rust plugin registration, (2) create the GitHub Actions CI/CD pipeline that builds, signs, and publishes releases on tag push, and (3) build the frontend update notification UI with a React hook and banner component. This ordering follows the strict dependency chain -- nothing works without the signing key, the updater needs `latest.json` from CI before it can be tested, and the UI must be verified against a real published release.

The primary risks are losing the signing keypair (which permanently bricks updates for all installed copies), version desync across `tauri.conf.json`/`Cargo.toml`/`package.json` (which silently breaks update detection), and triggering an update during an active mod file operation (which could corrupt mod state). All three are preventable with one-time setup discipline and a simple pre-install check in the UI.

## Key Findings

### Recommended Stack

The existing v1.0 stack (Tauri 2.x, React 19, TypeScript 5, Vite 6, SQLite, Zustand, TanStack Query, shadcn/ui) is unchanged. The v1.1 additions are minimal and official.

**New dependencies:**
- `tauri-plugin-updater` (Rust + JS): checks GitHub Releases endpoint, downloads and verifies signed updates -- the only supported path for Tauri v2 auto-updates
- `tauri-plugin-process` (Rust + JS): provides `relaunch()` after update install -- tiny footprint, official plugin
- `tauri-apps/tauri-action@v1` (CI): builds Tauri app, creates GitHub Release with installer + `latest.json` in one step
- `swatinem/rust-cache@v2` (CI): caches Rust build artifacts, cuts CI time from ~15min to ~5min

**What NOT to add:** CrabNebula Cloud (overkill), custom update servers (unnecessary), Windows code signing certificates (not required for updater to function, defer), `tauri-plugin-notification` (use in-app dialog instead), beta/canary update channels (scope creep for a single-developer project).

### Expected Features

**Must have (table stakes):**
- Async update check on app launch (non-blocking, 5-second delay)
- User-initiated install only (never auto-install -- users may be mid-mod-operation)
- Download progress indication via plugin events
- Signed update verification (Tauri enforces this, cannot be skipped)
- CI builds on tag push with automatic `latest.json` generation
- NSIS installer in passive mode (progress bar, no wizard)
- Version display in app UI

**Should have (differentiators):**
- Release notes displayed in update prompt (free -- comes from `latest.json` notes field)
- "Remind me later" dismiss option
- Guard against updating during active file operations

**Defer (v2+):**
- Beta/stable update channels
- Rollback to previous version
- Cross-platform CI builds (macOS/Linux)
- Delta/differential updates (Tauri does not support this)

### Architecture Approach

The update system is fully additive and isolated from existing mod management. It introduces three new artifacts: a GitHub Actions workflow file, a React hook (`useUpdateCheck`), and a UI component (`UpdateBanner`). The hook calls the plugin's JS API directly -- no custom Rust IPC commands needed. Update state is React-local, not in Zustand stores or SQLite. Only 6 existing files need modification, all with small, well-defined changes (2 lines in Cargo.toml, 2 lines in package.json, ~15 lines in tauri.conf.json, 2 lines in capabilities, 2 lines in lib.rs, ~3 lines in App.tsx).

**Major components:**
1. GitHub Actions workflow (`.github/workflows/release.yml`) -- builds, signs, and publishes releases on tag push
2. `useUpdateCheck` hook -- encapsulates check/download/install lifecycle with progress tracking
3. `UpdateBanner` component -- non-intrusive notification with install button and progress bar
4. Signing keypair -- Ed25519 keys for update signature verification (one-time generation)

### Critical Pitfalls

1. **Lost signing key bricks all installed copies** -- back up the private key in 2+ secure locations before any other work; this key is irreplaceable without forcing manual reinstall on all users
2. **Version triple desync** -- `tauri.conf.json`, `Cargo.toml`, and `package.json` must have matching versions; add a CI check step or create a bump script
3. **NSIS `perMachine` install mode breaks auto-update** -- keep `currentUser` mode (the default); switching to `perMachine` causes silent error 740 failures
4. **Update during active file operation corrupts mod state** -- disable the install button while file operations are in progress; use `on_before_exit` to finalize journal entries
5. **Missing `createUpdaterArtifacts` config** -- must be set to `true` in `tauri.conf.json` `bundle` section or no `.sig` files are generated and updates silently fail

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Updater Foundation (Config + Dependencies)
**Rationale:** Everything depends on the signing key and plugin registration. This is pure configuration with no behavioral changes to the app. The dependency chain is absolute: no key means no signed artifacts, no signed artifacts means the updater rejects everything.
**Delivers:** Signing keypair generated and backed up, plugins registered in Rust and JS, `tauri.conf.json` configured with updater endpoint/pubkey/`createUpdaterArtifacts`, capabilities updated with `updater:default` and `process:default`, GitHub secrets set, install mode verified as `currentUser`.
**Addresses:** Signing key setup, plugin integration, updater config (all P1 blockers from FEATURES.md)
**Avoids:** Lost signing key (Pitfall 1), missing `createUpdaterArtifacts` (Pitfall 5), endpoint URL misconfiguration (Pitfall 6), NSIS install mode (Pitfall 3)

### Phase 2: CI/CD Pipeline
**Rationale:** The updater needs a published `latest.json` on GitHub Releases to test against. The pipeline must work before the frontend UI can be validated end-to-end. This phase produces the artifact that Phase 3 consumes.
**Delivers:** `.github/workflows/release.yml` that builds, signs, and creates a draft GitHub Release on tag push. Version sync validation. Verified draft release with `.exe`, `.sig`, and `latest.json` artifacts.
**Addresses:** CI builds on tag push (P1), automatic `latest.json` generation (P1)
**Avoids:** GitHub token permissions (Pitfall 7), version triple desync (Pitfall 2)

### Phase 3: Update UI + End-to-End Validation
**Rationale:** The UI can only be properly tested once a real release exists on GitHub. Build the hook and component, then validate the full cycle: check -> notify -> download -> install -> relaunch.
**Delivers:** `useUpdateCheck` hook, `UpdateBanner` component, version display in app, passive install mode with progress bar, end-to-end test against a real published release.
**Addresses:** Update check on launch (P1), update notification UI (P1), download + install with progress (P1), version display (P1), passive install mode (P1)
**Avoids:** Update during file operation (Pitfall 4), auto-install without consent (anti-pattern), no progress feedback (performance trap)

### Phase Ordering Rationale

- Phases follow a strict dependency chain: key generation -> CI pipeline -> frontend UI. Each phase produces an artifact the next phase requires.
- Phase 1 is pure config changes that can be verified locally (build produces `.sig` files).
- Phase 2 produces the GitHub Release that Phase 3 tests against. Without a real `latest.json` on GitHub, the updater has nothing to check.
- Phase 3 is the only phase with user-facing behavior changes.
- This ordering means each phase can be independently verified before moving on, reducing the "looks done but isn't" risk identified in PITFALLS.md.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (CI/CD):** First-time GitHub Actions setup for Tauri. STACK.md references `tauri-action@v1` while ARCHITECTURE.md uses `@v0` -- confirm which version supports Tauri v2 `createUpdaterArtifacts` format. May hit `tauriScript` configuration issues.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Config):** Well-documented in official Tauri docs; copy-paste configuration with clear examples.
- **Phase 3 (Update UI):** Standard React hook + component pattern; reference implementation with full code provided in ARCHITECTURE.md.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All dependencies are official Tauri plugins with stable v2 releases; docs are thorough |
| Features | HIGH | Feature set is small and well-defined; official plugin API covers all requirements; competitor analysis confirms the right feature boundary |
| Architecture | HIGH | Fully additive with no existing system changes; reference implementation patterns available in ARCHITECTURE.md with working code |
| Pitfalls | HIGH | Critical pitfalls sourced from official Tauri issue tracker (#7184, #8223, #9835); NSIS elevation bug is well-documented |

**Overall confidence:** HIGH

### Gaps to Address

- **`tauri-action` version:** STACK.md says `@v1`, ARCHITECTURE.md says `@v0`. Both claim Tauri v2 support. Resolve during Phase 2 implementation by checking the action's README at build time.
- **Windows code signing (Authenticode):** Deferred, but SmartScreen warnings will affect user experience on first install and on every update. Not a blocker for the updater to function, but plan to address before wide distribution.
- **Database migration across updates:** The app already runs migrations on startup via `db::migrations`, but the interaction between update-killed-app and journal recovery on a new binary version has not been tested. Add explicit end-to-end test in Phase 3.
- **Offline/network failure handling:** Updater check should fail gracefully when there is no network. Not covered in detail in any research file -- verify during Phase 3 testing.
- **Version bump workflow:** Research recommends a bump script for syncing 3 files, but no specific tooling is prescribed. Decide during Phase 2 whether to use a shell script, `cargo-release`, or manual bumping.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Updater Plugin](https://v2.tauri.app/plugin/updater/) -- configuration, JS API, signing requirements
- [Tauri v2 GitHub Pipelines](https://v2.tauri.app/distribute/pipelines/github/) -- CI/CD workflow structure
- [Tauri v2 Process Plugin](https://v2.tauri.app/plugin/process/) -- relaunch API
- [Tauri v2 Windows Installer](https://v2.tauri.app/distribute/windows-installer/) -- NSIS configuration, install modes
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) -- action inputs, release creation, updater JSON generation
- [Tauri NSIS elevation issues](https://github.com/tauri-apps/tauri/issues/7184) -- `perMachine` install mode breaks auto-update
- [Tauri NSIS multi-user issue](https://github.com/tauri-apps/tauri/issues/8223) -- elevation edge cases
- [Tauri v2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- Authenticode vs updater signing distinction

### Secondary (MEDIUM confidence)
- [thatgurjot.com auto-updater guide](https://thatgurjot.com/til/tauri-auto-updater/) -- practical implementation walkthrough
- [ratulmaharaj.com updater post](https://ratulmaharaj.com/posts/tauri-automatic-updates/) -- end-to-end configuration
- [DEV Community: Ship Tauri v2](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- CI/CD reference
- [Tauri updater GitHub discussion #10206](https://github.com/orgs/tauri-apps/discussions/10206) -- community Q&A
- [Tauri beta channels discussion #11069](https://github.com/tauri-apps/tauri/discussions/11069) -- why to defer update channels

---
*Research completed: 2026-03-08*
*Ready for roadmap: yes*
