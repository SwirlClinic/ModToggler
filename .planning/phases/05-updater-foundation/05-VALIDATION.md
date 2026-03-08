---
phase: 5
slug: updater-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo check (Rust compile verification) |
| **Config file** | src-tauri/Cargo.toml |
| **Quick run command** | `cargo check --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `cargo tauri build` (with TAURI_SIGNING_PRIVATE_KEY set) |
| **Estimated runtime** | ~15s (check), ~120s (full build) |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo tauri build` with signing key set
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds (check), 120 seconds (build)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | INFRA-01 | manual | Verify `~/.tauri/modtoggler.key` and `.key.pub` exist | N/A | pending |
| 05-01-02 | 01 | 1 | INFRA-02 | smoke | `cargo check --manifest-path src-tauri/Cargo.toml` | N/A | pending |
| 05-01-03 | 01 | 1 | INFRA-03 | smoke | `cargo check --manifest-path src-tauri/Cargo.toml` | N/A | pending |
| 05-01-04 | 01 | 1 | INFRA-04 | smoke | `cargo tauri build` then check for `.sig` files | N/A | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] Key generation must happen before any build verification can succeed
- [ ] `TAURI_SIGNING_PRIVATE_KEY` env var must be set for signed build verification
- [ ] No automated test files needed — this phase is verified by successful build output

*Existing infrastructure covers compilation checks. Build-time signing verification requires key generation first.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ed25519 keypair exists | INFRA-01 | Key generation is a one-time interactive step | Run `npx tauri signer generate -w ~/.tauri/modtoggler.key`, verify both `.key` and `.key.pub` files exist |
| Key backed up in 2+ locations | INFRA-01 | Backup is a manual process (GitHub Secrets + password manager) | Copy key content to GitHub repo secrets as `TAURI_SIGNING_PRIVATE_KEY`, store in password manager |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s (check mode)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
