# Phase 5: Updater Foundation - Research

**Researched:** 2026-03-08
**Domain:** Tauri v2 updater plugin, Ed25519 signing, NSIS bundle configuration
**Confidence:** HIGH

## Summary

Phase 5 configures the ModToggler app to verify and receive signed updates from GitHub Releases. This is pure infrastructure -- no UI is built, no update check logic runs. The deliverables are: a signing keypair, the updater and process plugins wired into the app, configuration pointing at the GitHub Releases endpoint, and builds that produce `.sig` signature files.

The Tauri v2 updater plugin (`tauri-plugin-updater`) uses Minisign (Ed25519) signatures to verify update authenticity. The plugin reads a public key and endpoint list from `tauri.conf.json`, and the build system signs artifacts when `TAURI_SIGNING_PRIVATE_KEY` is set. The `tauri-plugin-process` plugin is needed for app relaunch after update install (Phase 7) and should be added now to avoid a second round of dependency/capability changes.

**Primary recommendation:** Add both `tauri-plugin-updater` and `tauri-plugin-process` with version `"2"` (semver-compatible), configure `tauri.conf.json` with the public key and GitHub Releases endpoint, enable `createUpdaterArtifacts: true`, and set NSIS install mode to `"passive"`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Passwordless Ed25519 signing key (no TAURI_SIGNING_PRIVATE_KEY_PASSWORD needed)
- Key generation scripted as a helper task (not manual docs-only)
- Key backed up in: GitHub Secrets + password manager (2 locations minimum)
- Key loss = all installed copies can never auto-update again -- treat as critical
- Repository: SwirlClinic/ModToggler (public)
- Update endpoint: `https://github.com/SwirlClinic/ModToggler/releases/latest/download/latest.json`
- Tag format: `v1.0.0` (semver with v prefix, matches tauri-action default)
- Public repo = no auth headers needed in updater config
- Keep `"targets": "all"` -- build NSIS + MSI + others
- Only NSIS supports auto-update, but MSI available for users who prefer it

### Claude's Discretion
- Exact plugin registration order in lib.rs
- Whether to add `tauri-plugin-process` now (needed for relaunch in Phase 7) or defer
- NSIS install mode configuration details

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INFRA-01 | App uses Ed25519 signing keypair for update artifact verification | Key generation via `cargo tauri signer generate`, Minisign Ed25519 format, passwordless option supported |
| INFRA-02 | Tauri updater plugin is registered and configured with public key and GitHub Releases endpoint | `tauri-plugin-updater` Cargo + npm deps, plugin registration in lib.rs, `plugins.updater` config block in tauri.conf.json |
| INFRA-03 | Updater capabilities/permissions are granted in the app's capability config | `updater:default` and `process:default` permissions in capabilities/default.json |
| INFRA-04 | `createUpdaterArtifacts` is enabled in bundle config to generate `.sig` files | `bundle.createUpdaterArtifacts: true` in tauri.conf.json, requires TAURI_SIGNING_PRIVATE_KEY env var at build time |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-updater | 2 | In-app update verification and download | Official Tauri plugin, only supported updater mechanism |
| tauri-plugin-process | 2 | App exit and relaunch after update | Official Tauri plugin, required for post-update restart |
| @tauri-apps/plugin-updater | 2 | JS guest bindings for updater (Phase 7) | Official JS bindings, paired with Rust crate |
| @tauri-apps/plugin-process | 2 | JS guest bindings for process (Phase 7) | Official JS bindings, paired with Rust crate |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/cli (existing) | 2 | `cargo tauri signer generate` command | Key generation only |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Tauri built-in updater | CrabNebula Cloud | Hosted service, overkill for single-dev public repo |
| GitHub Releases endpoint | Custom update server | Unnecessary complexity for public repo |

**Installation:**
```bash
# Rust dependencies (in src-tauri/)
cargo add tauri-plugin-updater@2 tauri-plugin-process@2

# JS guest bindings (in project root)
npm install @tauri-apps/plugin-updater @tauri-apps/plugin-process
```

## Architecture Patterns

### Configuration Changes Map
```
src-tauri/
  Cargo.toml              # + tauri-plugin-updater, tauri-plugin-process
  src/lib.rs              # + .plugin() registrations in builder chain
  tauri.conf.json         # + plugins.updater block, + bundle.createUpdaterArtifacts
  capabilities/
    default.json          # + "updater:default", "process:default"
package.json              # + @tauri-apps/plugin-updater, @tauri-apps/plugin-process
```

### Pattern 1: Plugin Registration in lib.rs

**What:** Chain `.plugin()` calls on `tauri::Builder::default()` before `.invoke_handler()`
**When to use:** Adding any Tauri plugin

The existing code registers plugins at lines 48-54. The updater plugin uses a builder pattern; the process plugin uses a simple `init()`.

```rust
// Source: https://v2.tauri.app/plugin/updater/
tauri::Builder::default()
    .plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:modtoggler.db", db::migrations::get_migrations())
            .build(),
    )
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_updater::Builder::new().build())  // NEW
    .plugin(tauri_plugin_process::init())                   // NEW
    .manage(state::AppState::default())
    // ... rest unchanged
```

**Registration order recommendation:** Add updater and process after existing plugins, before `.manage()`. Order among plugins does not matter for correctness, but grouping infrastructure plugins together is conventional.

### Pattern 2: tauri.conf.json Updater Config

**What:** Add `plugins.updater` block with public key and endpoint
**Source:** https://v2.tauri.app/plugin/updater/

```json
{
  "plugins": {
    "updater": {
      "pubkey": "PASTE_PUBLIC_KEY_HERE",
      "endpoints": [
        "https://github.com/SwirlClinic/ModToggler/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  },
  "bundle": {
    "createUpdaterArtifacts": true
  }
}
```

Key points:
- `pubkey`: The full public key string output by `cargo tauri signer generate` (starts with `dW5...` or similar base64)
- `endpoints`: Array of URLs. Tauri tries them in order. Single endpoint is fine for GitHub Releases
- `windows.installMode`: `"passive"` shows progress bar only, no wizard. Matches user decision
- `createUpdaterArtifacts`: Must be `true` (not `"v1Compatible"`) since this is a new app, not a v1 migration

### Pattern 3: Key Generation Script

**What:** Generate Ed25519 signing keypair using Tauri CLI
**Source:** https://v2.tauri.app/plugin/updater/

```bash
# Generate keypair with no password (user decision: passwordless)
# -w writes the private key to file; public key is printed to stdout
npx tauri signer generate -w ~/.tauri/modtoggler.key
```

When prompted for password, press Enter twice (empty password).

Output:
- Private key file: `~/.tauri/modtoggler.key`
- Public key: printed to stdout (copy into tauri.conf.json `pubkey` field)
- Public key file: `~/.tauri/modtoggler.key.pub`

### Pattern 4: Build-Time Signing Environment

**What:** Set env vars so `cargo tauri build` signs artifacts
**Source:** https://v2.tauri.app/plugin/updater/

```bash
# Option A: Key content directly (recommended for CI)
export TAURI_SIGNING_PRIVATE_KEY="content of ~/.tauri/modtoggler.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""

# Option B: Key file path (for local builds)
export TAURI_SIGNING_PRIVATE_KEY="~/.tauri/modtoggler.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```

When `TAURI_SIGNING_PRIVATE_KEY` is set and `createUpdaterArtifacts` is `true`, `cargo tauri build` produces `.sig` files alongside each installer.

### Anti-Patterns to Avoid
- **Committing the private key to git:** The private key must NEVER be in the repository. Only the public key goes in tauri.conf.json
- **Using `"v1Compatible"` for createUpdaterArtifacts:** This is only for apps migrating from Tauri v1. New apps use `true`
- **Setting perMachine NSIS install:** Per Tauri issue #7184, perMachine install mode breaks auto-update. Use currentUser (default)
- **Forgetting TAURI_SIGNING_PRIVATE_KEY_PASSWORD:** Even when passwordless, the env var must be set to empty string `""` or the build may prompt interactively

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Update signing | Custom signing scheme | Tauri's built-in Minisign/Ed25519 | Cryptographic signing has zero room for error; Tauri handles it end-to-end |
| Update manifest | Manual latest.json creation | `tauri-action` GitHub Action (Phase 6) | The manifest format has platform-specific keys and signature embedding |
| Key generation | Manual openssl/minisign | `cargo tauri signer generate` | Produces keys in exact format Tauri expects |
| Update download/verify | Custom HTTP + verify logic | `tauri-plugin-updater` APIs | Handles signature verification, partial downloads, atomic install |

**Key insight:** The entire update pipeline is designed as an integrated system. Every piece (key format, signature format, manifest format, verification logic) must match exactly. Using any non-Tauri tool for any step risks subtle incompatibilities.

## Common Pitfalls

### Pitfall 1: Key Loss Bricks All Installed Copies
**What goes wrong:** If the signing private key is lost, no future update can be signed with the matching key. All users with the app installed will never receive auto-updates again.
**Why it happens:** Single point of failure in the signing chain.
**How to avoid:** Back up the private key in 2+ locations (GitHub Secrets + password manager) BEFORE any other work. Verify backup by restoring from each location.
**Warning signs:** Key only exists in one place; key stored only on local machine.

### Pitfall 2: Version Triple Desync
**What goes wrong:** Update detection fails silently if `tauri.conf.json`, `Cargo.toml`, and `package.json` versions don't match.
**Why it happens:** Three files declare version independently.
**How to avoid:** Keep all three at `0.1.0` during this phase. Version bumping is a Phase 6/CI concern.
**Warning signs:** `cargo tauri build` warnings about version mismatch.

### Pitfall 3: Missing TAURI_SIGNING_PRIVATE_KEY_PASSWORD
**What goes wrong:** Build hangs waiting for password input, or fails with cryptic error.
**Why it happens:** Even with a passwordless key, Tauri checks for the env var.
**How to avoid:** Always set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""` (empty string) alongside the private key.
**Warning signs:** Build seems to hang during signing step.

### Pitfall 4: perMachine NSIS Install Mode
**What goes wrong:** Auto-update fails because the updater cannot write to Program Files without admin elevation.
**Why it happens:** perMachine installs to `C:\Program Files\` which requires admin rights.
**How to avoid:** Use `currentUser` install mode (NSIS default). Do NOT set `bundle.windows.nsis.installMode` to `"perMachine"`.
**Warning signs:** Update downloads but fails to install; UAC prompt during update.

### Pitfall 5: Public Key Format
**What goes wrong:** Updater fails to verify signatures if the public key is incorrectly copied.
**Why it happens:** The public key is a single base64 string. Accidental truncation or line breaks break it.
**How to avoid:** Copy the exact output from `cargo tauri signer generate` or from the `.pub` file. No line breaks.
**Warning signs:** "signature verification failed" errors when testing updates.

## Code Examples

### Complete lib.rs Plugin Registration
```rust
// Source: existing lib.rs + https://v2.tauri.app/plugin/updater/
tauri::Builder::default()
    .plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:modtoggler.db", db::migrations::get_migrations())
            .build(),
    )
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .manage(state::AppState::default())
    .invoke_handler(builder.invoke_handler())
    .setup(|app| {
        // ... existing setup code unchanged
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Complete tauri.conf.json Changes
```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "createUpdaterArtifacts": true,
    "icon": ["...existing..."]
  },
  "plugins": {
    "updater": {
      "pubkey": "GENERATED_PUBLIC_KEY_HERE",
      "endpoints": [
        "https://github.com/SwirlClinic/ModToggler/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

### Complete capabilities/default.json
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
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

### Cargo.toml Additions
```toml
[dependencies]
# ... existing deps ...
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 built-in updater | Tauri v2 plugin-based updater | Tauri 2.0 (Oct 2024) | Updater is now a separate plugin, not core |
| `createUpdaterArtifacts: "v1Compatible"` | `createUpdaterArtifacts: true` | Tauri 2.0 | New apps should use `true`; `"v1Compatible"` only for migration |
| RSA signatures (v1) | Ed25519/Minisign (v2) | Tauri 2.0 | Faster, smaller signatures |

**Deprecated/outdated:**
- Tauri v1 updater configuration (different config format, built into core)
- `allowlist` permissions system (replaced by capabilities in v2)

## Open Questions

1. **Exact latest tauri-plugin-updater version**
   - What we know: Version 2.x is correct. Docs.rs shows 2.9.0, npm shows 2.10.0
   - What's unclear: Exact latest patch version at time of implementation
   - Recommendation: Use `"2"` (semver range) in Cargo.toml -- Cargo will resolve latest compatible. Same approach as existing plugins in the project

2. **Key file location convention on Windows**
   - What we know: Docs show `~/.tauri/myapp.key` (Unix convention)
   - What's unclear: Whether `$HOME/.tauri/` works on Windows or needs `$USERPROFILE`
   - Recommendation: Use `$HOME/.tauri/modtoggler.key` -- Git Bash and PowerShell both resolve `$HOME`

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.x (frontend), Cargo test (Rust) |
| Config file | No vitest.config.* detected; no Rust test files for this domain |
| Quick run command | `npm run tauri build 2>&1 | head -50` (verify build completes) |
| Full suite command | `cargo tauri build` (produces signed artifacts) |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INFRA-01 | Ed25519 keypair exists | manual | Verify `~/.tauri/modtoggler.key` and `.key.pub` exist | N/A |
| INFRA-02 | Updater plugin registered and configured | smoke | `cargo check --manifest-path src-tauri/Cargo.toml` (compiles with plugin) | N/A |
| INFRA-03 | Capabilities granted | smoke | `cargo check --manifest-path src-tauri/Cargo.toml` (capabilities validated at compile) | N/A |
| INFRA-04 | `.sig` files produced | smoke | `cargo tauri build` then check for `.sig` files in target/release/bundle/ | N/A |

### Sampling Rate
- **Per task commit:** `cargo check --manifest-path src-tauri/Cargo.toml` (fast compile check)
- **Per wave merge:** `cargo tauri build` with TAURI_SIGNING_PRIVATE_KEY set (full build + signing)
- **Phase gate:** Full build produces `.nsis.zip` and `.nsis.zip.sig` files

### Wave 0 Gaps
- [ ] Key generation must happen before any build verification can succeed
- [ ] `TAURI_SIGNING_PRIVATE_KEY` env var must be set for signed build verification
- [ ] No automated test files needed -- this phase is verified by successful build output

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Updater Plugin docs](https://v2.tauri.app/plugin/updater/) - Complete setup guide, config schema, key generation
- [Tauri v2 Process Plugin docs](https://v2.tauri.app/plugin/process/) - Setup and permissions
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/) - Config schema for bundle and plugins

### Secondary (MEDIUM confidence)
- [Tauri v2 Auto-Updater Guide (thatgurjot.com)](https://thatgurjot.com/til/tauri-auto-updater/) - Practical walkthrough verified against official docs
- [CrabNebula Tauri v2 Auto-Update Guide](https://docs.crabnebula.dev/cloud/guides/auto-updates-tauri/) - Additional setup reference
- [tauri-plugin-updater on crates.io](https://crates.io/crates/tauri-plugin-updater) - Version info

### Tertiary (LOW confidence)
- [Tauri issue #7184](https://github.com/tauri-apps/tauri/issues/7184) - perMachine auto-update failure (referenced in STATE.md decisions)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tauri plugins, well-documented, only viable path
- Architecture: HIGH - Configuration changes are prescriptive, verified against official docs and existing project structure
- Pitfalls: HIGH - Well-known issues documented in Tauri issues and multiple community guides

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable -- Tauri v2 plugin API is stable)
