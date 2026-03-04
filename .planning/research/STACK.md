# Stack Research

**Domain:** Tauri v2 desktop mod manager (file toggling, game mod management)
**Researched:** 2026-03-04
**Confidence:** MEDIUM-HIGH (core stack HIGH; peripheral crates MEDIUM due to version discrepancies in sources)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Tauri | 2.10.3 | App shell, IPC bridge, file system access | Chosen by project; smaller binary than Electron, Rust backend handles file ops safely, active development (2.10.3 released 2026-03-04) |
| React | 19.x | Frontend UI framework | Chosen by project; React 19 is current stable, Tauri's create-tauri-app templates target React 19 |
| TypeScript | 5.x | Type safety across frontend and IPC boundary | Mandatory — without TS, the type-safe IPC bridge (tauri-specta) has no value; catches interface drift between Rust commands and frontend callers |
| Vite | 7.x | Frontend build tool | Default bundler for Tauri's React template; native ES modules dev server means near-instant HMR, no config required for TS/JSX |
| Rust (stable) | 1.77.2+ | Backend logic, file operations, zip extraction | Required by Tauri; Rust handles large file moves without freezing the UI; async Tauri commands keep the app responsive during 200MB mod toggles |

### State Management

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| Zustand | 5.x (5.0.6+) | Global UI state (selected game, active mod list, toggling in-progress flags) | Minimal boilerplate, no Provider wrapping needed, React 19 compatible via useSyncExternalStore, community consensus for Tauri apps; Redux is overkill for this scope |
| TanStack Query | 5.x (5.90+) | Async data layer over Tauri commands (loading mod lists, reading game configs from SQLite) | Wraps `invoke()` calls with caching, loading/error states, and refetch-on-focus semantics; eliminates manual loading state management across mod listing and conflict detection queries |

### UI and Styling

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| Tailwind CSS | 4.x (4.2+) | Utility-first styling | v4 is current stable (released Jan 2025); zero-config CSS-native approach; pairs natively with shadcn/ui; dramatically faster build than v3 |
| shadcn/ui | latest (CLI-based) | Accessible component primitives (buttons, dialogs, toggles, lists) | Not a dependency — components are copied into your codebase as source you own; built on Radix UI primitives so accessibility is handled; updated for Tailwind v4 and React 19; toggle/switch and list components directly match the mod manager UI pattern |
| Lucide React | latest | Icon set | Ships with shadcn/ui; consistent, typed icon components; covers toggle, folder, warning, game-pad icons needed for this UI |

### Routing

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| TanStack Router | 1.x | Client-side routing (game selector → mod list → mod detail) | Type-safe route params and search params — no stringly-typed `useParams`; designed for Vite/SPAs (no SSR overhead); community-confirmed pattern for Tauri desktop apps; React Router v6+ works but loses the type safety benefit |

### Data Persistence

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| @tauri-apps/plugin-sql | 2.x | SQLite interface from frontend | Official Tauri plugin; provides migrations, parameterized queries, and async execute/select from TypeScript; SQLite is the right choice (single file, no server, handles relational mod/game/profile data) |
| tauri-plugin-sql (Rust) | 2.3.x | SQLite via sqlx in Rust backend | Same plugin — Rust-side registers it; use sqlx directly in Rust commands when you need transactions or complex queries the JS API doesn't expose |

### IPC and Type Safety

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| tauri-specta | 2.0.0-rc.21 | Generate TypeScript bindings from Rust command signatures | Eliminates the hand-maintained `invoke<ReturnType>('command_name', args)` pattern; Rust structs annotated with `#[derive(Type, Serialize)]` automatically produce matching TS types; compile-time guarantee that frontend call shapes match Rust handler signatures |

### File Operations (Rust Backend)

| Crate | Version | Purpose | Why Recommended |
|-------|---------|---------|-----------------|
| zip | 2.x (latest stable) | Extract .zip mod archives in Rust | Official `zip` crate; `ZipArchive::extract()` handles multi-file archives; run in a Tauri `async` command so extraction of large archives doesn't block the UI thread |
| @tauri-apps/plugin-fs | 2.x | Frontend-triggered file ops within capability-scoped paths | Official plugin; `rename()` moves files (toggle on = rename from staging to game dir); scoped permissions model prevents path traversal; needed for any file operations not done purely in Rust |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| create-tauri-app | Scaffold initial project | `npm create tauri-app@latest` — choose React + TypeScript template to get Vite config wired |
| tauri-cli | Build, dev server, sign/bundle | `npm run tauri dev` / `npm run tauri build`; v2 CLI required (not v1) |
| Vitest | Unit testing | Ships with Vite ecosystem; mock Tauri `invoke` with `vi.mock('@tauri-apps/api/core')`; test store logic and command handlers in isolation |
| ESLint + typescript-eslint | Lint | Standard for TS React projects; catch IPC mismatches and type errors before runtime |
| Prettier | Formatting | No decisions to make per file; configure once and forget |

---

## Installation

```bash
# Scaffold the app (run once)
npm create tauri-app@latest ModToggler -- --template react-ts

# State management + async data
npm install zustand @tanstack/react-query @tanstack/react-router

# UI components (shadcn CLI handles individual component installs)
npm install tailwindcss @tailwindcss/vite
npx shadcn@latest init
# Then add components as needed:
npx shadcn@latest add switch button dialog card badge

# Icons (installed with shadcn but explicit for clarity)
npm install lucide-react

# Tauri plugins (JS side)
npm run tauri add sql
npm run tauri add fs
# or manually:
npm install @tauri-apps/plugin-sql @tauri-apps/plugin-fs

# Dev dependencies
npm install -D vitest @testing-library/react typescript-eslint prettier

# Rust side — in src-tauri/:
cargo add tauri-plugin-sql --features sqlite
cargo add zip
cargo add tauri-specta --features typescript,tauri
cargo add specta-typescript
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Zustand | Redux Toolkit | Only if the team already has Redux expertise and the app grows to 50+ interdependent slices; overkill here |
| Zustand | Jotai | Viable if you prefer atom-based granular subscriptions; Zustand's slice pattern is simpler for this use case |
| TanStack Query | Manual `invoke` + `useState` | Never — even for simple cases TanStack Query's error/loading/stale states prevent subtle bugs |
| TanStack Router | React Router v6 | If the team already uses React Router extensively and doesn't need typed search params; both work, TanStack Router just gives more safety |
| shadcn/ui | MUI (Material UI) | If the design system needs to match Material Design conventions; MUI's opinionated styling fights Tailwind |
| shadcn/ui | Radix UI (bare) | If you want lower-level control without shadcn's layout opinions; more assembly required |
| @tauri-apps/plugin-sql + SQLite | JSON flat files via plugin-store | Only for trivial key/value config (< 5 settings); once you have mod lists, profiles, conflict tracking, relational data wins |
| zip crate (Rust) | tauri-plugin-extract | tauri-plugin-extract is 3-star, 9-commit, no clear Tauri v2 support confirmation; the zip crate is battle-tested (107M downloads) and used directly in a custom Tauri command |
| tauri-specta | Hand-typed `invoke` calls | Never on a project of this size; drift between Rust types and frontend types is guaranteed to cause bugs |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Electron | 80-200MB baseline binary; entire Chromium bundled; slower startup; Tauri is already chosen, but noted for completeness | Tauri v2 |
| Create React App (CRA) | Officially deprecated; webpack is slow; no longer maintained | Vite via create-tauri-app template |
| Redux (without Toolkit) | Massive boilerplate; no benefit over Zustand for this app's state complexity | Zustand |
| MobX | Implicit reactivity makes debugging hard; overkill for this scope | Zustand |
| next.js + Tauri | Next.js's SSR/RSC model fights Tauri's SPA assumption; no routing benefit in a desktop app; adds significant complexity | Vite + TanStack Router |
| tauri-plugin-extract | 3 stars, 9 commits, no confirmed Tauri v2 support; community-unknown | zip Rust crate in custom Tauri command |
| zip-extract crate | Explicitly deprecated in favor of the zip crate's built-in `extract()` method | zip crate directly |
| Separate UAC helper for Program Files | Adds binary distribution complexity; Tauri v2's `fs` plugin with explicit path scopes (`C:/Program Files/**/*`) handles this — test on target machine to verify | @tauri-apps/plugin-fs with explicit scope |
| CSS Modules or styled-components | Fights Tailwind's utility-first approach; shadcn/ui uses Tailwind for all component styles | Tailwind CSS + shadcn/ui |

---

## Stack Patterns by Variant

**For file toggle operations (mod on/off):**
- Write a Rust Tauri command (`#[tauri::command] async fn toggle_mod(...)`) that calls `std::fs::rename()` — Rust's rename is an atomic OS-level move within the same drive, much faster than copy+delete
- Expose it via tauri-specta for type-safe `invoke`
- Call it via TanStack Query mutation (`useMutation`) to get optimistic update / rollback semantics

**For zip import:**
- Write an async Rust command that accepts a path, opens the zip with the `zip` crate, iterates entries, extracts to `~/.modtoggler/staging/[game]/[mod_name]/`, and returns the discovered file manifest
- Progress reporting: use Tauri events (`app.emit_to()`) to stream extraction progress to the frontend; display in a dialog using shadcn `Progress` component

**For conflict detection:**
- Store file-to-mod mappings in SQLite: `mod_files(mod_id, relative_path)`
- Query for overlapping `relative_path` values when enabling a mod
- Run purely in Rust command layer for performance on large file lists

**For Windows Program Files access:**
- Configure `fs` plugin scope in `src-tauri/capabilities/default.json`:
  ```json
  { "identifier": "fs:scope", "allow": [{ "path": "$HOME/.modtoggler/**" }, { "path": "C:/Program Files (x86)/Steam/steamapps/**" }] }
  ```
- Do all actual file moves in Rust commands (not plugin-fs from JS) for reliability with large files
- Test on target machine without admin rights first; rename() within the same drive partition does not require elevation

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| tauri 2.10.x | tauri-plugin-fs 2.x, tauri-plugin-sql 2.x | Plugin major versions must match Tauri major version; v1 plugins do not work with Tauri v2 |
| tauri-specta 2.0.0-rc.21 | tauri 2.x, specta 2.x | Still RC but widely used in the community; stable enough for production; do not use tauri-specta 1.x (targets Tauri v1) |
| React 19 | TanStack Query 5.x, Zustand 5.x, TanStack Router 1.x | All require React 18+; React 19 tested and working |
| Tailwind CSS 4.x | shadcn/ui (latest CLI) | shadcn CLI now initializes with Tailwind v4 by default for new projects; use `npx shadcn@latest init` |
| zip crate | Rust 1.77.2+ (required by tauri-plugin-sql) | No conflicts; standard Rust crate |

---

## Sources

- [Tauri v2 official site](https://v2.tauri.app/) — core framework, plugin list, version 2.10.3 confirmed
- [Tauri File System Plugin docs](https://v2.tauri.app/plugin/file-system/) — scope configuration, rename/copy APIs, path traversal prevention
- [Tauri SQL Plugin docs](https://v2.tauri.app/plugin/sql/) — SQLite setup, migration support, Rust 1.77.2+ requirement
- [tauri-specta GitHub](https://github.com/specta-rs/tauri-specta) — version 2.0.0-rc.21, Tauri v2 requirement confirmed (MEDIUM confidence — RC status)
- [Zustand GitHub](https://github.com/pmndrs/zustand) — version 5.0.6, useSyncExternalStore, React 18+ (HIGH confidence)
- [TanStack Query npm](https://www.npmjs.com/package/@tanstack/react-query) — version 5.90.21 (HIGH confidence)
- [TanStack Router docs](https://tanstack.com/router/latest) — v1.x, Tauri desktop app pattern confirmed by community (MEDIUM confidence)
- [Tailwind CSS v4 blog](https://tailwindcss.com/blog/tailwindcss-v4) — v4.0 released Jan 2025, v4.2 current (HIGH confidence)
- [shadcn/ui Tailwind v4 docs](https://ui.shadcn.com/docs/tailwind-v4) — React 19 + Tailwind v4 support confirmed (HIGH confidence)
- [Vite docs](https://vite.dev/guide/) — version 7.3.1 (HIGH confidence)
- [zip crate docs.rs](https://docs.rs/crate/zip/latest) — 8.1.0 referenced; battle-tested at 107M downloads (MEDIUM confidence — version sourcing had discrepancies across search results)
- [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) — production-ready Tauri v2 + React 19 + shadcn + Zustand + TanStack Query template; cross-validates recommended stack (MEDIUM confidence)
- [Tauri UAC elevation discussion](https://github.com/orgs/tauri-apps/discussions/4201) — Windows elevated permissions approach (MEDIUM confidence)

---

*Stack research for: Tauri v2 desktop mod manager (ModToggler)*
*Researched: 2026-03-04*
