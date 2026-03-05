---
phase: 2
slug: core-mod-loop
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-05
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (Rust)** | `cargo test` with `tokio::test` for async, `tempfile` for fixtures |
| **Framework (Frontend)** | Vitest 4.x + jsdom + @testing-library/react |
| **Config file** | `vitest.config.ts` (frontend), inline in Cargo.toml (Rust) |
| **Quick run command** | `npx vitest run --reporter=verbose` / `cargo test -p modtoggler` |
| **Full suite command** | `npx vitest run && cargo test -p modtoggler` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p modtoggler` + `npx vitest run`
- **After every plan wave:** Run `npx vitest run && cargo test -p modtoggler`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | IMPORT-01 | unit (Rust) | `cargo test -p modtoggler import` | No - W0 | pending |
| 02-01-02 | 01 | 1 | IMPORT-02 | unit (Rust) | `cargo test -p modtoggler extract` | No - W0 | pending |
| 02-01-03 | 01 | 1 | IMPORT-03 | unit (Rust) | `cargo test -p modtoggler group` | No - W0 | pending |
| 02-01-04 | 01 | 1 | IMPORT-04 | unit (Rust) | `cargo test -p modtoggler sub_mod` | No - W0 | pending |
| 02-01-05 | 01 | 1 | IMPORT-06 | unit (Rust) | `cargo test -p modtoggler zipslip` | No - W0 | pending |
| 02-02-01 | 02 | 1 | TOGGLE-01 | unit (Rust) | `cargo test -p modtoggler toggle` | No - W0 | pending |
| 02-02-02 | 02 | 1 | TOGGLE-02 | unit (Rust) | `cargo test -p modtoggler disable` | No - W0 | pending |
| 02-02-03 | 02 | 1 | TOGGLE-03 | unit (Rust) | `cargo test -p modtoggler enable` | No - W0 | pending |
| 02-02-04 | 02 | 1 | TOGGLE-05 | unit (Rust) | `cargo test -p modtoggler sub_mod_toggle` | No - W0 | pending |
| 02-02-05 | 02 | 1 | TOGGLE-07 | unit (Rust) | `cargo test -p modtoggler delete` | No - W0 | pending |
| 02-03-01 | 03 | 2 | CONFLICT-01 | unit (Rust) | `cargo test -p modtoggler conflict` | No - W0 | pending |
| 02-03-02 | 03 | 2 | CONFLICT-02 | unit (TS) | `npx vitest run src/components/ConflictDialog` | No - W0 | pending |
| 02-04-01 | 04 | 2 | IMPORT-05 | unit (TS) | `npx vitest run src/components/ModCard` | No - W0 | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/services/import.rs` — test stubs for IMPORT-01 through IMPORT-06
- [ ] `src-tauri/src/services/toggle.rs` — test stubs for TOGGLE-01 through TOGGLE-05, TOGGLE-07
- [ ] `src-tauri/src/services/conflict.rs` — test stubs for CONFLICT-01
- [ ] `src/components/ModCard.test.tsx` — mod card rendering tests for IMPORT-05
- [ ] `src/components/ConflictDialog.test.tsx` — conflict dialog tests for CONFLICT-02
- [ ] `src/hooks/useMods.test.ts` — hook tests following useGames.test.ts pattern
- [ ] Test zip fixtures: small test .zip files with known structure for Rust tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Conflict warning on enable | CONFLICT-03 | Requires full Tauri runtime + UI interaction | Enable conflicting mod, verify dialog appears with correct mod names and file paths |
| Drag-and-drop import | IMPORT-01 | Requires Tauri drag-drop events | Drag .zip onto mod list, verify import dialog appears |
| Toggle state persists across restart | TOGGLE-04 | Requires app restart cycle | Toggle mod, close app, reopen, verify state unchanged |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
