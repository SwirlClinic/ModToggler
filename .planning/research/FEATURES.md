# Feature Research

**Domain:** Auto-update and CI/CD for Tauri v2 desktop app (v1.1 milestone for ModToggler)
**Researched:** 2026-03-08
**Confidence:** HIGH (official Tauri v2 docs, tauri-action repo, multiple verified implementation guides)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any self-updating desktop app must have. Missing these = broken update experience.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Check for updates on app launch | Users expect to be notified without manual checking | LOW | `check()` API on startup; compare remote version to `Cargo.toml`/`tauri.conf.json` version |
| Non-blocking update notification | Update check must not block app usage | LOW | Run check async; show result only when available |
| User-initiated install (not forced) | Users hate forced restarts mid-session; mod operations may be in progress | LOW | Show "Update Available" UI; user clicks "Install" when ready |
| Download progress indication | Users need to know something is happening during download | LOW | `downloadAndInstall()` emits `Started`, `Progress`, `Finished` events with content_length and chunk_length |
| Signed update verification | Users must trust that updates are authentic, not tampered | MEDIUM | Tauri enforces this -- cannot be disabled. Requires key pair generation and CI secret management |
| Automatic latest.json generation in CI | Release artifacts must be machine-readable for the updater to find them | LOW | `tauri-action` generates `latest.json` automatically with `includeUpdaterJson: true` |
| Windows NSIS installer for updates | Standard Windows update mechanism for per-user installs | LOW | NSIS is the default and recommended target; supports ARM64; no admin required for per-user install |
| CI builds on tag push | Releases should be automated, not manual | MEDIUM | GitHub Actions workflow triggered on version tag push; `tauri-action` handles build + upload |

### Differentiators (Competitive Advantage)

Features that improve the update experience beyond the basics. Not required but valuable for a mod manager.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Update prompt preserving mod operation safety | Mod toggling involves file moves; an update during a file operation could corrupt state | LOW | Disable "Install Now" button while file operations are in progress; check transaction journal state |
| Release notes display in-app | Users want to know what changed before deciding to update | LOW | `notes` field from `latest.json` rendered in the update dialog; Tauri provides this automatically |
| "Remind me later" / dismiss update | Users may be mid-session and not want to restart | LOW | Dismiss notification; re-check on next launch or after N hours |
| Passive install mode (progress bar, no interaction) | Smoother UX than showing the full NSIS installer wizard | LOW | Set `windows.installMode: "passive"` in updater config; shows progress bar only |
| Version display in app UI | Users should know what version they're running | LOW | Read from `tauri.conf.json` version at build time; display in settings/about |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Silent/forced auto-update | "Just keep it up to date" | Kills the app on Windows due to NSIS installer limitation; interrupts active mod operations; violates user trust | Notify + prompt UX; user chooses when to install |
| Beta/canary update channels | Power users want bleeding edge | Adds significant infrastructure complexity (separate endpoints, channel switching UI, version comparison logic); overkill for a single-developer project | Ship stable releases only; use GitHub pre-releases for manual beta testing |
| Rollback to previous version | "The update broke something" | Requires storing previous installers, managing rollback state, potentially reverting DB migrations; enormous complexity | Users can download previous version from GitHub Releases manually; keep releases page accessible |
| Delta/differential updates | Smaller downloads | Tauri does not support delta updates; the full installer is downloaded every time; attempting custom delta logic is a maintenance nightmare | Full installer downloads are small (~5-10MB for Tauri apps); not worth optimizing |
| In-app changelog history | "Show me all past changes" | Requires maintaining structured changelog data, rendering engine, pagination; scope creep for an update feature | Link to GitHub Releases page for full history; show only current update's notes |
| Auto-update mods alongside app update | "Update everything at once" | Already ruled out of scope in PROJECT.md; mod updates can break setups; completely different problem domain | Keep app updates and mod management separate |

---

## Feature Dependencies

```
[Signing Key Generation]
    └──required by──> [CI Build Pipeline]
                          └──required by──> [GitHub Release with Artifacts]
                                                └──required by──> [latest.json Endpoint]
                                                                      └──required by──> [Updater Plugin Check]
                                                                                            └──required by──> [Update Notification UI]
                                                                                                                  └──required by──> [Download + Install Flow]

[tauri.conf.json updater config]
    └──required by──> [Updater Plugin Check]

[Capabilities permissions (updater:default)]
    └──required by──> [Updater Plugin Check]

[Version bump workflow]
    └──required by──> [CI Build Pipeline]  (version in tauri.conf.json/Cargo.toml/package.json must match tag)
```

### Dependency Notes

- **Signing key is the foundation:** Without a key pair, no signed artifacts are produced, and the updater refuses to install anything. Generate keys before any other work.
- **CI pipeline must exist before updater works:** The updater checks `latest.json` on GitHub Releases. That file is generated by `tauri-action`. No CI = no `latest.json` = updater has nothing to check.
- **Version consistency is critical:** `tauri.conf.json`, `Cargo.toml`, and `package.json` must all have matching versions. Mismatch causes the updater to either skip valid updates or attempt invalid ones.
- **Update UI depends on working check:** Build the notification UI after confirming the check/download/install flow works end-to-end with a real release.

---

## MVP Definition

### Launch With (v1.1)

Minimum for a working auto-update system.

- [ ] Signing key pair generated and private key stored as GitHub secret
- [ ] `tauri-plugin-updater` integrated in Rust backend with plugin registration
- [ ] `tauri.conf.json` configured with endpoints, pubkey, `createUpdaterArtifacts`, and `windows.installMode: "passive"`
- [ ] Updater permissions added to capabilities (`updater:default`)
- [ ] GitHub Actions workflow: build on tag push, upload artifacts + `latest.json` to GitHub Release
- [ ] Frontend: check for updates on app launch (async, non-blocking)
- [ ] Frontend: "Update Available" notification with version and release notes
- [ ] Frontend: "Install Now" button that calls `downloadAndInstall()` with progress feedback
- [ ] Frontend: version display in settings or title bar

### Add After Validation (v1.x)

Features to add once the basic update flow is proven stable.

- [ ] "Remind me later" with periodic re-check -- trigger: user feedback that the prompt is annoying
- [ ] Guard against updating during active file operations -- trigger: after confirming the transaction journal can detect in-progress operations
- [ ] Manual "Check for Updates" button in settings -- trigger: users report wanting to check on-demand

### Future Consideration (v2+)

Features to defer until there is clear demand.

- [ ] Update channels (beta/stable) -- only if the project grows to need staged rollouts
- [ ] CrabNebula Cloud for managed update infrastructure -- only if GitHub Releases becomes limiting
- [ ] Cross-platform CI (macOS/Linux builds) -- only when cross-platform support is prioritized

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Signing key setup + CI secrets | HIGH (blocker) | LOW | P1 |
| GitHub Actions build workflow | HIGH (blocker) | MEDIUM | P1 |
| Updater plugin integration (Rust) | HIGH | LOW | P1 |
| tauri.conf.json updater config | HIGH (blocker) | LOW | P1 |
| Capabilities permissions | HIGH (blocker) | LOW | P1 |
| Update check on launch | HIGH | LOW | P1 |
| Update notification UI | HIGH | LOW | P1 |
| Download + install with progress | HIGH | LOW | P1 |
| Version display in app | MEDIUM | LOW | P1 |
| Passive install mode (Windows) | MEDIUM | LOW | P1 |
| "Remind me later" dismiss | LOW | LOW | P2 |
| Guard during file operations | MEDIUM | MEDIUM | P2 |
| Manual "Check for Updates" button | LOW | LOW | P2 |
| Beta update channel | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1.1 launch
- P2: Should have, add when feedback warrants
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | Vortex | Fluffy Mod Manager | MO2 | Our Approach |
|---------|--------|--------------------|-----|--------------|
| Auto-update app | Yes (Electron auto-updater) | Yes (custom check) | Yes (GitHub releases check) | Yes -- Tauri updater plugin + GitHub Releases |
| Update notification | System tray notification | In-app dialog | In-app banner | In-app notification banner (non-blocking) |
| Forced updates | No (user chooses) | No | No | No -- user-initiated install only |
| Release notes in update dialog | Yes | No | Brief | Yes -- rendered from `latest.json` notes field |
| Update channels | Stable only | Stable only | Stable + dev builds | Stable only (keep it simple) |
| CI/CD automation | Yes (Nexus internal) | Unknown | GitHub Actions | GitHub Actions with tauri-action |
| Signed updates | Yes (code signing) | Unknown | No formal signing | Yes -- Tauri mandatory signing |
| Progress feedback | Yes | Minimal | Minimal | Yes -- download progress bar from updater events |

---

## Technical Details (for implementers)

### Tauri Updater Plugin Configuration

The updater requires these additions to `tauri.conf.json`:

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "YOUR_PUBLIC_KEY_HERE",
      "endpoints": [
        "https://github.com/OWNER/ModToggler/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

### Windows-Specific Behavior

- **App is force-killed during install:** Windows NSIS installer requires the app to exit. Tauri handles this automatically but the app should use `on_before_exit()` to clean up (e.g., finalize any pending transaction journal entries).
- **Passive install mode:** Shows a progress bar but requires no user interaction during the NSIS install. Recommended over `quiet` (which needs pre-existing admin) or `basicUi` (which shows a full wizard).
- **NSIS over MSI:** NSIS is the recommended target for per-user installs (no admin needed), supports ARM64, and is the default in Tauri v2.

### Update Endpoint Format (latest.json)

Generated automatically by `tauri-action` with `includeUpdaterJson: true`:

```json
{
  "version": "1.1.0",
  "notes": "Release notes here",
  "pub_date": "2026-03-15T10:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "SIGNATURE_CONTENT",
      "url": "https://github.com/OWNER/ModToggler/releases/download/v1.1.0/ModToggler_1.1.0_x64-setup.nsis.zip"
    }
  }
}
```

### Required Secrets for CI

| Secret | Purpose | How to Set |
|--------|---------|------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Signs update artifacts | `tauri signer generate -w ~/.tauri/modtoggler.key`, store key content as GitHub secret |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Unlocks signing key | Password chosen during key generation, store as GitHub secret |
| `GITHUB_TOKEN` | Upload release artifacts | Automatic; ensure workflow has `contents: write` permission |

---

## Sources

- [Tauri v2 Updater Plugin -- Official Docs](https://v2.tauri.app/plugin/updater/) -- HIGH confidence
- [Tauri v2 GitHub Actions Pipeline -- Official Docs](https://v2.tauri.app/distribute/pipelines/github/) -- HIGH confidence
- [tauri-apps/tauri-action -- GitHub](https://github.com/tauri-apps/tauri-action) -- HIGH confidence
- [Tauri v2 Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/) -- HIGH confidence
- [Tauri v2 Auto-Updater with GitHub -- Gurjot](https://thatgurjot.com/til/tauri-auto-updater/) -- MEDIUM confidence (practical implementation walkthrough)
- [Tauri v2 Updater -- Ratul Maharaj](https://ratulmaharaj.com/posts/tauri-automatic-updates/) -- MEDIUM confidence (implementation guide)
- [Ship Tauri v2 App: GitHub Actions and Release Automation -- DEV Community](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- MEDIUM confidence
- [Tauri Updater + GitHub Releases Discussion #10206](https://github.com/orgs/tauri-apps/discussions/10206) -- MEDIUM confidence
- [Beta and Stable Channels Discussion #11069](https://github.com/tauri-apps/tauri/discussions/11069) -- MEDIUM confidence

---

*Feature research for: ModToggler v1.1 -- Auto-Update Releases*
*Researched: 2026-03-08*
