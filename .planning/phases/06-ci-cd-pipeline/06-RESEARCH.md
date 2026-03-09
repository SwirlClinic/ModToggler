# Phase 6: CI/CD Pipeline - Research

**Researched:** 2026-03-08
**Domain:** GitHub Actions / Tauri Release Automation
**Confidence:** HIGH

## Summary

This phase creates a GitHub Actions workflow that builds, signs, and publishes ModToggler releases when a `v*` tag is pushed. The ecosystem has a clear "blessed" tool for this: `tauri-apps/tauri-action`, which handles the Tauri build, artifact signing, updater JSON generation, and GitHub Release creation in a single action step.

The workflow is straightforward for a single-platform (Windows-only) build: one runner, one job, no matrix. The main complexity is in the version-patching step -- the CI must extract the version from the pushed tag and patch three files (`tauri.conf.json`, `Cargo.toml`, `package.json`) before calling tauri-action, because tauri-action reads the version from `tauri.conf.json` to populate the `__VERSION__` placeholder in `tagName`.

**Primary recommendation:** Use `tauri-apps/tauri-action@v0` (the only available major version) with `includeUpdaterJson: true`, `updaterJsonPreferNsis: true`, and `generateReleaseNotes: true`. Patch all three version files from the git tag before the build step. Cache Rust with `swatinem/rust-cache@v2` and npm with `actions/setup-node` cache.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Manual tag push triggers release: `git tag v1.0.0 && git push --tags`
- Tag push on any branch triggers the build (no branch restriction)
- CI extracts version from the tag and patches all 3 version files (tauri.conf.json, Cargo.toml, package.json) before building
- CI commits the version bump back to the repo after patching, keeping repo in sync with releases
- Windows only (single `windows-latest` runner) -- no cross-platform matrix
- Keep `targets: all` -- produce everything Tauri can build (NSIS, MSI, etc.), consistent with local builds
- Releases published immediately (no draft step) -- updater needs published releases to find latest.json
- GitHub auto-generated release notes from commits/PRs since last release
- No extra artifacts beyond what tauri-action produces (installers, .sig files, latest.json)
- Pin to `@v1` (current stable major for Tauri v2 apps)

### Claude's Discretion
- Exact workflow YAML structure and job naming
- Caching strategy for Rust/npm dependencies
- Node.js version selection
- How to implement the version-patch-and-commit step (sed, jq, or a script)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CICD-01 | GitHub Actions workflow builds Windows NSIS installer on version tag push | tauri-action@v0 with `on: push: tags: ['v*']` trigger builds NSIS installer on windows-latest runner |
| CICD-02 | CI signs artifacts using the stored signing key secret | tauri-action reads `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from environment; secrets already configured in GitHub |
| CICD-03 | CI uploads installer + latest.json to a GitHub Release automatically | tauri-action with `includeUpdaterJson: true` generates and uploads latest.json alongside .exe/.sig artifacts to a published GitHub Release |

</phase_requirements>

## Standard Stack

### Core
| Library/Tool | Version | Purpose | Why Standard |
|---|---|---|---|
| tauri-apps/tauri-action | @v0 (latest: v0.6.1) | Build Tauri app, create release, upload artifacts + latest.json | Official Tauri GitHub Action; only maintained option |
| actions/checkout | @v4 | Check out repository code | Standard GitHub Actions checkout |
| actions/setup-node | @v4 | Install Node.js with npm cache | Standard Node.js setup |
| dtolnay/rust-toolchain | @stable | Install latest stable Rust | Community standard for Rust in CI |
| swatinem/rust-cache | @v2 | Cache Rust target/ directory | Reduces Rust build time from 10-15min to 2-3min on cache hit |

### Key tauri-action Inputs
| Input | Value | Purpose |
|---|---|---|
| `tagName` | `v__VERSION__` | Creates release tagged with version from tauri.conf.json |
| `releaseName` | `ModToggler v__VERSION__` | Human-readable release title |
| `releaseDraft` | `false` | Publish immediately (updater needs published release) |
| `prerelease` | `false` | Mark as stable release |
| `includeUpdaterJson` | `true` (default) | Generates latest.json for the updater plugin |
| `updaterJsonPreferNsis` | `true` | Use NSIS (.exe) not WiX (.msi) URLs in latest.json |
| `generateReleaseNotes` | `true` | Auto-generate release notes from commits/PRs |

### CRITICAL: tauri-action Version Correction

The CONTEXT.md locks the decision to "Pin to @v1". However, **there is no `v1` tag** on the tauri-apps/tauri-action repository. The available tags are:

- `v0` (floating major tag, currently points to v0.6.1)
- `v0.6` (floating minor tag)
- `v0.6.1` (specific release, Jan 3 2026)

The README examples use `@v1` in some places, but the actual releases page and tags page show only v0.x releases. The correct pin is `@v0`. This is the only available major version and is actively maintained for Tauri v2. The planner should use `@v0` and note this correction from the CONTEXT.md decision.

**Confidence: HIGH** -- verified directly from GitHub tags page and releases page.

## Architecture Patterns

### Recommended Workflow Structure
```
.github/
  workflows/
    release.yml          # Single workflow file
```

### Pattern 1: Tag-Triggered Release Workflow
**What:** A single workflow triggered by `v*` tag pushes that patches versions, builds, signs, and publishes.
**When to use:** Every release.

```yaml
# Source: Tauri v2 official docs + community patterns
name: Release

on:
  push:
    tags:
      - 'v*'

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: true

jobs:
  release:
    runs-on: windows-latest
    permissions:
      contents: write

    steps:
      - uses: actions/checkout@v4

      - name: Extract version from tag
        shell: bash
        run: |
          TAG=${{ github.ref_name }}
          VERSION=${TAG#v}
          echo "VERSION=$VERSION" >> $GITHUB_ENV

      # Version patch step (see Pattern 2)

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Install frontend dependencies
        run: npm ci

      - name: Build and release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'ModToggler v__VERSION__'
          releaseDraft: false
          prerelease: false
          updaterJsonPreferNsis: true
          generateReleaseNotes: true
```

### Pattern 2: Version Patching from Tag (Bash on Windows runner)
**What:** Extract version from git tag, patch all three version files, commit back.
**Why:** tauri-action reads version from `tauri.conf.json` to replace `__VERSION__` placeholder. If the tag is `v1.0.0` but `tauri.conf.json` says `0.1.0`, the release is created for the wrong version.

The `windows-latest` runner supports bash via Git Bash. Use `shell: bash` for string manipulation.

```yaml
# Using node -e for cross-platform JSON manipulation (jq not available on windows-latest)
- name: Patch version files
  shell: bash
  run: |
    # Patch package.json
    node -e "
      const pkg = require('./package.json');
      pkg.version = '${{ env.VERSION }}';
      require('fs').writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
    "

    # Patch tauri.conf.json
    node -e "
      const conf = require('./src-tauri/tauri.conf.json');
      conf.version = '${{ env.VERSION }}';
      require('fs').writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2) + '\n');
    "

    # Patch Cargo.toml (sed works in Git Bash on Windows)
    sed -i "s/^version = \".*\"/version = \"${{ env.VERSION }}\"/" src-tauri/Cargo.toml
```

For the commit-back step:

```yaml
- name: Commit version bump
  shell: bash
  run: |
    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
    git diff --staged --quiet || git commit -m "chore: bump version to ${{ env.VERSION }}"
    git push origin HEAD:${{ github.event.repository.default_branch }}
```

**Important:** The commit-back step pushes to the default branch. Since the workflow triggers on tag push (not branch push), this commit will NOT re-trigger the workflow. The `git diff --staged --quiet ||` guard prevents empty commits when versions already match.

### Pattern 3: Concurrency Control
**What:** Cancel in-progress releases if a new tag is pushed quickly.
**Why:** Prevents duplicate release artifacts and wasted runner minutes.

```yaml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: true
```

### Anti-Patterns to Avoid
- **Using `releaseDraft: true` with the updater:** The updater endpoint (`/releases/latest/download/latest.json`) only finds published releases. Draft releases are invisible to the updater.
- **Patching versions AFTER tauri-action:** The action reads tauri.conf.json at build time. Patching after means the built artifacts have the old version baked in.
- **Using `jq` on Windows runners:** `jq` is not pre-installed on `windows-latest`. Use `node -e` instead since Node.js is always available.
- **Pinning to `@v1`:** This tag does not exist. Use `@v0`.
- **Using `sed -i` with Cargo.toml carelessly:** The `sed` pattern must only match the top-level `[package]` version, not dependency version lines. The first `version = "..."` in Cargo.toml is the package version when the file starts with `[package]`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Tauri build + artifact upload | Custom `tauri build` + `gh release create` | `tauri-apps/tauri-action@v0` | Handles build, artifact naming, release creation, asset upload, latest.json generation, and .sig file upload in one step |
| latest.json generation | Script to manually construct updater manifest | `includeUpdaterJson: true` on tauri-action | The JSON format includes platform keys, signatures, and URLs that must match exactly what the updater plugin expects |
| Release notes | Manual changelog writing | `generateReleaseNotes: true` | GitHub API auto-generates notes from merged PRs and commits since last release |
| Rust caching | Manual `actions/cache` with Rust target dir | `swatinem/rust-cache@v2` | Handles cache key computation, selective caching, and cache invalidation automatically |
| JSON file patching on Windows | `jq` commands | `node -e` with `require()` / `JSON.parse` | `jq` is not available on windows-latest; Node.js is always present |

**Key insight:** tauri-action is doing A LOT under the hood -- building the app, generating NSIS/MSI installers, creating .sig signature files, generating latest.json with correct platform keys and URLs, creating/finding the GitHub Release, and uploading all artifacts. Reproducing this manually would be error-prone and fragile.

## Common Pitfalls

### Pitfall 1: Version Mismatch Between Tag and tauri.conf.json
**What goes wrong:** You push tag `v1.2.0` but tauri.conf.json still says `"version": "0.1.0"`. tauri-action creates a release for `v0.1.0`, not `v1.2.0`. The updater never finds the update because the version in latest.json doesn't match.
**Why it happens:** The `__VERSION__` placeholder reads from tauri.conf.json, not from the git tag.
**How to avoid:** Patch all three version files from the tag BEFORE the tauri-action step.
**Warning signs:** Release title shows wrong version; latest.json has unexpected version number.

### Pitfall 2: Cargo.toml sed Matches Wrong Line
**What goes wrong:** `sed` replaces a dependency version instead of the package version.
**Why it happens:** Multiple `version = "..."` lines exist in Cargo.toml (one for the package, many for dependencies).
**How to avoid:** Ensure the sed pattern targets only the first occurrence, or use a more specific pattern. Since `[package]` is the first section in the project's Cargo.toml and `version` is the second line under it, a simple `sed -i '0,/^version = ".*"/s//version = "NEW_VERSION"/'` (first-match-only) works. Alternatively, use a dedicated tool or `node -e` to parse TOML.
**Warning signs:** Build errors about invalid dependency versions; `cargo build` fails.

### Pitfall 3: Permissions Error Creating Release
**What goes wrong:** Workflow fails with "Resource not accessible by integration."
**Why it happens:** The default GITHUB_TOKEN doesn't have write permissions.
**How to avoid:** Add `permissions: contents: write` to the job. Also verify in repo Settings > Actions > General > Workflow permissions that "Read and write permissions" is selected.
**Warning signs:** 403 errors in the tauri-action step.

### Pitfall 4: Draft Release Breaks Updater
**What goes wrong:** The app doesn't find updates even though the release exists.
**Why it happens:** The updater endpoint uses `/releases/latest/download/latest.json`. GitHub's "latest release" API only returns published (non-draft, non-prerelease) releases.
**How to avoid:** Set `releaseDraft: false` (already decided in CONTEXT.md).
**Warning signs:** Updater reports "no update available" despite a new release existing.

### Pitfall 5: Commit-Back Triggers Infinite Loop
**What goes wrong:** Pushing the version bump commit back to the repo re-triggers the workflow.
**Why it happens:** If the workflow triggers on branch push events.
**How to avoid:** This workflow only triggers on tag push (`tags: ['v*']`), not branch push. A plain `git push` of a commit to a branch will NOT trigger tag-filtered workflows. No loop risk with the current design.
**Warning signs:** Multiple workflow runs spawning from a single tag push.

### Pitfall 6: Signing Key Not Available
**What goes wrong:** Build succeeds but no `.sig` files are generated, or the build fails.
**Why it happens:** `TAURI_SIGNING_PRIVATE_KEY` secret not set or empty in the repository.
**How to avoid:** Secrets are already configured (confirmed in CONTEXT.md). The env block passes them to tauri-action.
**Warning signs:** Missing .sig files in the release assets; updater signature verification fails.

## Code Examples

### Complete Workflow (Recommended)

```yaml
# Source: Assembled from official Tauri docs, tauri-action README, and community patterns
# File: .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: true

jobs:
  release:
    runs-on: windows-latest
    permissions:
      contents: write

    steps:
      - uses: actions/checkout@v4

      - name: Extract version from tag
        shell: bash
        run: |
          VERSION=${GITHUB_REF_NAME#v}
          echo "VERSION=$VERSION" >> $GITHUB_ENV
          echo "Releasing version: $VERSION"

      - name: Patch version in project files
        shell: bash
        run: |
          node -e "
            const fs = require('fs');
            const v = '${{ env.VERSION }}';

            // package.json
            const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            pkg.version = v;
            fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');

            // tauri.conf.json
            const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
            conf.version = v;
            fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2) + '\n');
          "

          # Cargo.toml - replace first version line only (package version)
          sed -i "0,/^version = \".*\"/{s/^version = \".*\"/version = \"${{ env.VERSION }}\"/}" src-tauri/Cargo.toml

      - name: Commit version bump
        shell: bash
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
          if ! git diff --staged --quiet; then
            git commit -m "chore: bump version to ${{ env.VERSION }}"
            git push origin HEAD:${{ github.event.repository.default_branch }}
          fi

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Install frontend dependencies
        run: npm ci

      - name: Build and release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'ModToggler v__VERSION__'
          releaseDraft: false
          prerelease: false
          updaterJsonPreferNsis: true
          generateReleaseNotes: true
```

### Expected Release Assets
After a successful build, the GitHub Release should contain:
- `ModToggler_X.Y.Z_x64-setup.exe` -- NSIS installer
- `ModToggler_X.Y.Z_x64-setup.nsis.zip` -- Zipped NSIS installer (for updater)
- `ModToggler_X.Y.Z_x64-setup.nsis.zip.sig` -- Signature for updater verification
- `ModToggler_X.Y.Z_x64_en-US.msi` -- MSI installer (targets: all)
- `ModToggler_X.Y.Z_x64_en-US.msi.zip` -- Zipped MSI (for updater)
- `ModToggler_X.Y.Z_x64_en-US.msi.zip.sig` -- MSI signature
- `latest.json` -- Updater manifest with version, URLs, and signatures

### latest.json Format (Auto-Generated)
```json
{
  "version": "1.0.0",
  "notes": "Auto-generated release notes",
  "pub_date": "2026-03-08T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "base64-encoded-ed25519-signature",
      "url": "https://github.com/SwirlClinic/ModToggler/releases/download/v1.0.0/ModToggler_1.0.0_x64-setup.nsis.zip"
    }
  }
}
```

When `updaterJsonPreferNsis: true`, the `windows-x86_64` platform entry points to the NSIS zip, not the MSI zip. This is correct because the app uses NSIS install mode (`"installMode": "passive"`).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| tauri-action@v0.4.x (Tauri v1) | tauri-action@v0.6.x (Tauri v2) | Late 2024 | v0.6+ supports Tauri v2 updater artifact format |
| Manual latest.json creation | `includeUpdaterJson: true` (default) | tauri-action v0.5+ | Auto-generates updater manifest with correct platform keys |
| WiX (.msi) default for updater | `updaterJsonPreferNsis: true` option | tauri-action v0.5+ | NSIS preferred for new Tauri v2 apps |
| Separate release notes action | `generateReleaseNotes: true` input | tauri-action v0.5+ | Passes through to GitHub API |
| pinned Node 18 | `node-version: 'lts/*'` | 2025 | tauri-action v0.6.0 uses Node 24 internally; runner's Node version is for npm/frontend build only |

**Deprecated/outdated:**
- `includeUpdaterJson` name: In some older docs referred to as `includeUpdaterJson`, now confirmed as the correct name (default: `true`)
- `@v1` tag: Does NOT exist despite some documentation referencing it. Use `@v0`.

## Open Questions

1. **Cargo.lock regeneration after Cargo.toml version patch**
   - What we know: Changing `version` in Cargo.toml may cause Cargo.lock to change. The blog post mentions running `cargo generate-lockfile` after patching.
   - What's unclear: Whether tauri-action handles this automatically during its build step (likely yes, since `cargo build` updates Cargo.lock).
   - Recommendation: Let the build step handle it. If Cargo.lock is committed, the version patch commit should include it. Since `tauri build` runs `cargo build`, it will update Cargo.lock as needed. Don't explicitly add a `cargo generate-lockfile` step -- if it causes issues, the build step will surface it.

2. **Exact artifact naming with `targets: all`**
   - What we know: With `targets: all`, both NSIS and MSI are produced. `updaterJsonPreferNsis: true` controls which goes into latest.json.
   - What's unclear: The exact file names depend on the product name and version in tauri.conf.json.
   - Recommendation: The naming is deterministic based on `productName` and `version` in tauri.conf.json. No action needed; verify after first release.

3. **Commit-back and detached HEAD on tag checkout**
   - What we know: `actions/checkout` on a tag push checks out the tag commit, which may result in a detached HEAD state.
   - What's unclear: Whether `git push origin HEAD:main` works correctly from detached HEAD.
   - Recommendation: This should work fine -- `HEAD:main` pushes the current commit to the `main` branch regardless of HEAD state. If issues arise, add `git checkout -b temp-release` before committing.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | GitHub Actions (workflow validation) |
| Config file | `.github/workflows/release.yml` |
| Quick run command | `act -n` (dry run, requires nektos/act) or manual tag push |
| Full suite command | `git tag v0.0.1-test && git push --tags` (manual end-to-end) |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CICD-01 | Workflow builds Windows NSIS installer on tag push | e2e / manual-only | Push a `v*` tag and verify workflow runs | N/A -- workflow file is the deliverable |
| CICD-02 | CI signs artifacts using stored signing key | e2e / manual-only | Check release assets for `.sig` files after workflow run | N/A -- verified by presence of .sig in release |
| CICD-03 | CI uploads installer + latest.json to GitHub Release | e2e / manual-only | Check release assets for .exe and latest.json after workflow run | N/A -- verified by presence of assets in release |

### Sampling Rate
- **Per task commit:** YAML lint / syntax validation (`actionlint` if available)
- **Per wave merge:** Manual test tag push to verify end-to-end
- **Phase gate:** Push a test tag, verify GitHub Release contains all expected assets

### Wave 0 Gaps
- [ ] `.github/workflows/release.yml` -- the workflow file itself (the entire deliverable)
- [ ] `.github/workflows/` directory -- needs to be created

*(Note: This phase is infrastructure-as-code. The "test" IS pushing a tag and verifying the release. There are no unit tests to write -- validation is end-to-end by nature.)*

## Sources

### Primary (HIGH confidence)
- [tauri-apps/tauri-action tags page](https://github.com/tauri-apps/tauri-action/tags) -- confirmed v0.6.1 latest, NO v1 tag exists
- [tauri-apps/tauri-action releases](https://github.com/tauri-apps/tauri-action/releases) -- v0.6.1 released Jan 3 2026
- [tauri-apps/tauri-action action.yml (v0 tag)](https://raw.githubusercontent.com/tauri-apps/tauri-action/refs/tags/v0/action.yml) -- complete input definitions
- [Tauri v2 official docs: GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github/) -- official workflow example
- [Tauri v2 updater docs](https://v2.tauri.app/plugin/updater/) -- updater endpoint and latest.json format

### Secondary (MEDIUM confidence)
- [Ship Your Tauri v2 App Like a Pro (DEV Community)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- complete real-world workflow with caching, signing, version patching
- [swatinem/rust-cache](https://github.com/Swatinem/rust-cache) -- Rust caching configuration for Tauri projects
- [GitHub Docs: Automatically generated release notes](https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes) -- generateReleaseNotes behavior

### Tertiary (LOW confidence)
- None -- all findings verified with primary or secondary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- tauri-action is the only official/maintained option; inputs verified from action.yml
- Architecture: HIGH -- single workflow, single runner, well-documented pattern
- Pitfalls: HIGH -- version mismatch issue verified by multiple sources; permissions issue well-known
- Version correction (@v0 not @v1): HIGH -- directly verified from GitHub tags page

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable domain; tauri-action releases are infrequent)
