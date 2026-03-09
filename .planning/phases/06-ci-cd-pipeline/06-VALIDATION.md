---
phase: 6
slug: ci-cd-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | GitHub Actions (workflow validation via tag push) |
| **Config file** | `.github/workflows/release.yml` |
| **Quick run command** | YAML syntax check: `node -e "require('fs').readFileSync('.github/workflows/release.yml','utf8')"` |
| **Full suite command** | Push a test tag: `git tag v0.0.1-test && git push --tags` |
| **Estimated runtime** | ~5s (syntax), ~5min (full CI run) |

---

## Sampling Rate

- **After every task commit:** YAML file exists and parses without error
- **After every plan wave:** Push test tag to verify end-to-end workflow
- **Before `/gsd:verify-work`:** Full workflow run must produce expected release assets
- **Max feedback latency:** 5 seconds (local), 5 minutes (CI)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | CICD-01 | manual/e2e | Push `v*` tag, verify workflow triggers | N/A | pending |
| 06-01-02 | 01 | 1 | CICD-02 | manual/e2e | Check release for `.sig` files | N/A | pending |
| 06-01-03 | 01 | 1 | CICD-03 | manual/e2e | Check release for installer + `latest.json` | N/A | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `.github/workflows/` directory must be created
- [ ] `.github/workflows/release.yml` is the entire deliverable

*This phase is infrastructure-as-code. The workflow file IS the deliverable.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Workflow triggers on tag push | CICD-01 | Requires actual GitHub Actions runner | Push `v*` tag, check Actions tab for running workflow |
| Artifacts are signed | CICD-02 | Requires CI build with secrets | Check release assets for `.sig` files |
| Release contains installer + latest.json | CICD-03 | Requires CI build completion | Check GitHub Release page for expected assets |
| latest.json has correct version/URL/signature | CICD-03 | Requires downloading and inspecting manifest | Download latest.json, verify version matches tag, URL resolves, signature present |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s (local checks)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
