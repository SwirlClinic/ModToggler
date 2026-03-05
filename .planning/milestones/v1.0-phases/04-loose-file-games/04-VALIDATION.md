---
phase: 4
slug: loose-file-games
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Vitest 4.x (frontend) |
| **Config file** | Cargo.toml / vitest.config.ts |
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
| 4-01-01 | 01 | 1 | LOOSE-01 | unit | `cd src-tauri && cargo test queries::tests::test_insert_game_loose` | Partial | ⬜ pending |
| 4-01-02 | 01 | 1 | LOOSE-02 | unit | `cd src-tauri && cargo test queries::tests::test_file_entry_destination_path` | ❌ W0 | ⬜ pending |
| 4-01-03 | 01 | 1 | LOOSE-03 | unit | `cd src-tauri && cargo test queries::tests::test_file_entry_destination_path` | ❌ W0 | ⬜ pending |
| 4-01-04 | 01 | 1 | LOOSE-04 | unit | `cd src-tauri && cargo test toggle::tests::test_build_loose_file_pairs` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Migration v8: add `destination_path` to `file_entries`, add `mod_type` to `mods`
- [ ] `src-tauri/src/services/toggle.rs` — add test_build_loose_file_pairs test
- [ ] `src-tauri/src/db/queries.rs` — add test for insert_file_entry with destination_path
- [ ] Migration version 8 test — verify migration count and version uniqueness

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| File picker multi-select dialog | LOOSE-02 | OS native dialog | Open loose-file import, verify multi-select file picker appears |
| File mapping table editable paths | LOOSE-03 | UI interaction | Import files, verify destination path column is editable and defaults to "/" |
| Loose badge on game name | LOOSE-01 | Visual styling | Configure game as loose, verify badge appears |
| Bulk path assignment | LOOSE-03 | UI interaction | Select multiple files, set path for all at once |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
