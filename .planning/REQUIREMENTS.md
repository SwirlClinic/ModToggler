# Requirements: ModToggler

**Defined:** 2026-03-08
**Core Value:** Users can quickly toggle mods on and off without manually moving files, with confidence the app tracks what belongs to which mod.

## v1.1 Requirements

Requirements for auto-update releases milestone. Each maps to roadmap phases.

### Update Infrastructure

- [ ] **INFRA-01**: App uses Ed25519 signing keypair for update artifact verification
- [ ] **INFRA-02**: Tauri updater plugin is registered and configured with public key and GitHub Releases endpoint
- [ ] **INFRA-03**: Updater capabilities/permissions are granted in the app's capability config
- [ ] **INFRA-04**: `createUpdaterArtifacts` is enabled in bundle config to generate `.sig` files

### Update UX

- [ ] **UPD-01**: App checks for updates on launch without blocking the UI
- [ ] **UPD-02**: User sees a notification banner when an update is available, showing the new version
- [ ] **UPD-03**: User can view release notes for the available update before deciding to install
- [ ] **UPD-04**: User can click "Install" to download and install the update with a progress indicator
- [ ] **UPD-05**: Update installs in passive mode (progress bar only, no NSIS wizard)
- [ ] **UPD-06**: User can see the current app version in the UI

### CI/CD Pipeline

- [ ] **CICD-01**: GitHub Actions workflow builds Windows NSIS installer on version tag push
- [ ] **CICD-02**: CI signs artifacts using the stored signing key secret
- [ ] **CICD-03**: CI uploads installer + `latest.json` to a GitHub Release automatically

## Future Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Update UX Enhancements

- **UPD-07**: User can dismiss update notification with "Remind me later"
- **UPD-08**: App guards against installing updates during active file operations
- **UPD-09**: User can manually check for updates from a settings button

### Distribution

- **DIST-01**: Cross-platform CI builds (macOS/Linux) when cross-platform support is added
- **DIST-02**: Beta/stable update channels for staged rollouts

## Out of Scope

| Feature | Reason |
|---------|--------|
| Silent/forced auto-update | Kills app on Windows (NSIS limitation); interrupts mod operations; violates user trust |
| Delta/differential updates | Tauri does not support delta updates; full installer is small (~5-10MB) |
| Rollback to previous version | Enormous complexity (stored installers, DB migration rollback); users can download from GitHub Releases |
| Beta update channels | Overkill for single-developer project; use GitHub pre-releases for manual testing |
| In-app changelog history | Scope creep; link to GitHub Releases for full history |
| Windows code signing (EV/OV cert) | Cost ($100-400/yr); defer until SmartScreen warnings become a user issue |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01 | Phase 5 | Pending |
| INFRA-02 | Phase 5 | Pending |
| INFRA-03 | Phase 5 | Pending |
| INFRA-04 | Phase 5 | Pending |
| UPD-01 | Phase 7 | Pending |
| UPD-02 | Phase 7 | Pending |
| UPD-03 | Phase 7 | Pending |
| UPD-04 | Phase 7 | Pending |
| UPD-05 | Phase 7 | Pending |
| UPD-06 | Phase 7 | Pending |
| CICD-01 | Phase 6 | Pending |
| CICD-02 | Phase 6 | Pending |
| CICD-03 | Phase 6 | Pending |

**Coverage:**
- v1.1 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-08 after roadmap creation*
