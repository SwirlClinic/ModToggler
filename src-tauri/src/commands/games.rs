use std::path::Path;
use sqlx::SqlitePool;

use crate::db::queries::{self, GameRecord};
use crate::error::AppError;
use crate::services::file_ops;

/// Return type for add_game and edit_game.
/// - cross_drive_warning: staging folder is on a different drive than mod_dir (slower moves)
/// - has_existing_mods: mod_dir already contains .pak/.ucas/.utoc files the user may want to import
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct AddGameResult {
    pub game: GameRecord,
    pub cross_drive_warning: bool,
    pub has_existing_mods: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn add_game(
    pool: tauri::State<'_, SqlitePool>,
    name: String,
    mod_dir: String,
    staging_dir: Option<String>,
    mod_structure: String,
) -> Result<AddGameResult, AppError> {
    // Compute staging dir: use provided override or default ~/.modtoggler/games/[slug]/staging/
    let staging = match staging_dir {
        Some(s) if !s.is_empty() => s,
        _ => {
            let slug = slug_from_name(&name);
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            format!("{}/.modtoggler/games/{}/staging", home, slug)
        }
    };

    // Detect if mod_dir is in a protected path (Program Files)
    let requires_elevation = is_protected_path(&mod_dir);

    // Detect cross-drive staging
    let cross_drive = !file_ops::same_volume(Path::new(&mod_dir), Path::new(&staging));

    // Scan mod_dir for existing .pak/.ucas/.utoc files the user might want to import.
    // Best-effort: if mod_dir doesn't exist yet, has_existing_mods = false.
    let has_existing_mods = scan_existing_mods(Path::new(&mod_dir)).await;

    // Create the staging directory on disk (idempotent)
    file_ops::create_staging_dir(Path::new(&staging)).await?;

    let game = queries::insert_game(
        &pool,
        &name,
        &mod_dir,
        &staging,
        &mod_structure,
        requires_elevation,
    )
    .await?;

    Ok(AddGameResult {
        game,
        cross_drive_warning: cross_drive,
        has_existing_mods,
    })
}

/// Scan a directory (non-recursively) for .pak, .ucas, or .utoc files.
/// Returns true if any are found, false if the directory is absent, empty, or has none.
pub(crate) async fn scan_existing_mods(mod_dir: &Path) -> bool {
    let mut entries = match tokio::fs::read_dir(mod_dir).await {
        Ok(e) => e,
        Err(_) => return false,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_ascii_lowercase();
            if matches!(ext_lower.as_str(), "pak" | "ucas" | "utoc") {
                return true;
            }
        }
    }
    false
}

#[tauri::command]
#[specta::specta]
pub async fn remove_game(
    pool: tauri::State<'_, SqlitePool>,
    id: i64,
) -> Result<(), AppError> {
    // Read game record first to get staging dir path before deleting
    let games = queries::list_games_db(&pool).await?;
    let game = games
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| AppError::GameNotFound(format!("Game ID {} not found", id)))?;

    // Delete from DB (cascades to mods and file_entries)
    queries::delete_game(&pool, id).await?;

    // Remove staging directory -- best effort, don't fail if already gone
    let staging_path = Path::new(&game.staging_dir);
    if staging_path.exists() {
        let _ = tokio::fs::remove_dir_all(staging_path).await;
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn edit_game(
    pool: tauri::State<'_, SqlitePool>,
    id: i64,
    name: String,
    mod_dir: String,
    staging_dir: String,
    mod_structure: String,
) -> Result<AddGameResult, AppError> {
    let requires_elevation = is_protected_path(&mod_dir);
    let cross_drive = !file_ops::same_volume(Path::new(&mod_dir), Path::new(&staging_dir));

    // Ensure staging dir exists (may have changed)
    file_ops::create_staging_dir(Path::new(&staging_dir)).await?;

    let game = queries::update_game(
        &pool,
        id,
        &name,
        &mod_dir,
        &staging_dir,
        &mod_structure,
        requires_elevation,
    )
    .await?;

    // For edit, has_existing_mods reflects current state of the (possibly new) mod_dir
    let has_existing_mods = scan_existing_mods(Path::new(&mod_dir)).await;

    Ok(AddGameResult {
        game,
        cross_drive_warning: cross_drive,
        has_existing_mods,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_games(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<GameRecord>, AppError> {
    queries::list_games_db(&pool).await
}

/// Convert a game name to a filesystem-safe slug for staging dir path.
fn slug_from_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Check if a path is inside a Windows protected directory (Program Files).
fn is_protected_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("program files") || lower.contains("program files (x86)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_from_name_basic() {
        assert_eq!(slug_from_name("Tekken 8"), "tekken-8");
    }

    #[test]
    fn test_slug_from_name_special_chars() {
        assert_eq!(slug_from_name("My Game: Edition"), "my-game--edition");
    }

    #[test]
    fn test_is_protected_path_program_files() {
        assert!(is_protected_path(
            "C:/Program Files (x86)/Steam/steamapps/common/TEKKEN 8/Mods"
        ));
        assert!(is_protected_path("C:/Program Files/SomeGame/Mods"));
    }

    #[test]
    fn test_is_protected_path_not_protected() {
        assert!(!is_protected_path("C:/Games/Tekken8/Mods"));
        assert!(!is_protected_path(
            "D:/SteamLibrary/steamapps/common/TEKKEN 8/Mods"
        ));
    }

    #[tokio::test]
    async fn test_scan_existing_mods_finds_pak() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pak_path = dir.path().join("testmod.pak");
        tokio::fs::write(&pak_path, b"fake pak data").await.unwrap();
        assert!(
            scan_existing_mods(dir.path()).await,
            "Should detect .pak file"
        );
    }

    #[tokio::test]
    async fn test_scan_existing_mods_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(
            !scan_existing_mods(dir.path()).await,
            "Empty dir should return false"
        );
    }

    #[tokio::test]
    async fn test_scan_existing_mods_nonexistent_dir() {
        let path = std::path::PathBuf::from("C:/this/path/does/not/exist/at/all");
        assert!(
            !scan_existing_mods(&path).await,
            "Non-existent dir should return false"
        );
    }
}
