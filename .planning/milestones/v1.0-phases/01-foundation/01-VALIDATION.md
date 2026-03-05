---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-04
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 2.x (frontend) + cargo test (Rust backend) |
| **Config file** | `vitest.config.ts` — Wave 0 creates if not present |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml && npm run test -- --run` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo test --manifest-path src-tauri/Cargo.toml && npm run test -- --run`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | GAME-01 | unit (Rust) | `cargo test commands::games::test_add_game` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | GAME-02 | unit (Rust) | `cargo test commands::games::test_remove_game_cascade` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | GAME-03 | unit (Rust) | `cargo test commands::games::test_edit_game` | ❌ W0 | ⬜ pending |
| 01-01-04 | 01 | 1 | GAME-04 | unit (Vitest) | `npm run test -- --run src/components/GameSelector.test.tsx` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | TOGGLE-04 | unit (Rust) | `cargo test db::queries::test_mod_enabled_persists` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | TOGGLE-06 | unit (Rust) | `cargo test services::journal::test_incomplete_journal_detected` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | RELIAB-01 | unit (Rust) | `cargo test commands::integrity::test_missing_from_game_dir` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | RELIAB-02 | unit (Rust) | `cargo test services::file_ops::test_cross_drive_fallback` | ❌ W0 | ⬜ pending |
| 01-03-03 | 03 | 1 | RELIAB-03 | unit (Rust) | `cargo test error::test_permission_denied_mapping` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/games.rs` — test module with `test_add_game`, `test_remove_game_cascade`, `test_edit_game`
- [ ] `src-tauri/src/services/journal.rs` — test module with `test_incomplete_journal_detected`
- [ ] `src-tauri/src/services/file_ops.rs` — test module with `test_cross_drive_fallback` (mock IO)
- [ ] `src-tauri/src/commands/integrity.rs` — test module with `test_missing_from_game_dir`
- [ ] `src-tauri/src/error.rs` — test module with `test_permission_denied_mapping`
- [ ] `src/components/GameSelector.test.tsx` — Vitest + React Testing Library; mock `commands.listGames()`
- [ ] `vitest.config.ts` — configure with `@tauri-apps/api` mock
- [ ] `src-tauri/src/db/test_helpers.rs` — in-memory SQLite database fixture for Rust tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| UAC elevation prompt appears once at session start | RELIAB-03 | Requires real Windows UAC dialog | Launch app with game in Program Files, verify single UAC prompt |
| Staging folder created on disk when adding game | GAME-01 | Filesystem side-effect verification | Add game, check ~/.modtoggler/games/[name]/staging/ exists |
| Folder picker dialog opens native OS picker | GAME-01 | Native dialog interaction | Click browse button, verify native folder picker opens |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
