# Roadmap: ModToggler

## Milestones

- ✅ **v1.0 MVP** -- Phases 1-4 (shipped 2026-03-05)
- 🚧 **v1.1 Auto-Update Releases** -- Phases 5-7 (in progress)

## Phases

<details>
<summary>v1.0 MVP (Phases 1-4) -- SHIPPED 2026-03-05</summary>

- [x] Phase 1: Foundation (5/5 plans) -- completed 2026-03-05
- [x] Phase 2: Core Mod Loop (5/5 plans) -- completed 2026-03-05
- [x] Phase 3: Profiles (2/2 plans) -- completed 2026-03-05
- [x] Phase 4: Loose-File Games (3/3 plans) -- completed 2026-03-05

</details>

### v1.1 Auto-Update Releases

- [x] **Phase 5: Updater Foundation** - Signing keypair, plugin registration, and updater configuration
- [ ] **Phase 6: CI/CD Pipeline** - GitHub Actions workflow that builds, signs, and publishes releases
- [ ] **Phase 7: Update UI** - In-app update notification with download, install, and version display

## Phase Details

<details>
<summary>v1.0 MVP Phase Details (Phases 1-4) -- SHIPPED 2026-03-05</summary>

### Phase 1: Foundation
**Goal**: Tauri v2 + React project with SQLite schema, atomic file operations, and crash recovery
**Plans**: 5/5 complete

### Phase 2: Core Mod Loop
**Goal**: Users can import mods from zip, toggle them on/off, and see conflicts
**Plans**: 5/5 complete

### Phase 3: Profiles
**Goal**: Users can save and restore named mod configurations per game
**Plans**: 2/2 complete

### Phase 4: Loose-File Games
**Goal**: Users can manage mods for games with loose files scattered across the game root
**Plans**: 3/3 complete

</details>

### Phase 5: Updater Foundation
**Goal**: App is configured to verify and receive signed updates from GitHub Releases
**Depends on**: Phase 4 (v1.0 complete)
**Requirements**: INFRA-01, INFRA-02, INFRA-03, INFRA-04
**Success Criteria** (what must be TRUE):
  1. Ed25519 signing keypair exists with private key backed up in 2+ secure locations
  2. Running `cargo tauri build` produces `.sig` signature files alongside the installer
  3. Updater plugin is registered in Rust and configured with the public key and GitHub Releases endpoint URL
  4. App builds and launches without errors with all new plugin dependencies and capabilities
**Plans:** 2 plans
Plans:
- [x] 05-01-PLAN.md -- Generate keypair, install dependencies, configure updater plugin and capabilities
- [x] 05-02-PLAN.md -- Run signed build, verify .sig output, user backs up key

### Phase 6: CI/CD Pipeline
**Goal**: Pushing a version tag to GitHub automatically produces a signed release with installer and update manifest
**Depends on**: Phase 5
**Requirements**: CICD-01, CICD-02, CICD-03
**Success Criteria** (what must be TRUE):
  1. Pushing a `v*` tag to GitHub triggers a workflow that builds the Windows NSIS installer
  2. The workflow signs artifacts using the repository's stored signing key secret
  3. A GitHub Release is created containing the `.exe` installer, `.sig` file, and `latest.json` manifest
  4. The `latest.json` contains correct version, download URL, and signature for the updater to consume
**Plans:** 1 plan
Plans:
- [ ] 06-01-PLAN.md -- Create release workflow and verify end-to-end with test tag push

### Phase 7: Update UI
**Goal**: Users are notified of available updates and can install them from within the app
**Depends on**: Phase 6
**Requirements**: UPD-01, UPD-02, UPD-03, UPD-04, UPD-05, UPD-06
**Success Criteria** (what must be TRUE):
  1. App checks for updates on launch without blocking the UI or mod management features
  2. User sees a non-intrusive banner showing the new version number when an update is available
  3. User can view release notes and click "Install" to download the update with a visible progress indicator
  4. Update installs in passive mode (progress bar, no NSIS wizard) and the app relaunches on the new version
  5. User can see the current app version displayed in the UI at any time
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 5/5 | Complete | 2026-03-05 |
| 2. Core Mod Loop | v1.0 | 5/5 | Complete | 2026-03-05 |
| 3. Profiles | v1.0 | 2/2 | Complete | 2026-03-05 |
| 4. Loose-File Games | v1.0 | 3/3 | Complete | 2026-03-05 |
| 5. Updater Foundation | v1.1 | 2/2 | Complete | 2026-03-09 |
| 6. CI/CD Pipeline | v1.1 | 0/1 | Planned | - |
| 7. Update UI | v1.1 | 0/? | Not started | - |
