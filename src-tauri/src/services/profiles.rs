use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::db::queries::{self, ProfileRecord};
use crate::error::AppError;
use crate::services::toggle;

/// Sub-mod state entry stored as JSON in profile_entries.sub_mod_states.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubModState {
    pub sub_mod_id: i64,
    pub enabled: bool,
}

/// Result of applying a profile, including any mods that were skipped (deleted since save).
#[derive(Debug, Serialize, Type, Clone)]
pub struct ApplyProfileResult {
    pub skipped_mods: Vec<String>,
}

/// Save the current mod configuration for a game as a named profile.
/// If a profile with the same name exists for this game, it is replaced.
pub async fn save_profile(
    pool: &SqlitePool,
    game_id: i64,
    name: &str,
) -> Result<ProfileRecord, AppError> {
    // If name exists, delete old profile (user already confirmed overwrite on frontend)
    if let Some(existing) = queries::get_profile_by_name(pool, game_id, name).await? {
        queries::delete_profile(pool, existing.id).await?;
    }

    // Insert new profile row
    let profile = queries::insert_profile(pool, game_id, name).await?;

    // Snapshot all mods for this game
    let mods = queries::list_mods_for_game(pool, game_id).await?;

    for mod_rec in &mods {
        let sub_mods = queries::list_sub_mods(pool, mod_rec.id).await?;
        let sub_states: Vec<SubModState> = sub_mods
            .iter()
            .map(|sm| SubModState {
                sub_mod_id: sm.id,
                enabled: sm.user_enabled, // Use user_enabled (user intent), not effective enabled
            })
            .collect();

        let sub_json = if sub_states.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&sub_states)?)
        };

        queries::insert_profile_entry(
            pool,
            profile.id,
            mod_rec.id,
            mod_rec.enabled,
            sub_json.as_deref(),
        )
        .await?;
    }

    Ok(profile)
}

/// Apply a saved profile: diff current state vs saved state, toggle mods to match.
/// Processes disables before enables to avoid spurious conflicts.
/// Returns IDs of mods that were skipped (deleted since profile was saved).
pub async fn apply_profile(
    app: &AppHandle,
    pool: &SqlitePool,
    profile_id: i64,
) -> Result<ApplyProfileResult, AppError> {
    let profile = queries::get_profile(pool, profile_id).await?;
    let entries = queries::list_profile_entries(pool, profile_id).await?;
    let current_mods = queries::list_mods_for_game(pool, profile.game_id).await?;

    let mut skipped_mods: Vec<String> = Vec::new();

    // Phase 1: Disables first (avoid spurious conflicts)
    for entry in &entries {
        match queries::get_mod(pool, entry.mod_id).await {
            Ok(current_mod) => {
                if current_mod.enabled && !entry.enabled {
                    toggle::toggle_mod(app, pool, entry.mod_id, false).await?;
                }
            }
            Err(_) => {
                // Mod was deleted since profile was saved
                skipped_mods.push(format!("mod_id={}", entry.mod_id));
            }
        }
    }

    // Also disable mods that exist now but are NOT in the profile (imported after save)
    for current_mod in &current_mods {
        let in_profile = entries.iter().any(|e| e.mod_id == current_mod.id);
        if !in_profile && current_mod.enabled {
            toggle::toggle_mod(app, pool, current_mod.id, false).await?;
        }
    }

    // Phase 2: Enables
    for entry in &entries {
        if !entry.enabled {
            continue;
        }
        match queries::get_mod(pool, entry.mod_id).await {
            Ok(current_mod) => {
                if !current_mod.enabled {
                    toggle::toggle_mod(app, pool, entry.mod_id, true).await?;
                }
            }
            Err(_) => {
                // Already tracked as skipped in Phase 1
            }
        }
    }

    // Phase 3: Sub-mod states
    for entry in &entries {
        if let Some(ref sub_states_json) = entry.sub_mod_states {
            // Only apply sub-mod states if the parent mod exists and is enabled
            if queries::get_mod(pool, entry.mod_id).await.is_ok() {
                let mod_rec = queries::get_mod(pool, entry.mod_id).await?;
                if mod_rec.enabled {
                    apply_sub_mod_states(app, pool, entry.mod_id, sub_states_json).await?;
                }
            }
        }
    }

    Ok(ApplyProfileResult { skipped_mods })
}

/// Parse sub-mod states JSON and toggle each sub-mod to match the saved state.
async fn apply_sub_mod_states(
    app: &AppHandle,
    pool: &SqlitePool,
    _mod_id: i64,
    sub_states_json: &str,
) -> Result<(), AppError> {
    let saved_states: Vec<SubModState> = serde_json::from_str(sub_states_json)?;

    for saved in &saved_states {
        // Check if sub-mod still exists
        match queries::get_sub_mod(pool, saved.sub_mod_id).await {
            Ok(current_sm) => {
                // Compare saved user intent vs current user_enabled
                if current_sm.user_enabled != saved.enabled {
                    // Only toggle if parent is enabled (toggle_sub_mod enforces this)
                    toggle::toggle_sub_mod(app, pool, saved.sub_mod_id, saved.enabled).await?;
                }
            }
            Err(_) => {
                // Sub-mod was deleted, skip silently
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::queries::{insert_game, insert_mod, insert_sub_mod, update_mod_enabled, update_sub_mod_enabled};

    /// Create an in-memory SQLite pool with all migrations applied.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory pool");

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("Failed to enable foreign keys");

        for migration in crate::db::migrations::get_migrations() {
            sqlx::raw_sql(migration.sql)
                .execute(&pool)
                .await
                .expect(&format!("Failed migration: {}", migration.description));
        }

        pool
    }

    #[tokio::test]
    async fn test_save_profile_snapshots_mods() {
        let pool = test_pool().await;
        let game = insert_game(&pool, "Tekken 8", "C:/game/Mods", "C:/staging", "structured", false)
            .await
            .unwrap();
        let m1 = insert_mod(&pool, game.id, "Mod A", "C:/staging/a").await.unwrap();
        let m2 = insert_mod(&pool, game.id, "Mod B", "C:/staging/b").await.unwrap();
        // Enable Mod A
        update_mod_enabled(&pool, m1.id, true).await.unwrap();

        // Add a sub-mod to Mod A with user_enabled=true
        let sm = insert_sub_mod(&pool, m1.id, "Option X", "Option_X").await.unwrap();
        update_sub_mod_enabled(&pool, sm.id, true, true).await.unwrap();

        let profile = save_profile(&pool, game.id, "Test Profile").await.unwrap();
        assert_eq!(profile.name, "Test Profile");

        let entries = queries::list_profile_entries(&pool, profile.id).await.unwrap();
        assert_eq!(entries.len(), 2, "Should have entries for both mods");

        // Find entries by mod_id
        let entry_a = entries.iter().find(|e| e.mod_id == m1.id).unwrap();
        let entry_b = entries.iter().find(|e| e.mod_id == m2.id).unwrap();
        assert!(entry_a.enabled, "Mod A was enabled");
        assert!(!entry_b.enabled, "Mod B was disabled");

        // Check sub-mod states were captured
        assert!(entry_a.sub_mod_states.is_some());
        let sub_states: Vec<SubModState> =
            serde_json::from_str(entry_a.sub_mod_states.as_ref().unwrap()).unwrap();
        assert_eq!(sub_states.len(), 1);
        assert_eq!(sub_states[0].sub_mod_id, sm.id);
        assert!(sub_states[0].enabled, "Should use user_enabled=true");
    }

    #[tokio::test]
    async fn test_save_profile_overwrites_existing() {
        let pool = test_pool().await;
        let game = insert_game(&pool, "Tekken 8", "C:/game/Mods", "C:/staging", "structured", false)
            .await
            .unwrap();
        let m1 = insert_mod(&pool, game.id, "Mod A", "C:/staging/a").await.unwrap();

        // Save first version
        let p1 = save_profile(&pool, game.id, "My Profile").await.unwrap();
        let entries1 = queries::list_profile_entries(&pool, p1.id).await.unwrap();
        assert_eq!(entries1.len(), 1);
        assert!(!entries1[0].enabled);

        // Enable mod, save again with same name
        update_mod_enabled(&pool, m1.id, true).await.unwrap();
        let p2 = save_profile(&pool, game.id, "My Profile").await.unwrap();

        // New profile replaces old
        assert_ne!(p1.id, p2.id);
        let entries2 = queries::list_profile_entries(&pool, p2.id).await.unwrap();
        assert_eq!(entries2.len(), 1);
        assert!(entries2[0].enabled, "New save should reflect enabled state");

        // Old profile entries should be gone
        let old_entries = queries::list_profile_entries(&pool, p1.id).await.unwrap();
        assert!(old_entries.is_empty());

        // Only one profile with this name
        let all = queries::list_profiles_for_game(&pool, game.id).await.unwrap();
        assert_eq!(all.len(), 1);
    }
}
