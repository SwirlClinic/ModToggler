# Stack Research

**Domain:** Tauri v2 desktop mod manager (file toggling, game mod management)
**Researched:** 2026-03-08 (v1.1 additions), 2026-03-04 (v1.0 base)
**Confidence:** HIGH (Tauri updater plugin is well-documented, GitHub Actions workflow is official)

---

## v1.1 Auto-Update Stack Additions

These are the NEW dependencies required for the auto-update releases milestone. The existing v1.0 stack (Tauri v2, React 19, SQLite, TanStack Query, Zustand, shadcn/ui, Tailwind CSS, tauri-specta) remains unchanged.

### New Rust Dependencies

| Crate | Version | Purpose | Why Recommended |
|-------|---------|---------|-----------------|
| tauri-plugin-updater | 2 | Check for updates against GitHub Releases endpoint, download and install update artifacts | Official Tauri plugin; handles signature verification, platform detection, download with progress callbacks; the only supported path for Tauri v2 auto-updates |
| tauri-plugin-process | 2 | Relaunch the app after installing an update | Required for `app.restart()` after update install; official plugin, tiny footprint |

### New JavaScript Dependencies

| Package | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| @tauri-apps/plugin-updater | ^2.0.0 | Frontend API: `check()` for updates, `downloadAndInstall()` with progress events | Official JS bindings for the updater plugin; provides typed `Update` object with version, body, date, and download progress callback |
| @tauri-apps/plugin-process | ^2.0.0 | Frontend API: `relaunch()` after update install | Official JS bindings; `relaunch()` restarts the app after update is applied |

### CI/CD Tooling (not app dependencies)

| Tool | Version | Purpose | Why Recommended |
|------|---------|---------|-----------------|
| tauri-apps/tauri-action | v1 | GitHub Action: builds Tauri app, creates GitHub Release, uploads installers + latest.json | Official action from Tauri team; handles NSIS/MSI bundling, updater artifact generation, release creation in one step; `uploadUpdaterJson: true` (default) generates the latest.json the updater plugin consumes |
| actions/checkout | v4 | Checkout repo in CI | Standard |
| actions/setup-node | v4 | Install Node.js with npm cache | Standard |
| dtolnay/rust-toolchain | stable | Install Rust toolchain | Standard; lightweight alternative to actions-rs |
| swatinem/rust-cache | v2 | Cache Rust build artifacts across CI runs | Cuts CI build time from ~15min to ~5min on subsequent runs |

### Signing (required by updater)

| Tool | Purpose | Notes |
|------|---------|-------|
| `tauri signer generate` | Generate Ed25519 keypair for update signature verification | Run once locally: `npx tauri signer generate -w ~/.tauri/modtoggler.key`. Public key goes in tauri.conf.json, private key goes in GitHub Secrets |

---

## Configuration Changes Required

### tauri.conf.json additions

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "endpoints": [
        "https://github.com/OWNER/ModToggler/releases/latest/download/latest.json"
      ]
    }
  },
  "bundle": {
    "createUpdaterArtifacts": true
  }
}
```

**Key points:**
- `pubkey` is the PUBLIC key from `tauri signer generate` (safe to commit)
- `endpoints` points to the latest GitHub Release's `latest.json` artifact
- `createUpdaterArtifacts` tells the bundler to produce `.nsis.zip` + `.nsis.zip.sig` alongside the normal `.exe` installer
- The updater uses NSIS installer format on Windows by default

### Capabilities (src-tauri/capabilities/default.json)

Add these permissions:
```json
{
  "permissions": [
    "updater:default",
    "process:default"
  ]
}
```

`updater:default` grants: `allow-check`, `allow-download`, `allow-install`, `allow-download-and-install`.
`process:default` grants: `allow-exit`, `allow-restart`.

**Note:** If no capabilities file exists yet (permissions may be inline in tauri.conf.json), create `src-tauri/capabilities/default.json` with both existing and new permissions.

### Rust plugin registration (src-tauri/src/lib.rs or main.rs)

```rust
tauri::Builder::default()
    // existing plugins...
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
```

**Important:** The updater plugin uses `Builder::new().build()` (not `init()`) because it supports configuration options like custom headers or target overrides.

### GitHub Secrets Required

| Secret | Value | Purpose |
|--------|-------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | Content of the private key from `tauri signer generate` | Signs update artifacts so the app can verify they came from you |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password set during key generation | Decrypts the private key during CI builds |

`GITHUB_TOKEN` is automatically available in GitHub Actions workflows.

---

## Installation Commands

```bash
# Rust side (run from src-tauri/)
cargo add tauri-plugin-updater
cargo add tauri-plugin-process

# JavaScript side (run from project root)
npm install @tauri-apps/plugin-updater @tauri-apps/plugin-process

# Generate signing keypair (run once, locally)
npx tauri signer generate -w ~/.tauri/modtoggler.key
# Save the public key to tauri.conf.json "plugins.updater.pubkey"
# Save the private key content to GitHub repo secret TAURI_SIGNING_PRIVATE_KEY
```

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| CrabNebula Cloud | Paid service for update hosting; GitHub Releases is free and sufficient for this project's scale | GitHub Releases endpoint with latest.json |
| Custom update server | Unnecessary infrastructure; GitHub Releases serves static files reliably and handles CDN distribution | GitHub Releases endpoint |
| tauri-plugin-autostart | Not needed for update functionality; updates are checked on app launch, not via background service | Manual check on app startup or periodic interval |
| Electron-updater patterns | Different ecosystem; Tauri has its own signing and update format | tauri-plugin-updater |
| Windows code signing certificate (Authenticode) | Separate from updater signing; nice-to-have for removing SmartScreen warnings but costs $200-400/year; not required for the updater to function | Updater Ed25519 signing (free, built-in) |
| tauri-plugin-notification | Tempting for update alerts, but a simple in-app dialog (shadcn Alert or Dialog) is less intrusive and doesn't require OS notification permissions | shadcn/ui Dialog component for update prompt |

---

## GitHub Actions Workflow Structure

The workflow needs to:
1. Trigger on version tag push (e.g., `v*`)
2. Build for Windows (primary target)
3. Sign update artifacts with `TAURI_SIGNING_PRIVATE_KEY`
4. Create GitHub Release with installer + latest.json

```yaml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: windows-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - run: npm ci
      - uses: tauri-apps/tauri-action@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'ModToggler v__VERSION__'
          releaseDraft: true
          prerelease: false
```

**Key decisions:**
- **Windows-only for now** because the project targets Windows; add macOS/Linux matrix later if cross-platform becomes a goal
- **`releaseDraft: true`** so you can review the release before publishing; the updater only sees published releases
- **Tag-triggered** (not branch-triggered) because version bumps should be intentional
- **`v__VERSION__`** reads the version from `tauri.conf.json` automatically

---

## Version Sync Strategy

The version in `tauri.conf.json` drives everything:
- The tag name (via `__VERSION__` placeholder in tauri-action)
- The version the updater compares against when checking for updates
- The version shown in the GitHub Release

**Bump workflow:** Update `version` in `tauri.conf.json` (and optionally `package.json` + `Cargo.toml`) -> commit -> push tag -> CI builds and releases.

The `latest.json` generated by tauri-action contains the version from `tauri.conf.json` and the updater compares it against the running app's version using semver.

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| GitHub Releases endpoint | CrabNebula Cloud | If you need analytics on update adoption rates, staged rollouts, or serve millions of users; overkill for ModToggler |
| GitHub Releases endpoint | Self-hosted static JSON | If GitHub is unavailable in target users' region; unlikely for a modding tool |
| tauri-apps/tauri-action@v1 | Manual `tauri build` + gh CLI | If you need highly custom release artifacts; the action handles signing, zipping, latest.json in one step |
| NSIS installer (default) | MSI installer | If deploying to enterprise/managed Windows environments; NSIS is standard for consumer desktop apps and supports auto-update out of the box |
| Ed25519 updater signing | No signing | Not possible; the updater plugin requires signature verification and will reject unsigned updates |
| Draft releases | Auto-publish | If you have full test coverage and trust the CI pipeline; drafts let you verify before users see the update |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| tauri-plugin-updater 2.x | tauri 2.x | Must match Tauri major version; plugin tracks Tauri releases |
| tauri-plugin-process 2.x | tauri 2.x | Same version alignment requirement |
| @tauri-apps/plugin-updater 2.x | @tauri-apps/api 2.x | JS bindings must match Rust plugin major version |
| tauri-apps/tauri-action@v1 | tauri 2.x | v1 of the action supports Tauri v2; `@v0` also works but v1 is current |
| tauri-plugin-updater 2.10+ | tauri-action latest.json format | tauri-action now includes `{os}-{arch}-{installer}` keys; updater 2.10+ required for this format |

---

## Existing v1.0 Stack (unchanged)

The following stack is already in place and validated. Listed for completeness -- do not reinstall or reconfigure.

### Core Technologies

| Technology | Version | Purpose |
|------------|---------|---------|
| Tauri | 2.x | App shell, IPC, file system |
| React | 19.x | Frontend UI |
| TypeScript | 5.x | Type safety |
| Vite | 6.x | Build tool |
| Rust (stable) | 1.77.2+ | Backend |

### State & Data

| Library | Version | Purpose |
|---------|---------|---------|
| Zustand | 5.x | UI state |
| TanStack Query | 5.x | Async data |
| TanStack Router | 1.x | Routing |
| sqlx + tauri-plugin-sql | 0.8 / 2.x | SQLite |
| tauri-specta | 2.0.0-rc.21 | Typed IPC |

### UI

| Library | Purpose |
|---------|---------|
| Tailwind CSS 4.x | Styling |
| shadcn/ui | Components |
| Lucide React | Icons |

---

## Sources

- [Tauri v2 Updater Plugin docs](https://v2.tauri.app/plugin/updater/) -- setup, configuration, JS API, signing (HIGH confidence)
- [Tauri v2 GitHub Pipelines docs](https://v2.tauri.app/distribute/pipelines/github/) -- GitHub Actions workflow structure (HIGH confidence)
- [tauri-apps/tauri-action GitHub](https://github.com/tauri-apps/tauri-action) -- action inputs, v1 release, uploadUpdaterJson default (HIGH confidence)
- [tauri-plugin-updater crates.io](https://crates.io/crates/tauri-plugin-updater) -- version 2.9.0+ confirmed on crates.io (MEDIUM confidence -- exact latest version unverifiable due to crates.io fetch failure)
- [@tauri-apps/plugin-updater npm](https://www.npmjs.com/@tauri-apps/plugin-updater) -- version 2.10.0 reported (MEDIUM confidence)
- [Tauri v2 Process Plugin docs](https://v2.tauri.app/plugin/process/) -- relaunch API (HIGH confidence)
- [thatgurjot.com Tauri auto-updater guide](https://thatgurjot.com/til/tauri-auto-updater/) -- practical setup walkthrough (MEDIUM confidence)
- [ratulmaharaj.com Tauri v2 updater](https://ratulmaharaj.com/posts/tauri-automatic-updates/) -- end-to-end configuration (MEDIUM confidence)

---

*Stack research for: ModToggler v1.1 Auto-Update Releases*
*Researched: 2026-03-08*
