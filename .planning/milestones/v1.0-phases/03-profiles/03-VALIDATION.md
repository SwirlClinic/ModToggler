---
phase: 3
slug: profiles
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Vitest 4.x (frontend) |
| **Config file** | vitest.config.ts / Cargo.toml |
| **Quick run command** | `cd src-tauri && cargo test` |
| **Full suite command** | `npx vitest run && cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test`
- **After every plan wave:** Run `npx vitest run && cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | PROFILE-01 | unit | `cd src-tauri && cargo test queries::tests::test_save_profile` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 1 | PROFILE-01 | unit | `cd src-tauri && cargo test queries::tests::test_save_profile_overwrite` | ❌ W0 | ⬜ pending |
| 3-01-03 | 01 | 1 | PROFILE-02 | unit | `cd src-tauri && cargo test profiles::tests::test_apply_profile` | ❌ W0 | ⬜ pending |
| 3-01-04 | 01 | 1 | PROFILE-02 | unit | `cd src-tauri && cargo test profiles::tests::test_apply_profile_missing_mods` | ❌ W0 | ⬜ pending |
| 3-01-05 | 01 | 1 | PROFILE-03 | unit | `cd src-tauri && cargo test queries::tests::test_delete_profile_cascade` | ❌ W0 | ⬜ pending |
| 3-01-06 | 01 | 1 | PROFILE-04 | unit | `cd src-tauri && cargo test queries::tests::test_profiles_per_game` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Migration v7: `profiles` + `profile_entries` tables
- [ ] Profile CRUD query tests in `src-tauri/src/db/queries.rs` (tests module) — covers PROFILE-01, PROFILE-03, PROFILE-04
- [ ] Profile apply logic tests — covers PROFILE-02 (skipped mods, sub-mod restoration)
- [ ] `npx shadcn@latest add popover` — UI component dependency

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dropdown shows profile names + actions | PROFILE-01 | UI visual behavior | Open dropdown, verify profile names listed above separator, Save/Manage below |
| Profile load updates dropdown label | PROFILE-02 | Session UI state | Load profile, verify button label changes to profile name |
| Overwrite confirmation dialog | PROFILE-01 | Dialog interaction flow | Save with existing name, verify confirmation dialog appears |
| Delete confirmation dialog | PROFILE-03 | Dialog interaction flow | Click delete in Manage, verify confirmation dialog appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
