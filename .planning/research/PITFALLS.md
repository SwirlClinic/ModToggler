# Pitfalls Research

**Domain:** Auto-update + CI/CD for existing Tauri v2 desktop app (file-managing mod toggler)
**Researched:** 2026-03-08
**Confidence:** HIGH (Tauri updater config), MEDIUM (CI/CD workflow edge cases), HIGH (data safety during updates)

---

## Critical Pitfalls

### Pitfall 1: Losing the Updater Signing Key Bricks All Installed Copies

**What goes wrong:**
Tauri's updater requires a signature verification using a keypair. The public key is embedded in the app binary. If you lose the private key, you can never sign a valid update again. Every copy of your app already in the wild will fail signature verification and refuse to update. You must ask all users to manually download and reinstall.

**Why it happens:**
The private key is generated once with `tauri signer generate` and stored locally or as a CI secret. Developers treat it like any other credential and don't realize it is irreplaceable -- unlike an API key, you cannot rotate this key without a breaking change to every installed copy.

**How to avoid:**
- Generate the key pair before any other updater work. Store the private key in at least two secure locations (e.g., GitHub Actions secret + encrypted backup in a password manager).
- Document the key generation date and backup locations in a private note.
- The public key goes into `tauri.conf.json` under `plugins.updater.pubkey`. The private key is set as `TAURI_SIGNING_PRIVATE_KEY` environment variable during builds.
- Never commit the private key to the repository.

**Warning signs:**
- Key only exists as a GitHub Actions secret with no other backup
- No documentation of where the key is stored
- Developer cannot locate the key file when asked

**Phase to address:**
First phase -- key generation and secure storage must happen before any build pipeline work.

---

### Pitfall 2: Version Triple Desync (Cargo.toml / tauri.conf.json / package.json)

**What goes wrong:**
Tauri reads the app version from `tauri.conf.json`. The updater compares this version against the remote `latest.json` to decide if an update is available. If `Cargo.toml`, `tauri.conf.json`, and `package.json` have different version strings, the build succeeds but the updater either never detects updates (version in binary is already "higher" than the release) or always detects updates (version stuck at an old value).

The current codebase has `version: "0.1.0"` in both `tauri.conf.json` and `Cargo.toml`. These must be kept in sync and bumped together on every release.

**Why it happens:**
Three files define versions independently. Developers bump one and forget the others. The build does not fail on mismatch -- it silently ships with the wrong version. `tauri-action` uses the `tauri.conf.json` version for the release tag, so a Cargo.toml mismatch is invisible until runtime.

**How to avoid:**
- Create a version bump script (or use `cargo-release` / a simple shell script) that updates all three files atomically.
- Add a CI check step that extracts the version from all three files and fails the build if they differ.
- Alternatively, use `tauri.conf.json` as the single source of truth and have Cargo.toml read from it (Tauri's `tauri.conf.json` version is authoritative for the updater).

**Warning signs:**
- Manual version edits in PRs that only touch one file
- Release tag says v1.2.0 but the about dialog says v1.1.0
- Updater check always returns "up to date" even after publishing a new release

**Phase to address:**
CI/CD pipeline phase -- add the version sync check as a build step.

---

### Pitfall 3: NSIS Installer Mode Breaks Silent Update

**What goes wrong:**
The NSIS installer has three install modes: `currentUser` (default), `perMachine`, and `both`. If you use `perMachine` or `both`, the updater must run the NSIS installer with administrator privileges. But the Tauri updater launches the installer from a non-elevated process, causing error code 740 ("The requested operation requires elevation"). The update silently fails or shows a confusing error.

This is a known Tauri issue (GitHub #7184). The app currently does not specify an install mode, so it defaults to `currentUser`, which is correct. Changing it to `perMachine` later would break auto-update.

**Why it happens:**
Developers switch to `perMachine` to install in Program Files (feels more "professional"), not realizing it creates an elevation requirement that the updater cannot satisfy cleanly.

**How to avoid:**
- Use `currentUser` install mode. This installs to `AppData\Local\Programs\` which does not require elevation. The updater can replace files without UAC prompts.
- Do NOT change to `perMachine` or `both` -- it breaks the silent update flow on Windows.
- If you previously shipped `perMachine` and want to switch, this is a breaking change requiring manual reinstall.
- Document this decision so future contributors do not "fix" it.

**Warning signs:**
- `nsis.installMode` set to anything other than `currentUser` in `tauri.conf.json`
- Users report "update failed" with no visible error (error 740 is swallowed)
- UAC prompt appearing during what should be a silent update

**Phase to address:**
Updater configuration phase -- verify install mode is `currentUser` and lock it down.

---

### Pitfall 4: Update Fires Mid-Toggle, Corrupting Mod State

**What goes wrong:**
Tauri's Windows updater automatically quits the application before running the NSIS installer. If the user triggers an update while a mod toggle operation (file move) is in progress, the app is killed mid-operation. Files end up half-moved -- some in the game directory, some in staging. The transaction journal has an incomplete entry. On next startup after the update, the journal recovery runs, but if the update changed file paths or database schema, recovery may fail.

**Why it happens:**
The default updater behavior (`downloadAndInstall()`) immediately quits the app. Developers test updates when the app is idle, not during active file operations. The app already has a transaction journal for crash recovery, but the interaction between "update kills app" and "journal recovery on new version" is never tested.

**How to avoid:**
- Never auto-install updates. Use a two-step flow: `check()` then show a notification. Only call `downloadAndInstall()` when the user explicitly clicks "Install Update."
- Before starting the install, check if any file operations are in progress (the `AppState` could track this). If operations are active, show "Please wait for the current operation to finish before updating."
- Use the `on_before_exit` hook to finalize or roll back any in-progress journal entries.
- Test the scenario: start a large mod toggle, trigger update, verify journal recovery works after the new version launches.

**Warning signs:**
- Update install happens automatically without user confirmation
- No check for "is the app busy with file operations" before starting install
- No `on_before_exit` handler in the updater flow

**Phase to address:**
Updater integration phase -- the update UI must coordinate with the file operations state.

---

### Pitfall 5: `createUpdaterArtifacts` Not Set -- No .sig Files Generated

**What goes wrong:**
The build runs, produces an NSIS installer, but no `.sig` signature file is generated alongside it. The `latest.json` has no signature to include. The updater on the client downloads the update but fails signature verification and refuses to install. Users see "update available" but the install always fails.

**Why it happens:**
`createUpdaterArtifacts` defaults to `false` in `tauri.conf.json`. Developers add the updater plugin, configure endpoints and pubkey, but forget this bundler setting. The build succeeds because this is a bundler option, not a plugin option -- it is in a different section of the config.

**How to avoid:**
- Add `"createUpdaterArtifacts": true` to the `bundle` section of `tauri.conf.json` immediately when starting updater work.
- Verify that the build output directory contains both the `.exe` installer AND a `.exe.sig` file. If the `.sig` is missing, the config is wrong.
- The CI workflow should have a step that asserts the `.sig` file exists before creating the release.

**Warning signs:**
- Build output has `.exe` but no `.exe.sig` file
- `latest.json` has empty or missing `signature` field
- Client-side updater error about signature verification failure

**Phase to address:**
First phase -- this is a config change that must be in place before any updater testing.

---

### Pitfall 6: Endpoint URL Misconfiguration for GitHub Releases

**What goes wrong:**
The updater endpoint URL is wrong and the app either gets a 404 (no update found, ever) or gets HTML instead of JSON (GitHub's web page, not the raw file). The updater silently fails or throws a parse error.

**Why it happens:**
GitHub Releases has multiple URL patterns. The correct endpoint for `latest.json` is:
```
https://github.com/{owner}/{repo}/releases/latest/download/latest.json
```
Common mistakes:
- Using the API URL (`api.github.com/repos/...`) which returns a different JSON format
- Using the release page URL (`github.com/{owner}/{repo}/releases/latest`) which returns HTML
- Using `{{target}}` and `{{arch}}` placeholders in the URL when using `tauri-action` (which generates a single `latest.json` with all platforms inside, not per-platform files)

**How to avoid:**
- Use the exact URL format above. `tauri-action` uploads `latest.json` as a release asset.
- Test the URL manually in a browser after the first release -- it should return JSON with `version`, `platforms`, `signature` fields.
- Do not include `{{target}}` or `{{arch}}` in the endpoint URL when using GitHub Releases with `tauri-action` -- those placeholders are for custom update servers, not static JSON.

**Warning signs:**
- Updater check returns "up to date" on every version
- Browser navigation to the endpoint URL shows HTML or 404
- Updater logs show JSON parse errors

**Phase to address:**
Updater configuration phase.

---

### Pitfall 7: GitHub Actions Token Permissions Block Release Creation

**What goes wrong:**
The CI workflow runs, builds the app, but fails at the "create release" step with "Resource not accessible by integration" or a 403 error. No release is created, no artifacts uploaded.

**Why it happens:**
By default, `GITHUB_TOKEN` in GitHub Actions has read-only permissions. Creating releases and uploading assets requires write access to `contents`. Developers assume the token has full permissions because it works for checkout and other read operations.

**How to avoid:**
- Add `permissions: contents: write` at the top of the workflow file.
- Alternatively, go to repository Settings > Actions > General > Workflow permissions and select "Read and write permissions."
- The workflow-level `permissions` block is preferred because it is version-controlled and self-documenting.

**Warning signs:**
- CI build succeeds but release step fails with 403/404
- "Resource not accessible by integration" in the GitHub Actions log
- Release exists but has no assets attached

**Phase to address:**
CI/CD pipeline phase -- first workflow run will surface this immediately.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip code signing (no Windows certificate) | Free, no HSM/Azure Key Vault setup | SmartScreen blocks every install; users see scary warnings; auto-update triggers SmartScreen again on each update | MVP/beta only -- plan to sign before public release |
| Hardcode version in three files manually | No tooling to build | Version desync causes updater failures; debugging is painful | Never after first release -- automate immediately |
| Use `installMode: "both"` for flexibility | Users can choose install location | Breaks auto-update silently; error 740 on non-elevated update | Never for an app that uses auto-update |
| Skip database migration strategy for updates | No migration code to write | New version adds columns or tables, SQLite schema is stale, app crashes on startup | Never -- plan migration path from v1.0 |
| Test updater only locally, not end-to-end | Faster development | First real update from GitHub fails due to URL/signature/JSON format issues | Never -- do at least one real end-to-end test before announcing auto-update |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Tauri updater plugin | Adding plugin to Cargo.toml but forgetting to register it with `.plugin(tauri_plugin_updater::Builder::new().build())` in `lib.rs` | Plugin must be registered in the Tauri builder chain AND added to capabilities JSON |
| Tauri updater plugin | Forgetting to add `updater:default` to the capabilities file | The updater commands are permission-gated; without the capability, frontend calls silently fail |
| tauri-action (GitHub) | Using `tauri-action@v0` with a Tauri v2 project but not setting `tauriScript` to match your package manager | If using npm, set `tauriScript: "npx tauri"`. Wrong script = build fails before compilation |
| GitHub Releases endpoint | Using `{{target}}/{{arch}}` placeholders with `tauri-action` | `tauri-action` generates a single `latest.json` with all platforms. Use the flat URL without placeholders |
| Updater signing key | Setting `TAURI_SIGNING_PRIVATE_KEY` in `.env` file for CI | `.env` files are not loaded in GitHub Actions. Must be a GitHub Actions secret passed as an environment variable in the workflow YAML |
| SQLite database + app update | Assuming the database schema is unchanged across versions | New versions may need new tables/columns. Run migrations on startup (the app already does this via `db::migrations`) -- verify this works when the DB was created by an older version |
| Windows NSIS + updater | Not setting `bundle.targets` to `["nsis"]` explicitly | If `targets` is `"all"`, both MSI and NSIS are built. The updater uses NSIS. Dual installers cause confusion (which one is the update artifact?) |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Checking for updates on every app launch synchronously | App startup delayed by 1-5 seconds waiting for network | Check for updates asynchronously after UI renders; show notification when result arrives | Every launch, especially on slow/offline networks |
| Downloading update without progress feedback | UI appears frozen during download of 20-80 MB installer | Use `onProgress` callback from the updater plugin to show download progress bar | Every update on connections slower than 50 Mbps |
| Building all platform targets when only Windows is needed | CI workflow takes 30+ minutes, uses expensive runner minutes | Only build `windows-latest` target until cross-platform is actually needed | Every CI run, compounding over time |
| Not caching Rust build artifacts in CI | Each CI run compiles all dependencies from scratch (10-20 min) | Use `swatinem/rust-cache@v2` action -- caches `target/` directory between runs | Every CI run |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing the updater private key to the repository | Anyone with repo access can sign malicious updates that all installed copies will trust | Store as GitHub Actions secret only; add `*.key` to `.gitignore`; use the key content directly, not a file path, in CI |
| Not validating the update signature (impossible to skip in Tauri, but...) | Tauri enforces this -- but if someone forks and removes it, MITM attacks can push malicious binaries | Keep Tauri's signature verification enabled (it is mandatory by default). Do not set `dangerousInsecureTransportProtocol` in production |
| Shipping without Windows code signing | SmartScreen warns users; antivirus may quarantine the installer; users must click through 3+ scary dialogs | Budget for a code signing certificate (OV at minimum). EV certificates provide immediate SmartScreen reputation. Self-signed is not sufficient |
| Using HTTP (not HTTPS) for update endpoint | MITM can intercept update checks and serve malicious update metadata | Always use HTTPS endpoints. Tauri enforces TLS by default in production builds; do not enable `dangerousInsecureTransportProtocol` |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Auto-installing updates without user consent | App quits while user is mid-task (especially mid-toggle); feels hostile | Show "Update available" notification with "Install Now" and "Later" buttons. Only install when user clicks Install |
| No release notes in the update prompt | User has no idea what the update contains; reluctant to install | Include `notes` field in `latest.json` (tauri-action populates this from the GitHub Release body). Show release notes in the update dialog |
| Update notification on every app launch until installed | Nagging drives users to disable updates or ignore the app | Show notification once per session. Allow "Skip this version" option. Persist dismissed state |
| No indication that the app will restart | User is confused when the app suddenly closes and reopens | Explicitly warn: "The app will close and restart to install the update" before proceeding |
| Force-quit during long mod operations | User loses work; mods end up in broken state | Disable the "Install Now" button while file operations are in progress; show explanation |

---

## "Looks Done But Isn't" Checklist

- [ ] **Updater config:** `createUpdaterArtifacts: true` is set in `tauri.conf.json` -- verify `.sig` files are produced in build output
- [ ] **Endpoint URL:** Manually navigate to the endpoint URL after first release -- verify it returns valid JSON, not HTML or 404
- [ ] **Version comparison:** After publishing v1.1.0, run v1.0.0 and verify the updater detects the update -- do not just check the API
- [ ] **Signature verification:** After downloading the update, verify it actually installs -- a wrong public key causes download success but install failure
- [ ] **Offline handling:** Start the app with no network -- verify the updater check fails gracefully without crashing or blocking startup
- [ ] **Database migration:** Create a database with v1.0.0, then install v1.1.0 over it -- verify the app starts and all data is intact
- [ ] **Journal recovery after update:** Start a toggle operation, kill the app, update to new version, relaunch -- verify journal recovery works with the new binary
- [ ] **CI secret configuration:** Verify `TAURI_SIGNING_PRIVATE_KEY` is set in GitHub Actions secrets -- a missing secret produces a build that appears successful but has no valid signatures
- [ ] **Install mode:** Verify `nsis.installMode` is `currentUser` (or unset, which defaults to `currentUser`) -- anything else breaks auto-update
- [ ] **Tag format:** Verify the tag format in the workflow matches what `tauri-action` expects -- mismatched tag patterns cause the action to skip release creation

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Lost signing key | HIGH | Must generate new keypair, release a new version with new pubkey, ask ALL users to manually download and reinstall. No automatic recovery possible |
| Version desync shipped | LOW | Fix versions in all three files, tag a new patch release. Users who could not update will pick up the corrected release |
| Update corrupts mod state (killed mid-toggle) | MEDIUM | Transaction journal handles this IF the new version's journal recovery code is compatible with the old version's journal format. Test this explicitly |
| Wrong endpoint URL in shipped version | HIGH | Users running the broken version will never find updates. Must release a manual download and hope users find it. Fix the URL for next version |
| NSIS perMachine mode shipped | HIGH | Cannot auto-update back to currentUser. Must ask users to uninstall and reinstall. Switch is a breaking change |
| CI produces unsigned builds | LOW | Fix the CI secret, re-run the workflow, re-tag if needed. Only affects the broken release, not installed copies |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Lost signing key | Phase 1: Key generation + backup | Verify key exists in 2+ locations; verify CI secret is set; do a test build that produces `.sig` files |
| Version triple desync | Phase 2: CI/CD pipeline | Add CI step that extracts version from all three files and fails on mismatch |
| NSIS install mode breaks update | Phase 1: Updater configuration | Verify `installMode` is unset or `currentUser`; test that update installs without UAC prompt |
| Update fires mid-toggle | Phase 3: Update UI | Test: start toggle of large mod, click "Install Update", verify app blocks install until toggle completes |
| Missing `createUpdaterArtifacts` | Phase 1: Updater configuration | Verify build output contains `.sig` files alongside installer |
| Endpoint URL wrong | Phase 1: Updater configuration | After first CI release, verify endpoint returns valid JSON from a browser |
| GitHub token permissions | Phase 2: CI/CD pipeline | First workflow run will surface this; fix immediately |
| Database migration on update | Phase 1: Pre-updater prep | Create a v1.0.0 database, install the new version over it, verify startup succeeds with all data intact |
| No Windows code signing | Phase 2 or deferred | SmartScreen test: download the installer on a clean Windows machine, verify warning level is acceptable |

---

## Sources

- [Tauri v2 Updater Plugin Documentation](https://v2.tauri.app/plugin/updater/) -- HIGH confidence, official docs
- [Tauri v2 GitHub Actions Pipeline](https://v2.tauri.app/distribute/pipelines/github/) -- HIGH confidence, official docs
- [Tauri v2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- HIGH confidence, official docs
- [Tauri NSIS Updater Elevation Bug #7184](https://github.com/tauri-apps/tauri/issues/7184) -- HIGH confidence, official issue tracker
- [NSIS Multi-User Environment Issue #8223](https://github.com/tauri-apps/tauri/issues/8223) -- HIGH confidence, official issue tracker
- [NSIS perMachine Elevation Bug #9835](https://github.com/tauri-apps/tauri/issues/9835) -- HIGH confidence, official issue tracker
- [Auto-Updater User Data Preservation Discussion #7102](https://github.com/tauri-apps/tauri/discussions/7102) -- HIGH confidence, official discussion
- [Tauri v2 Auto-Updater with GitHub Guide](https://thatgurjot.com/til/tauri-auto-updater/) -- MEDIUM confidence, community guide
- [Ship Tauri v2 Like a Pro: CI/CD Part 2](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- MEDIUM confidence, community guide
- [Tauri v2 Updater Blog Post (Ratul)](https://ratulmaharaj.com/posts/tauri-automatic-updates/) -- MEDIUM confidence, community guide
- [SmartScreen Discussion #8046](https://github.com/tauri-apps/tauri/discussions/8046) -- HIGH confidence, official discussion

---
*Pitfalls research for: Auto-update + CI/CD for Tauri v2 desktop mod manager*
*Researched: 2026-03-08*
