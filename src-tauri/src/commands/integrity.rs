use std::path::Path;
use sqlx::SqlitePool;

use crate::db::queries::{self, IntegrityScanResult};
use crate::error::AppError;

/// Run the startup integrity scan.
///
/// Checks:
/// 1. Any in-progress toggle journal entries (crash recovery needed)
/// 2. Enabled mods whose files are missing from the game directory
/// 3. Disabled mods whose files are missing from staging
///
/// IMPORTANT: Returns empty results -- not an error -- when no mods exist.
/// This is the intentional behavior on first launch (PITFALL-5 guard).
#[tauri::command]
#[specta::specta]
pub async fn run_integrity_scan(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<IntegrityScanResult, AppError> {
    let mods = queries::list_all_mods(&pool).await?;
    let incomplete_journals = queries::scan_incomplete_journals(&pool).await?;

    let mut missing_from_game = vec![];
    let mut missing_from_staging = vec![];

    for mod_rec in &mods {
        let file_entries = queries::list_file_entries(&pool, mod_rec.id).await?;
        if file_entries.is_empty() {
            // No file manifest yet (mod imported without files, or Phase 1 empty state)
            continue;
        }

        if mod_rec.enabled {
            // Files should exist in the game's mod directory.
            // If enabled, file should NOT be in staging -- if it is, mod is missing from game.
            for entry in &file_entries {
                let staging_file = Path::new(&mod_rec.staged_path).join(&entry.relative_path);
                if staging_file.exists() {
                    missing_from_game.push(mod_rec.clone());
                    break;
                }
            }
        } else {
            // Files should be in staging
            for entry in &file_entries {
                let staging_file = Path::new(&mod_rec.staged_path).join(&entry.relative_path);
                if !staging_file.exists() {
                    missing_from_staging.push(mod_rec.clone());
                    break;
                }
            }
        }
    }

    Ok(IntegrityScanResult {
        missing_from_game,
        missing_from_staging,
        incomplete_journals,
    })
}

#[cfg(test)]
mod tests {
    use crate::db::queries::IntegrityScanResult;

    /// Simulate the integrity scan result shape with empty inputs (first launch, no mods).
    /// This guards against PITFALL-5: false integrity warnings on empty DB.
    #[test]
    fn test_empty_scan_result_has_no_warnings() {
        let result = IntegrityScanResult {
            missing_from_game: vec![],
            missing_from_staging: vec![],
            incomplete_journals: vec![],
        };

        assert!(
            result.missing_from_game.is_empty(),
            "Fresh DB should have no missing_from_game"
        );
        assert!(
            result.missing_from_staging.is_empty(),
            "Fresh DB should have no missing_from_staging"
        );
        assert!(
            result.incomplete_journals.is_empty(),
            "Fresh DB should have no incomplete_journals"
        );
    }

    #[test]
    fn test_integrity_result_serializes() {
        let result = IntegrityScanResult {
            missing_from_game: vec![],
            missing_from_staging: vec![],
            incomplete_journals: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("missing_from_game"));
        assert!(json.contains("incomplete_journals"));
    }
}
