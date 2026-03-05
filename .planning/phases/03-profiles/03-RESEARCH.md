# Phase 3: Profiles - Research

**Researched:** 2026-03-05
**Domain:** Profile CRUD + batch mod toggling (SQLite, Tauri commands, React UI)
**Confidence:** HIGH

## Summary

Phase 3 adds named profiles scoped per game. A profile is a snapshot of which mods (and sub-mods) are enabled/disabled. The backend needs a new `profiles` table with FK to `games`, plus a `profile_entries` table mapping profile -> mod -> enabled state (and sub-mod states). The frontend needs a dropdown in the ModList header bar and two dialogs (save and manage).

This phase is straightforward CRUD with one non-trivial operation: profile loading, which must batch-toggle multiple mods to match the saved state. The existing `toggle_mod` service already handles individual mod enable/disable with journal safety. Profile apply should call the existing toggle infrastructure per-mod rather than reimplementing file moves.

**Primary recommendation:** Store profiles as (profile_id, mod_id, enabled, sub_mod_entries_json) rows. Profile load iterates mods and calls the existing toggle service for each mod that needs to change state. Use shadcn/Radix Popover for the dropdown (not Select, since it needs mixed content: profile names + actions).

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Dropdown in the ModList header bar (next to Import button) -- labeled with last-loaded profile name
- Dropdown shows: saved profile names, separator, "Save Current...", "Manage Profiles"
- Clicking a profile name = immediate load (no confirmation dialog)
- "Save Current..." opens a small dialog with name input, Cancel/Save buttons
- Toast confirms save and load actions
- Current state is NOT auto-saved -- user saves explicitly
- Loading a profile batch-toggles all mods to match the saved state
- If a profile references mods that were deleted: skip missing mods, apply the rest, show toast listing which mods were skipped
- Profile data stays saved as-is even if some mods no longer exist
- Names must be unique per game -- saving with an existing name overwrites (with confirmation: "Overwrite existing profile 'X'?")
- "Manage Profiles" opens a dialog/panel listing all profiles for the current game with delete buttons
- Delete requires confirmation dialog
- Dropdown button shows last-loaded profile name; no "modified" tracking
- If no profile has been loaded this session, shows "Profiles"
- No modified/dirty state tracking

### Claude's Discretion
- Exact dropdown component implementation (Radix popover, custom, etc.)
- Profile data storage schema (DB table structure)
- Batch toggle optimization (whether to use a single transaction or per-mod toggles)
- Manage Profiles dialog layout
- Animation/transition for profile apply

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROFILE-01 | User can save current mod configuration as a named profile | DB schema (profiles + profile_entries tables), save command that snapshots current mod states |
| PROFILE-02 | User can load a saved profile, which enables/disables mods to match the profile state | Apply command that diffs current vs saved state, calls existing toggle_mod per change |
| PROFILE-03 | User can delete a saved profile | Simple DELETE CASCADE on profiles table |
| PROFILE-04 | Profiles are per-game | profiles.game_id FK to games(id), all queries filtered by game_id |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sqlx | (existing) | Profile DB queries | Already used for all DB access |
| tauri-specta | (existing) | Typed Tauri commands | Already generates TypeScript bindings |
| @tanstack/react-query | (existing) | Profile data fetching/mutations | Already used for all async state |
| zustand | (existing) | Last-loaded profile name (session state) | Already used for gameStore |
| sonner | (existing) | Toast notifications | Already used for save/load/delete feedback |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| radix-ui (via shadcn) | 1.4.x | Popover component for profile dropdown | Dropdown needs mixed content (names + actions) |
| lucide-react | (existing) | Icons for dropdown/dialog | ChevronDown, Save, Trash2 icons |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Radix Popover | shadcn Select | Select is for value selection only; Popover supports mixed content (items + separators + action buttons) |
| Per-mod toggles | Single SQL transaction | Per-mod reuses journal safety; single transaction would skip crash recovery |

**Installation:**
```bash
npx shadcn@latest add popover
```
(All other dependencies already installed)

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/profiles.rs    # Tauri commands: save, load, delete, list profiles
├── db/queries.rs           # Add profile CRUD queries (same file, new section)
├── db/migrations.rs        # Migration v7: profiles table, v8: profile_entries table (or combined)

src/
├── hooks/useProfiles.ts    # useProfiles, useSaveProfile, useLoadProfile, useDeleteProfile
├── components/
│   ├── ProfileDropdown.tsx  # Popover dropdown in ModList header
│   ├── SaveProfileDialog.tsx # Name input dialog
│   └── ManageProfilesDialog.tsx # List + delete dialog
├── store/gameStore.ts      # Add lastLoadedProfileName to existing store
```

### Pattern 1: Profile DB Schema
**What:** Two tables -- profiles (metadata) and profile_entries (mod states)
**When to use:** Always -- normalized storage for profile data
**Example:**
```sql
-- Migration v7
CREATE TABLE IF NOT EXISTS profiles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id     INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(game_id, name)
);
CREATE INDEX IF NOT EXISTS idx_profiles_game_id ON profiles(game_id);

CREATE TABLE IF NOT EXISTS profile_entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    mod_id      INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    enabled     INTEGER NOT NULL DEFAULT 0,
    sub_mod_states TEXT  -- JSON: [{"sub_mod_id": 1, "enabled": true}, ...]
);
CREATE INDEX IF NOT EXISTS idx_profile_entries_profile ON profile_entries(profile_id);
```

**Key design decisions:**
- `UNIQUE(game_id, name)` enforces per-game uniqueness at DB level
- `profile_entries.mod_id` FK to mods -- CASCADE DELETE means if a mod is deleted, its profile entries vanish automatically (profile row stays, entry just disappears)
- `sub_mod_states` as JSON avoids a third join table for a simple list of booleans
- No need to store mod names in profile -- join on load to get current names

### Pattern 2: Profile Save (Snapshot Current State)
**What:** Read all mods for game, create profile row, insert entries for each mod
**When to use:** When user clicks "Save Current..."
**Example:**
```rust
pub async fn save_profile(
    pool: &SqlitePool,
    game_id: i64,
    name: &str,
) -> Result<ProfileRecord, AppError> {
    // Check if name exists -- if so, delete old one (upsert pattern)
    if let Some(existing) = get_profile_by_name(pool, game_id, name).await? {
        delete_profile(pool, existing.id).await?;
    }

    let profile = insert_profile(pool, game_id, name).await?;
    let mods = queries::list_mods_for_game(pool, game_id).await?;

    for mod_rec in &mods {
        let sub_mods = queries::list_sub_mods(pool, mod_rec.id).await?;
        let sub_states: Vec<SubModState> = sub_mods.iter().map(|sm| SubModState {
            sub_mod_id: sm.id,
            enabled: sm.user_enabled, // Use user_enabled, not effective enabled
        }).collect();
        let sub_json = serde_json::to_string(&sub_states)?;
        insert_profile_entry(pool, profile.id, mod_rec.id, mod_rec.enabled, &sub_json).await?;
    }

    Ok(profile)
}
```

### Pattern 3: Profile Load (Apply Saved State)
**What:** Read profile entries, diff against current state, call toggle_mod for each change
**When to use:** When user clicks a profile name in dropdown
**Example:**
```rust
pub async fn apply_profile(
    app: &AppHandle,
    pool: &SqlitePool,
    profile_id: i64,
) -> Result<ApplyProfileResult, AppError> {
    let entries = list_profile_entries(pool, profile_id).await?;
    let profile = get_profile(pool, profile_id).await?;
    let current_mods = queries::list_mods_for_game(pool, profile.game_id).await?;

    let mut skipped_mods: Vec<String> = Vec::new();

    for entry in &entries {
        // Check if mod still exists
        match queries::get_mod(pool, entry.mod_id).await {
            Ok(current_mod) => {
                if current_mod.enabled != entry.enabled {
                    toggle::toggle_mod(app, pool, entry.mod_id, entry.enabled).await?;
                }
                // Handle sub-mod states if needed
                if let Some(ref sub_states_json) = entry.sub_mod_states {
                    apply_sub_mod_states(app, pool, entry.mod_id, sub_states_json).await?;
                }
            }
            Err(_) => {
                // Mod was deleted -- skip it
                skipped_mods.push(format!("mod_id={}", entry.mod_id));
            }
        }
    }

    // Also handle mods that exist now but weren't in the profile
    // (mods imported after profile was saved) -- disable them
    for current_mod in &current_mods {
        let in_profile = entries.iter().any(|e| e.mod_id == current_mod.id);
        if !in_profile && current_mod.enabled {
            toggle::toggle_mod(app, pool, current_mod.id, false).await?;
        }
    }

    Ok(ApplyProfileResult { skipped_mods })
}
```

### Pattern 4: Popover Dropdown Component
**What:** Radix Popover with profile list + action items
**When to use:** ModList header bar
**Example:**
```tsx
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Button } from '@/components/ui/button'
import { ChevronDown } from 'lucide-react'
import { Separator } from '@/components/ui/separator'

function ProfileDropdown({ gameId }: { gameId: number }) {
  const { data: profiles = [] } = useProfiles(gameId)
  const loadProfile = useLoadProfile()
  const lastLoaded = useGameStore((s) => s.lastLoadedProfileName)

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm">
          {lastLoaded || 'Profiles'}
          <ChevronDown className="ml-1 h-3 w-3" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-48 p-1">
        {profiles.map(p => (
          <button key={p.id} onClick={() => loadProfile.mutate(p.id)}
            className="w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent">
            {p.name}
          </button>
        ))}
        {profiles.length > 0 && <Separator className="my-1" />}
        <button onClick={openSaveDialog} className="...">Save Current...</button>
        <button onClick={openManageDialog} className="...">Manage Profiles</button>
      </PopoverContent>
    </Popover>
  )
}
```

### Anti-Patterns to Avoid
- **Storing full file paths in profiles:** Only store mod IDs and enabled states. File paths change if staging moves.
- **Reimplementing file moves in profile apply:** Always delegate to existing `toggle_mod` / `toggle_sub_mod` services. They handle journals, cross-drive, sub-mod ordering.
- **Storing mod names in profile_entries:** Names can change; always join on current mods table. For skipped-mod toasts, store the mod_id and note it was missing.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dropdown with mixed content | Custom dropdown from scratch | shadcn Popover | Keyboard nav, focus management, portal rendering |
| Mod toggle orchestration | Direct file moves in profile apply | Existing `toggle::toggle_mod()` | Journal safety, sub-mod ordering, cross-drive handling |
| Unique name enforcement | Application-level check only | `UNIQUE(game_id, name)` DB constraint | Race conditions, data integrity |
| Toast notifications | Alert dialogs | Sonner toast (already wired) | Non-blocking, consistent with existing UX |

## Common Pitfalls

### Pitfall 1: Forgetting Sub-Mod States in Profiles
**What goes wrong:** Profile saves only mod enabled/disabled but ignores sub-mod states. Loading a profile doesn't restore which sub-mod options were active.
**Why it happens:** Sub-mods are a secondary concept, easy to forget.
**How to avoid:** Include sub-mod states (keyed by sub_mod_id) in profile_entries. Use `user_enabled` (not `enabled`) when saving, since that represents the user's intent.
**Warning signs:** Sub-mods reset to default after profile load.

### Pitfall 2: New Mods Not Handled on Profile Load
**What goes wrong:** User imports a new mod after saving a profile. Loading that profile leaves the new mod in whatever state it was in.
**Why it happens:** Profile only has entries for mods that existed at save time.
**How to avoid:** On profile load, disable any enabled mods that are NOT in the profile (they were added after the profile was saved). This matches user expectation: "this profile means exactly these mods."
**Warning signs:** Loading a profile doesn't produce a clean state.

### Pitfall 3: Profile Apply Order Matters
**What goes wrong:** If you enable mods before disabling others, you might trigger unnecessary conflict warnings or file overwrites.
**Why it happens:** Toggling is sequential and the order affects intermediate states.
**How to avoid:** Apply disables first, then enables. This ensures a clean intermediate state.
**Warning signs:** Conflict toasts appearing during profile load.

### Pitfall 4: Session-Only "Last Loaded" State
**What goes wrong:** Storing last-loaded profile name in DB or localStorage causes confusion if profile was deleted or mods changed since.
**Why it happens:** Over-engineering persistence for a label.
**How to avoid:** Store `lastLoadedProfileName` in Zustand only (session state). Resets to "Profiles" on app restart. This matches the user decision: "no modified tracking."
**Warning signs:** Stale profile name showing after deletion.

### Pitfall 5: Overwrite Confirmation Must Check BEFORE Delete
**What goes wrong:** Deleting the old profile before confirming overwrite means cancellation loses the old profile.
**Why it happens:** Implementing upsert as delete-then-insert.
**How to avoid:** Show confirmation dialog on frontend before calling save command. The save command can safely delete-then-insert because user already confirmed.
**Warning signs:** Cancelling overwrite dialog but old profile is already gone.

## Code Examples

### Tauri Command Registration Pattern
```rust
// In src-tauri/src/lib.rs, add to collect_commands![]
commands::profiles::save_profile_cmd,
commands::profiles::load_profile_cmd,
commands::profiles::list_profiles_cmd,
commands::profiles::delete_profile_cmd,
```

### React Query Hook Pattern (follows useMods.ts exactly)
```typescript
export function useProfiles(gameId: number | null) {
  return useQuery({
    queryKey: ['profiles', gameId],
    queryFn: async () => {
      if (!gameId) return []
      return unwrap(await commands.listProfilesCmd(gameId))
    },
    enabled: gameId !== null,
  })
}

export function useSaveProfile() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { gameId: number; name: string }) => {
      return unwrap(await commands.saveProfileCmd(args.gameId, args.name))
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['profiles'] })
      toast.success(`Profile "${data.name}" saved`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Save failed')
    },
  })
}

export function useLoadProfile() {
  const queryClient = useQueryClient()
  const setLastLoaded = useGameStore((s) => s.setLastLoadedProfileName)
  return useMutation({
    mutationFn: async (args: { profileId: number; profileName: string }) => {
      return unwrap(await commands.loadProfileCmd(args.profileId))
    },
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['sub-mods'] })
      setLastLoaded(variables.profileName)
      if (data.skipped_mods.length > 0) {
        toast.warning(`Skipped missing mods: ${data.skipped_mods.join(', ')}`)
      } else {
        toast.success(`Profile loaded`)
      }
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Load failed')
    },
  })
}
```

### Zustand Store Extension
```typescript
interface GameStore {
  // ... existing fields ...
  lastLoadedProfileName: string | null
  setLastLoadedProfileName: (name: string | null) => void
}

export const useGameStore = create<GameStore>((set) => ({
  // ... existing ...
  lastLoadedProfileName: null,
  setLastLoadedProfileName: (name) => set({ lastLoadedProfileName: name }),
}))
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Store profile as flat JSON file | SQLite with proper FKs | Project convention | CASCADE deletes, referential integrity |
| Custom toggle logic per profile | Reuse existing toggle service | Phase 2 established | No duplicate file-move code |

## Open Questions

1. **Should profile load disable mods imported after the profile was saved?**
   - What we know: Context says "loading a profile batch-toggles all mods to match the saved state"
   - Recommendation: YES -- disable mods not in the profile. This gives a true "restore to this exact configuration" behavior. The user can always re-enable new mods after loading.

2. **Sub-mod state storage format**
   - What we know: Sub-mods have both `enabled` and `user_enabled` flags
   - Recommendation: Store as JSON array in `profile_entries.sub_mod_states`. Only store `sub_mod_id` + `enabled` (representing user intent). Simple and avoids a third table.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework (frontend) | Vitest 4.x + jsdom |
| Framework (backend) | cargo test (tokio::test for async) |
| Config file (frontend) | vitest.config.ts |
| Quick run command | `npx vitest run --reporter=verbose` / `cd src-tauri && cargo test` |
| Full suite command | `npx vitest run && cd src-tauri && cargo test` |

### Phase Requirements - Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROFILE-01 | Save current config as named profile | unit (Rust) | `cd src-tauri && cargo test queries::tests::test_save_profile` | No -- Wave 0 |
| PROFILE-01 | Save with duplicate name overwrites | unit (Rust) | `cd src-tauri && cargo test queries::tests::test_save_profile_overwrite` | No -- Wave 0 |
| PROFILE-02 | Load profile toggles mods to match | unit (Rust) | `cd src-tauri && cargo test profiles::tests::test_apply_profile` | No -- Wave 0 |
| PROFILE-02 | Load profile skips deleted mods | unit (Rust) | `cd src-tauri && cargo test profiles::tests::test_apply_profile_missing_mods` | No -- Wave 0 |
| PROFILE-03 | Delete profile removes entries | unit (Rust) | `cd src-tauri && cargo test queries::tests::test_delete_profile_cascade` | No -- Wave 0 |
| PROFILE-04 | Profiles scoped per game | unit (Rust) | `cd src-tauri && cargo test queries::tests::test_profiles_per_game` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test`
- **Per wave merge:** `npx vitest run && cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Profile CRUD query tests in `src-tauri/src/db/queries.rs` (tests module) -- covers PROFILE-01, PROFILE-03, PROFILE-04
- [ ] Profile apply logic tests -- covers PROFILE-02 (skipped mods, sub-mod restoration)
- [ ] Migration version 7 -- profiles + profile_entries tables
- [ ] `npx shadcn@latest add popover` -- UI component dependency

## Sources

### Primary (HIGH confidence)
- Project codebase inspection: db/queries.rs, services/toggle.rs, commands/mods.rs, hooks/useMods.ts, ModList.tsx, gameStore.ts, bindings.ts
- Existing patterns: React Query hooks, Tauri command registration, SQLite migration system, specta type generation

### Secondary (MEDIUM confidence)
- shadcn/Radix Popover component API (standard shadcn pattern, well-known)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, no new dependencies except shadcn popover
- Architecture: HIGH -- follows exact patterns from Phase 1 and 2 (new table, new queries, new commands, new hooks, new components)
- Pitfalls: HIGH -- directly derived from codebase analysis (sub-mod states, toggle ordering, session state)

**Research date:** 2026-03-05
**Valid until:** 2026-04-05 (stable -- no external dependencies changing)
