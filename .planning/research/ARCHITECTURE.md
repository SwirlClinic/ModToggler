# Architecture Research

**Domain:** Auto-update integration for Tauri v2 desktop app (ModToggler v1.1)
**Researched:** 2026-03-08
**Confidence:** HIGH (Tauri official docs, tauri-action repo, community guides verified)

## System Overview: Update Integration

This documents how the auto-update system integrates with the existing ModToggler architecture. The update system spans three domains: build-time CI/CD, a static update manifest on GitHub Releases, and runtime update checking in the app.

```
+---------------------------------------------------------------------+
|                     GitHub (Build + Distribution)                    |
|                                                                     |
|  +-----------------+    +------------------+    +----------------+  |
|  | GitHub Actions  |--->| GitHub Release   |--->| latest.json    |  |
|  | (tauri-action)  |    | (tagged v1.1.0)  |    | (update        |  |
|  |                 |    |                  |    |  manifest)     |  |
|  | Builds + signs  |    | .msi / .exe      |    | version, sig,  |  |
|  | on tag push     |    | .sig files       |    | download URLs  |  |
|  +-----------------+    +------------------+    +----------------+  |
|                                                        |            |
+--------------------------------------------------------|------------+
                                                         | HTTPS GET
+---------------------------------------------------------------------+
|                    ModToggler App (Runtime)                          |
|                                                                     |
|  +--------------------------------------------------------------+  |
|  |                   React Frontend (Webview)                    |  |
|  |                                                               |  |
|  |  +-----------+  +-----------+  +-------------------+          |  |
|  |  |  Existing |  |  Existing |  |  NEW: UpdateBanner|          |  |
|  |  |  ModList  |  |  Settings |  |  (toast/banner    |          |  |
|  |  |  etc.     |  |  Panel    |  |   with progress)  |          |  |
|  |  +-----------+  +-----------+  +-------------------+          |  |
|  |       |              |               |                        |  |
|  |  +-----------------------------------------------------------+|  |
|  |  |          NEW: useUpdateCheck() hook                        ||  |
|  |  |  calls @tauri-apps/plugin-updater check()                  ||  |
|  |  |  manages download progress, triggers relaunch              ||  |
|  |  +-----------------------------------------------------------+|  |
|  +--------------------------------------------------------------+|  |
|       |                                                          |  |
|  +--------------------------------------------------------------+|  |
|  |                   Rust Backend (Core)                         ||  |
|  |                                                               ||  |
|  |  +-----------+  +-----------+  +-------------------+          ||  |
|  |  |  Existing |  |  Existing |  |  NEW: Updater     |          ||  |
|  |  |  Commands |  |  Services |  |  Plugin           |          ||  |
|  |  |           |  |           |  |  (tauri-plugin-   |          ||  |
|  |  |           |  |           |  |   updater)        |          ||  |
|  |  +-----------+  +-----------+  +-------------------+          ||  |
|  |                                      |                        ||  |
|  |                                +-------------------+          ||  |
|  |                                |  NEW: Process     |          ||  |
|  |                                |  Plugin           |          ||  |
|  |                                |  (tauri-plugin-   |          ||  |
|  |                                |   process)        |          ||  |
|  |                                +-------------------+          ||  |
|  +--------------------------------------------------------------+|  |
+---------------------------------------------------------------------+
```

### Component Responsibilities

**New components only** -- existing components are documented in the v1.0 architecture.

| Component | Responsibility | Type |
|-----------|----------------|------|
| GitHub Actions workflow | Build, sign, and release Windows installer on tag push | NEW file: `.github/workflows/release.yml` |
| tauri-plugin-updater | Rust plugin that checks endpoint, verifies signatures, downloads + installs updates | NEW dependency (Cargo + npm) |
| tauri-plugin-process | Provides `relaunch()` to restart the app after update installs | NEW dependency (Cargo + npm) |
| Signing keypair | Ed25519 key pair for update signature verification | NEW artifact: `~/.tauri/modtoggler.key` (local dev) + GitHub secret |
| UpdateBanner / UpdateToast | Frontend UI showing update availability, download progress, install prompt | NEW React component |
| useUpdateCheck hook | Encapsulates update check logic, progress tracking, error handling | NEW React hook |
| latest.json | Static update manifest uploaded to GitHub Release by tauri-action | NEW build artifact (auto-generated) |
| tauri.conf.json `plugins.updater` | Configuration: public key, GitHub endpoint URL | MODIFIED existing file |
| capabilities/default.json | Add `updater:default` and `process:default` permissions | MODIFIED existing file |
| Cargo.toml | Add `tauri-plugin-updater` and `tauri-plugin-process` deps | MODIFIED existing file |
| package.json | Add `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` deps | MODIFIED existing file |
| lib.rs | Register updater and process plugins in Builder chain | MODIFIED existing file |

## New vs Modified Files

### New Files

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | GitHub Actions workflow for building and publishing releases |
| `src/hooks/useUpdateCheck.ts` | React hook for update lifecycle (check, download, install, relaunch) |
| `src/components/UpdateBanner.tsx` | UI component for update notifications and download progress |

### Modified Files

| File | Change | Scope |
|------|--------|-------|
| `src-tauri/Cargo.toml` | Add `tauri-plugin-updater` and `tauri-plugin-process` dependencies | 2 lines in `[dependencies]` |
| `package.json` | Add `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` | 2 lines in `dependencies` |
| `src-tauri/tauri.conf.json` | Add `bundle.createUpdaterArtifacts`, `plugins.updater` config block | ~15 lines added |
| `src-tauri/capabilities/default.json` | Add `"updater:default"` and `"process:default"` to permissions array | 2 lines |
| `src-tauri/src/lib.rs` | Register `.plugin(tauri_plugin_updater::Builder::new().build())` and `.plugin(tauri_plugin_process::init())` | 2 lines in Builder chain |
| `src/App.tsx` | Mount `UpdateBanner` component (or integrate check into layout) | ~3 lines |

## Recommended Project Structure (Additions Only)

```
ModToggler/
+-- .github/
|   +-- workflows/
|       +-- release.yml             # NEW: Build + release on tag push
+-- src/
|   +-- components/
|   |   +-- UpdateBanner.tsx        # NEW: Update notification UI
|   +-- hooks/
|   |   +-- useUpdateCheck.ts       # NEW: Update check + install hook
+-- src-tauri/
|   +-- Cargo.toml                  # MODIFIED: add updater + process plugins
|   +-- tauri.conf.json             # MODIFIED: add updater config
|   +-- capabilities/
|       +-- default.json            # MODIFIED: add updater + process perms
```

### Structure Rationale

- **No new Rust commands or services needed.** The updater plugin handles everything internally -- check, download, verify signature, install. The frontend drives the flow via the JS plugin API. No custom Rust IPC commands are required.
- **Single hook + single component pattern.** Matches the existing architecture where hooks wrap invoke/plugin calls and components consume hooks. `useUpdateCheck` is analogous to `useModToggle`.
- **GitHub Actions in standard location.** `.github/workflows/` is the only place GitHub recognizes workflow files.

## Architectural Patterns

### Pattern 1: Frontend-Driven Update Flow

**What:** The update check, download, and install are driven entirely from the React frontend using `@tauri-apps/plugin-updater`. No custom Rust commands are needed. The plugin's JS API talks directly to the Rust plugin internals.

**When to use:** Always for Tauri updater. The plugin is designed as a self-contained system.

**Trade-offs:** Less control from Rust side, but avoids duplicating what the plugin already does well. If you need custom pre-update logic (e.g., saving unsaved state), do it in the hook before calling `downloadAndInstall()`.

**Example:**
```typescript
// hooks/useUpdateCheck.ts
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { useState, useCallback, useEffect } from 'react';

interface UpdateState {
  status: 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';
  version?: string;
  notes?: string;
  progress?: { downloaded: number; total: number | null };
  error?: string;
}

export function useUpdateCheck() {
  const [state, setState] = useState<UpdateState>({ status: 'idle' });
  const [update, setUpdate] = useState<Update | null>(null);

  const checkForUpdate = useCallback(async () => {
    setState({ status: 'checking' });
    try {
      const result = await check();
      if (result) {
        setUpdate(result);
        setState({
          status: 'available',
          version: result.version,
          notes: result.body ?? undefined,
        });
      } else {
        setState({ status: 'idle' });
      }
    } catch (err) {
      setState({ status: 'error', error: String(err) });
    }
  }, []);

  const installUpdate = useCallback(async () => {
    if (!update) return;
    setState(prev => ({ ...prev, status: 'downloading', progress: { downloaded: 0, total: null } }));
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          setState(prev => ({
            ...prev,
            progress: { downloaded: 0, total: event.data.contentLength ?? null },
          }));
        } else if (event.event === 'Progress') {
          setState(prev => ({
            ...prev,
            progress: {
              downloaded: (prev.progress?.downloaded ?? 0) + event.data.chunkLength,
              total: prev.progress?.total ?? null,
            },
          }));
        } else if (event.event === 'Finished') {
          setState(prev => ({ ...prev, status: 'ready' }));
        }
      });
      await relaunch();
    } catch (err) {
      setState({ status: 'error', error: String(err) });
    }
  }, [update]);

  // Check on mount (after short delay to not block startup)
  useEffect(() => {
    const timer = setTimeout(checkForUpdate, 5000);
    return () => clearTimeout(timer);
  }, [checkForUpdate]);

  return { ...state, checkForUpdate, installUpdate };
}
```

### Pattern 2: Passive Update Notification (Not Auto-Install)

**What:** The app checks for updates on startup (after a delay) and shows a non-intrusive banner/toast. The user explicitly clicks "Install" to proceed. The app never downloads or installs without user consent.

**When to use:** Always for ModToggler. Users managing game mods need control over when their tool restarts -- they may be in the middle of configuring mods before a gaming session.

**Trade-offs:** Users may ignore updates, but this is preferable to interrupting a mod management session.

**Example:**
```typescript
// components/UpdateBanner.tsx
// Renders conditionally based on useUpdateCheck().status
// 'available' -> "Version {version} available — Install now"
// 'downloading' -> progress bar
// 'error' -> dismissible error message
```

### Pattern 3: Version Synchronization

**What:** The version in `tauri.conf.json`, `Cargo.toml`, and `package.json` must stay synchronized. The updater compares the running app's version (from `tauri.conf.json`) against the `version` field in `latest.json`. A mismatch = update available.

**When to use:** Every release. The GitHub Actions workflow reads the version from `tauri.conf.json` to create the tag.

**Trade-offs:** Manual version bumping is error-prone. Consider a `bump-version` script or use `cargo-bump` + a pre-tag script, but for a solo project, manual bumping in 3 files is manageable.

## Data Flow

### Update Check Flow

```
App starts
    |
    v (5 second delay)
useUpdateCheck.checkForUpdate()
    |
    v
@tauri-apps/plugin-updater check()
    |
    v
[Rust plugin] GET https://github.com/user/ModToggler/releases/latest/download/latest.json
    |
    v
Compare latest.json.version vs current app version (from tauri.conf.json)
    |
    +-- No update: return null --> setState('idle')
    |
    +-- Update available: return Update object --> setState('available')
         |
         v
    UpdateBanner renders: "v{version} available - Install"
         |
         v (user clicks Install)
    update.downloadAndInstall()
         |
         v
    [Rust plugin] Download .msi/.exe from URL in latest.json
    [Rust plugin] Verify Ed25519 signature against pubkey in tauri.conf.json
    [Rust plugin] Run installer (NSIS passive mode -- progress bar, no user input)
         |
         v
    relaunch() --> app restarts with new version
```

### CI/CD Build Flow

```
Developer bumps version in tauri.conf.json + Cargo.toml + package.json
    |
    v
git tag v1.1.0 && git push --tags
    |
    v
GitHub Actions triggers on tag push (v*)
    |
    v
Workflow: checkout -> setup Node + Rust -> npm install -> tauri build
    |
    v
tauri-action builds with TAURI_SIGNING_PRIVATE_KEY from GitHub secret
    |
    +-- Generates: ModToggler_1.1.0_x64-setup.nsis.exe
    +-- Generates: ModToggler_1.1.0_x64-setup.nsis.exe.sig
    +-- Generates: latest.json (platform entries with signature + download URL)
    |
    v
Creates GitHub Release (draft or published) with all artifacts
    |
    v
App instances poll latest.json on next check -> see new version
```

### Key Integration Points

1. **No existing data flows change.** The updater is fully additive -- it does not modify how mods, games, profiles, or file operations work.
2. **No database changes.** The updater does not need to store anything in SQLite.
3. **No new IPC commands.** The plugin handles its own IPC internally.
4. **State isolation.** Update state lives in the `useUpdateCheck` hook (React state), completely separate from Zustand mod/game stores. No cross-contamination.

## Configuration Details

### tauri.conf.json Additions

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "<GENERATED_PUBLIC_KEY_CONTENT>",
      "endpoints": [
        "https://github.com/USER/ModToggler/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

Key decisions:
- **`createUpdaterArtifacts: true`** (not `"v1Compatible"`) -- this is a new app, not migrating from Tauri v1.
- **Single endpoint** pointing to GitHub Releases `latest/download/latest.json`. The tauri-action auto-generates and uploads this file.
- **`installMode: "passive"`** -- shows a progress bar during install but requires no user clicks. Alternative is `"quiet"` (fully silent) but passive gives visual feedback.

### capabilities/default.json

```json
{
  "permissions": [
    "core:default",
    "sql:default",
    "dialog:default",
    "fs:default",
    "updater:default",
    "process:default"
  ]
}
```

`updater:default` grants: `allow-check`, `allow-download`, `allow-install`, `allow-download-and-install`.
`process:default` grants: `allow-exit`, `allow-restart`.

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-release:
    permissions:
      contents: write
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'

      - uses: dtolnay/rust-toolchain@stable

      - run: npm ci

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'ModToggler v__VERSION__'
          releaseBody: 'See the assets to download and install this version.'
          releaseDraft: true
          prerelease: false
          updaterJsonPreferNsis: true
```

Key decisions:
- **Windows-only matrix.** ModToggler is Windows-primary. No need for macOS/Linux runners yet. Add them later if cross-platform support is needed.
- **Tag-triggered, not branch-triggered.** Explicit `git tag v1.1.0` is more intentional than pushing to a release branch.
- **`releaseDraft: true`** -- creates a draft release so you can review before publishing. Publish manually from GitHub UI. Once published, `latest/download/latest.json` becomes accessible.
- **`updaterJsonPreferNsis: true`** -- use NSIS installer (`.exe`) over WiX (`.msi`) for the update target. NSIS supports passive/quiet install modes better.
- **`__VERSION__`** placeholder is auto-replaced by tauri-action with the version from `tauri.conf.json`.

### GitHub Repository Secrets Required

| Secret | Value | Where to generate |
|--------|-------|-------------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Content of the private key file (not a path) | `npx tauri signer generate -w ~/.tauri/modtoggler.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password used during key generation | Same command as above |

`GITHUB_TOKEN` is auto-provided by GitHub Actions -- no manual setup needed.

## Rust Plugin Registration

In `lib.rs`, add two plugins to the existing Builder chain:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    // ... existing plugins ...
    .plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:modtoggler.db", db::migrations::get_migrations())
            .build(),
    )
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    // ... rest of setup
```

The updater plugin uses `Builder::new().build()` (not just `init()`) because it has configurable options. The defaults are fine -- configuration comes from `tauri.conf.json`.

## Anti-Patterns

### Anti-Pattern 1: Checking for Updates on Every User Action

**What people do:** Poll for updates frequently or check before each mod toggle.
**Why it's wrong:** Each check is an HTTPS request to GitHub. Rate-limited, wastes bandwidth, and interrupts the user's workflow.
**Do this instead:** Check once on app startup (after a 5-second delay) and optionally expose a manual "Check for Updates" button in Settings. No background polling timer.

### Anti-Pattern 2: Auto-Installing Without User Consent

**What people do:** Detect update, immediately download and install, restart the app.
**Why it's wrong:** The user may be in the middle of toggling mods before launching a game. An unexpected restart loses their mental context and may leave mods in an inconsistent state if a toggle was in progress.
**Do this instead:** Show a non-intrusive notification. Let the user choose when to install. The update downloads and installs only when they click "Install."

### Anti-Pattern 3: Skipping Signature Verification

**What people do:** Disable signing to "simplify" the build process, or use a test key in production.
**Why it's wrong:** Without signature verification, a MITM attack or compromised CDN could push malicious binaries to all users. The updater plugin will refuse to install unsigned updates, so skipping signing means updates silently fail.
**Do this instead:** Generate a real key pair. Store the private key as a GitHub secret. The public key in `tauri.conf.json` is safe to commit. This is a one-time setup cost.

### Anti-Pattern 4: Writing Custom Rust Update Commands

**What people do:** Create `#[command] fn check_update()` functions that wrap the updater plugin from Rust, then expose them via tauri-specta like other commands.
**Why it's wrong:** Adds unnecessary IPC round-trips. The `@tauri-apps/plugin-updater` JS API already talks directly to the Rust plugin internals. Custom commands add code to maintain and test with no benefit.
**Do this instead:** Use the JS plugin API directly from the React hook. The only Rust change needed is registering the plugin in `lib.rs`.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| GitHub Releases | Static file hosting for `latest.json` + installer binaries | Free for public repos. `latest/download/` always points to the most recent non-draft, non-prerelease release. |
| GitHub Actions | CI/CD runner for building Windows installer | Free tier: 2,000 minutes/month for public repos. Windows runner uses ~2x minutes. A Tauri build takes ~5-10 min. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| UpdateBanner <-> useUpdateCheck | React hook return values | Standard hook pattern, no special coupling |
| useUpdateCheck <-> plugin-updater | Direct JS API calls | No IPC layer to maintain; plugin handles it |
| plugin-updater <-> GitHub Releases | HTTPS GET to endpoint URL | Configured in tauri.conf.json, not in code |
| tauri-action <-> tauri.conf.json | Reads version and bundle config at build time | Version must be correct before tagging |
| Update system <-> Existing mod system | None -- fully isolated | Update state is React-local, not in Zustand stores or SQLite |

## Build Order Recommendation

Based on dependency analysis, implement in this order:

1. **Signing key generation + configuration** (no code changes, but everything depends on it)
   - Generate key pair
   - Add public key to `tauri.conf.json`
   - Store private key + password as GitHub secrets
   - Add `createUpdaterArtifacts: true` to `tauri.conf.json`

2. **Rust plugin registration** (minimal code, enables everything else)
   - Add `tauri-plugin-updater` and `tauri-plugin-process` to `Cargo.toml`
   - Add `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` to `package.json`
   - Register plugins in `lib.rs`
   - Add permissions to `capabilities/default.json`

3. **GitHub Actions workflow** (can test independently of frontend UI)
   - Create `.github/workflows/release.yml`
   - Test with a manual tag push
   - Verify draft release contains `.exe`, `.sig`, and `latest.json`

4. **Frontend update UI** (needs a published release to test against)
   - Create `useUpdateCheck` hook
   - Create `UpdateBanner` component
   - Mount in `App.tsx`
   - Test by installing an older version and verifying it detects the newer release

## Sources

- [Tauri v2 Updater Plugin](https://v2.tauri.app/plugin/updater/) -- official documentation
- [Tauri v2 Process Plugin](https://v2.tauri.app/plugin/process/) -- official documentation
- [Tauri v2 GitHub Distribution Guide](https://v2.tauri.app/distribute/pipelines/github/) -- official documentation
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) -- official GitHub Action for building and releasing
- [@tauri-apps/plugin-updater JS API Reference](https://v2.tauri.app/reference/javascript/updater/) -- official API docs
- [Tauri v2 Auto-Updater with GitHub](https://thatgurjot.com/til/tauri-auto-updater/) -- community guide with practical gotchas
- [Tauri v2 Updater Blog Post](https://ratulmaharaj.com/posts/tauri-automatic-updates/) -- community reference

---
*Architecture research for: Auto-update integration (ModToggler v1.1)*
*Researched: 2026-03-08*
