# Phase 5: Updater Foundation - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Configure the app to verify and receive signed updates from GitHub Releases. This includes generating an Ed25519 signing keypair, adding the Tauri updater and process plugins, configuring tauri.conf.json with the public key and endpoint, enabling updater capabilities, and ensuring `createUpdaterArtifacts` is set so builds produce `.sig` files.

</domain>

<decisions>
## Implementation Decisions

### Key Management
- Passwordless Ed25519 signing key (no TAURI_SIGNING_PRIVATE_KEY_PASSWORD needed)
- Key generation scripted as a helper task (not manual docs-only)
- Key backed up in: GitHub Secrets + password manager (2 locations minimum)
- Key loss = all installed copies can never auto-update again — treat as critical

### GitHub Repo Setup
- Repository: SwirlClinic/ModToggler (public)
- Update endpoint: `https://github.com/SwirlClinic/ModToggler/releases/latest/download/latest.json`
- Tag format: `v1.0.0` (semver with v prefix, matches tauri-action default)
- Public repo = no auth headers needed in updater config

### Bundle Targets
- Keep `"targets": "all"` — build NSIS + MSI + others
- Only NSIS supports auto-update, but MSI available for users who prefer it

### Claude's Discretion
- Exact plugin registration order in lib.rs
- Whether to add `tauri-plugin-process` now (needed for relaunch in Phase 7) or defer
- NSIS install mode configuration details

</decisions>

<specifics>
## Specific Ideas

No specific requirements — standard Tauri updater plugin setup following official docs.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None directly reusable — this phase adds new infrastructure

### Established Patterns
- Plugin registration: `lib.rs:47-54` — chain `.plugin()` calls on `tauri::Builder::default()`
- Capabilities: `capabilities/default.json` — flat permission array, add `updater:default` and `process:relaunch`
- Version triple: `tauri.conf.json` (0.1.0), `Cargo.toml` (0.1.0), `package.json` (0.1.0) — all in sync

### Integration Points
- `src-tauri/Cargo.toml` — add `tauri-plugin-updater` and `tauri-plugin-process` dependencies
- `src-tauri/src/lib.rs` — register plugins in the builder chain
- `src-tauri/tauri.conf.json` — add `plugins.updater` config block and `bundle.createUpdaterArtifacts`
- `src-tauri/capabilities/default.json` — add updater and process permissions
- `package.json` — add `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` JS bindings

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-updater-foundation*
*Context gathered: 2026-03-08*
