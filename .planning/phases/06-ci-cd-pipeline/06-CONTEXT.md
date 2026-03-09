# Phase 6: CI/CD Pipeline - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

GitHub Actions workflow that builds, signs, and publishes releases when a version tag is pushed. Pushing a `v*` tag triggers the workflow, which patches version numbers from the tag, builds the app with signing, and creates a published GitHub Release with installer artifacts and `latest.json` manifest.

</domain>

<decisions>
## Implementation Decisions

### Workflow Trigger & Versioning
- Manual tag push triggers release: `git tag v1.0.0 && git push --tags`
- Tag push on any branch triggers the build (no branch restriction)
- CI extracts version from the tag and patches all 3 version files (tauri.conf.json, Cargo.toml, package.json) before building
- CI commits the version bump back to the repo after patching, keeping repo in sync with releases

### Build Matrix & Targets
- Windows only (single `windows-latest` runner) — no cross-platform matrix
- Keep `targets: all` — produce everything Tauri can build (NSIS, MSI, etc.), consistent with local builds
- Latest stable Rust toolchain (no pinned version)

### Release Structure
- Releases published immediately (no draft step) — updater needs published releases to find latest.json
- GitHub auto-generated release notes from commits/PRs since last release
- No extra artifacts beyond what tauri-action produces (installers, .sig files, latest.json)

### tauri-action Version
- Pin to `@v1` (current stable major for Tauri v2 apps)
- Resolves STATE.md concern about @v0 vs @v1 ambiguity

### Claude's Discretion
- Exact workflow YAML structure and job naming
- Caching strategy for Rust/npm dependencies
- Node.js version selection
- How to implement the version-patch-and-commit step (sed, jq, or a script)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — standard tauri-action workflow following official docs.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — no existing GitHub Actions workflows

### Established Patterns
- Version triple: `tauri.conf.json` (0.1.0), `Cargo.toml` (0.1.0), `package.json` (0.1.0) — all must stay in sync
- Signing secrets already in GitHub: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `createUpdaterArtifacts: true` already set in tauri.conf.json
- Updater endpoint: `https://github.com/SwirlClinic/ModToggler/releases/latest/download/latest.json`

### Integration Points
- `.github/workflows/` — new workflow file
- GitHub Secrets — `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (already configured)
- `tauri.conf.json`, `Cargo.toml`, `package.json` — version fields patched by CI

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-ci-cd-pipeline*
*Context gathered: 2026-03-09*
