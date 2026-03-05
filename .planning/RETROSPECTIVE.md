# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — MVP

**Shipped:** 2026-03-05
**Phases:** 4 | **Plans:** 15

### What Was Built
- Full mod management pipeline: import from zip, toggle on/off, conflict detection, sub-mod support
- Game configuration with integrity scanning and crash recovery via transaction journal
- Per-game profiles for saving/restoring mod configurations
- Loose-file game support with manual file tagging and destination path mapping

### What Worked
- Wave-based parallel execution kept plan durations consistently short (avg 6min/plan)
- Building reliability primitives first (journal, integrity scan) in Phase 1 paid off — toggle service in Phase 2 was straightforward
- Explicit sub-mod file ordering (before parent on disable, after on enable) prevented edge cases
- tauri-specta typed bindings eliminated IPC contract bugs between Rust and TypeScript

### What Was Inefficient
- Phase 2 progress table in ROADMAP.md became stale (showed 4/5 when 5/5 were complete)
- Some research docs had outdated crate versions (zip v2.x vs actual v8.x) — required correction during execution
- Performance metrics table in STATE.md only tracked 3 of 4 phases

### Patterns Established
- unwrap() helper for tauri-specta Result<T, AppError> pattern in React Query hooks
- Two-click confirm pattern for destructive actions (delete) instead of confirmation dialogs
- Popover (not Select) for mixed-content dropdowns (items + action buttons)
- mod_type column on mods table for explicit type tracking (not inferred)
- snake_case field names matching Rust bindings, not camelCase JavaScript convention

### Key Lessons
1. Build reliability infrastructure before the features that depend on it — retrofitting crash recovery is much harder
2. Pin dependency versions explicitly when crate ecosystems have breaking API changes between majors
3. Human verification phase (02-05) caught 3 real bugs — idempotent migrations, foreign keys, empty state UI — that automated tests missed

### Cost Observations
- Model mix: 100% opus (quality profile)
- Total execution: ~1.27 hours across 15 plans
- Notable: Average 6min/plan is very fast for full-stack features

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 4 | 15 | Initial baseline — wave-based parallel execution |

### Top Lessons (Verified Across Milestones)

1. (First milestone — lessons above are candidates for cross-validation in v1.1+)
