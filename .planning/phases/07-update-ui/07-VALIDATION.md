---
phase: 7
slug: update-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (existing) |
| **Config file** | vitest.config.ts |
| **Quick run command** | `npx vitest run --reporter=verbose` |
| **Full suite command** | `npx vitest run --reporter=verbose` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run --reporter=verbose`
- **After every plan wave:** Run `npx vitest run --reporter=verbose`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | UPD-01 | unit | `npx vitest run` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | UPD-02, UPD-05 | unit | `npx vitest run` | ❌ W0 | ⬜ pending |
| 07-01-03 | 01 | 1 | UPD-03 | manual | N/A | N/A | ⬜ pending |
| 07-01-04 | 01 | 1 | UPD-04 | manual | N/A | N/A | ⬜ pending |
| 07-01-05 | 01 | 1 | UPD-06 | unit | `npx vitest run` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/__tests__/useUpdateChecker.test.ts` — unit tests for update check hook logic
- [ ] `src/__tests__/UpdateBanner.test.tsx` — component rendering tests for update banner states

*Existing vitest infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Update banner appears with version | UPD-02 | Requires real update available from GitHub | Push a new tag, verify banner shows in running app |
| Release notes display and install flow | UPD-03 | Requires real download from GitHub Releases | Click "Install" on banner, verify download progress and install |
| Passive NSIS install and relaunch | UPD-04 | Requires real NSIS installer execution | Verify app closes briefly, NSIS runs passive mode, app relaunches on new version |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
