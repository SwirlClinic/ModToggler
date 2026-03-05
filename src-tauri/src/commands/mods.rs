use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::db::queries::{self, ConflictInfo, FileEntry, ModRecord, SubModRecord};
use crate::error::AppError;
use crate::services::import;
use crate::services::toggle;

/// Result of importing a mod from a zip archive.
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ImportResult {
    pub mod_record: ModRecord,
    pub file_count: usize,
    pub sub_mods: Vec<SubModRecord>,
    pub has_recognized_files: bool,
}

/// Import a mod from a zip archive: extract to staging, create DB records, return result.
#[tauri::command]
#[specta::specta]
pub async fn import_mod(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    zip_path: String,
    mod_name: String,
) -> Result<ImportResult, AppError> {
    // 1. Get game record for staging_dir
    let game = queries::get_game(&pool, game_id).await?;

    // 2. Create mod-specific staging subdir
    let slug = slug_from_name(&mod_name);
    let mod_staging = PathBuf::from(&game.staging_dir).join(&slug);
    std::fs::create_dir_all(&mod_staging)?;

    // 3. Extract zip via spawn_blocking (zip crate is sync)
    let zip = PathBuf::from(&zip_path);
    let staging_clone = mod_staging.clone();
    let manifest = tokio::task::spawn_blocking(move || {
        import::extract_zip_to_staging(&zip, &staging_clone)
    })
    .await
    .map_err(|e| AppError::IoError(format!("Spawn blocking failed: {}", e)))??;

    // 4. Partition files into main and sub-mods
    let (main_files, sub_mod_map) = import::partition_files(&manifest);

    // 5. Check for recognized mod files
    let has_recognized_files = import::has_recognized_mod_files(&manifest);

    // 6. Insert mod record (enabled=false, staged_path=mod subdir)
    let mod_record = queries::insert_mod(
        &pool,
        game_id,
        &mod_name,
        &mod_staging.display().to_string(),
    )
    .await?;

    // 7. Insert file entries for main files (sub_mod_id=None)
    for path in &main_files {
        queries::insert_file_entry(&pool, mod_record.id, path, None).await?;
    }

    // 8. For each sub-mod folder: insert sub_mod record + file entries
    let mut sub_mods = Vec::new();
    for (folder_name, files) in &sub_mod_map {
        let sm = queries::insert_sub_mod(&pool, mod_record.id, folder_name, folder_name).await?;
        for file_path in files {
            // Store with the Option_ prefix so relative_path matches disk layout
            let full_rel = format!("{}/{}", folder_name, file_path);
            queries::insert_file_entry(&pool, mod_record.id, &full_rel, Some(sm.id)).await?;
        }
        sub_mods.push(sm);
    }

    let file_count = manifest.len();

    // Suppress unused variable warning -- app handle available for future use (e.g., progress events)
    let _ = app;

    Ok(ImportResult {
        mod_record,
        file_count,
        sub_mods,
        has_recognized_files,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_mods(
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
) -> Result<Vec<ModRecord>, AppError> {
    queries::list_mods_for_game(&pool, game_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_mod_files(
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
) -> Result<Vec<FileEntry>, AppError> {
    queries::list_file_entries(&pool, mod_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_sub_mods_cmd(
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
) -> Result<Vec<SubModRecord>, AppError> {
    queries::list_sub_mods(&pool, mod_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_mod_cmd(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
    enable: bool,
) -> Result<(), AppError> {
    toggle::toggle_mod(&app, &pool, mod_id, enable).await
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_sub_mod_cmd(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    sub_mod_id: i64,
    enable: bool,
) -> Result<(), AppError> {
    toggle::toggle_sub_mod(&app, &pool, sub_mod_id, enable).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_mod_cmd(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
) -> Result<(), AppError> {
    toggle::delete_mod(&app, &pool, mod_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn check_conflicts_cmd(
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
    game_id: i64,
) -> Result<Vec<ConflictInfo>, AppError> {
    queries::check_conflicts(&pool, mod_id, game_id).await
}

/// Convert a mod name to a filesystem-safe slug for staging subdir path.
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
