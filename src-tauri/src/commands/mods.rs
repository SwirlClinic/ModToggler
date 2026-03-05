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

/// Input for a single loose file: where it comes from, where it goes in the game tree, and its name.
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct LooseFileInput {
    pub source_path: String,      // absolute path to source file
    pub destination_path: String,  // relative dir under game root (e.g., "bin/scripts", "/" for root)
    pub file_name: String,         // original filename
}

/// Import loose files into a new mod: copy to staging, create DB records.
#[tauri::command]
#[specta::specta]
pub async fn import_loose_files(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    mod_name: String,
    files: Vec<LooseFileInput>,
) -> Result<ImportResult, AppError> {
    let game = queries::get_game(&pool, game_id).await?;

    // Create mod-specific staging subdir
    let slug = slug_from_name(&mod_name);
    let mod_staging = PathBuf::from(&game.staging_dir).join(&slug);
    std::fs::create_dir_all(&mod_staging)?;

    // Build source pairs for copy_files_to_staging: (absolute_source, desired_filename)
    let source_pairs: Vec<(String, String)> = files
        .iter()
        .map(|f| (f.source_path.clone(), f.file_name.clone()))
        .collect();

    let staging_clone = mod_staging.clone();
    let actual_names = tokio::task::spawn_blocking(move || {
        import::copy_files_to_staging(&source_pairs, &staging_clone)
    })
    .await
    .map_err(|e| AppError::IoError(format!("Spawn blocking failed: {}", e)))??;

    // Insert mod record with mod_type="loose"
    let mod_record = queries::insert_mod_with_type(
        &pool,
        game_id,
        &mod_name,
        &mod_staging.display().to_string(),
        "loose",
    )
    .await?;

    // Insert file_entries with destination_path for each file
    for (i, actual_name) in actual_names.iter().enumerate() {
        let dest = &files[i].destination_path;
        queries::insert_file_entry_with_destination(
            &pool,
            mod_record.id,
            actual_name,
            None,
            Some(dest),
        )
        .await?;
    }

    let _ = app;

    Ok(ImportResult {
        mod_record,
        file_count: actual_names.len(),
        sub_mods: Vec::new(),
        has_recognized_files: false,
    })
}

/// Import loose files from a zip: extract, copy selected files to staging, create DB records.
#[tauri::command]
#[specta::specta]
pub async fn import_loose_zip(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    zip_path: String,
    mod_name: String,
    selected_files: Vec<LooseFileInput>,
) -> Result<ImportResult, AppError> {
    let game = queries::get_game(&pool, game_id).await?;

    // Create mod-specific staging subdir
    let slug = slug_from_name(&mod_name);
    let mod_staging = PathBuf::from(&game.staging_dir).join(&slug);
    std::fs::create_dir_all(&mod_staging)?;

    // Extract zip to a temporary directory first
    let temp_path = std::env::temp_dir().join(format!("modtoggler_zip_{}", std::process::id()));
    std::fs::create_dir_all(&temp_path)?;
    let zip = PathBuf::from(&zip_path);

    tokio::task::spawn_blocking(move || {
        import::extract_zip_to_staging(&zip, &temp_path)
    })
    .await
    .map_err(|e| AppError::IoError(format!("Spawn blocking failed: {}", e)))??;

    // Build source pairs from selected_files: source_path points to extracted file in temp dir
    let source_pairs: Vec<(String, String)> = selected_files
        .iter()
        .map(|f| (f.source_path.clone(), f.file_name.clone()))
        .collect();

    let staging_clone = mod_staging.clone();
    let actual_names = tokio::task::spawn_blocking(move || {
        import::copy_files_to_staging(&source_pairs, &staging_clone)
    })
    .await
    .map_err(|e| AppError::IoError(format!("Spawn blocking failed: {}", e)))??;

    // Insert mod record with mod_type="loose"
    let mod_record = queries::insert_mod_with_type(
        &pool,
        game_id,
        &mod_name,
        &mod_staging.display().to_string(),
        "loose",
    )
    .await?;

    // Insert file_entries with destination_path
    for (i, actual_name) in actual_names.iter().enumerate() {
        let dest = &selected_files[i].destination_path;
        queries::insert_file_entry_with_destination(
            &pool,
            mod_record.id,
            actual_name,
            None,
            Some(dest),
        )
        .await?;
    }

    let _ = app;

    Ok(ImportResult {
        mod_record,
        file_count: actual_names.len(),
        sub_mods: Vec::new(),
        has_recognized_files: false,
    })
}

/// Add files to an existing loose-file mod.
#[tauri::command]
#[specta::specta]
pub async fn add_files_to_mod(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    mod_id: i64,
    files: Vec<LooseFileInput>,
) -> Result<usize, AppError> {
    let mod_record = queries::get_mod(&pool, mod_id).await?;

    // Verify this is a loose mod
    if mod_record.mod_type != "loose" {
        return Err(AppError::IoError(
            "Cannot add individual files to a structured mod".to_string(),
        ));
    }

    let mod_staging = PathBuf::from(&mod_record.staged_path);

    // Build source pairs
    let source_pairs: Vec<(String, String)> = files
        .iter()
        .map(|f| (f.source_path.clone(), f.file_name.clone()))
        .collect();

    let staging_clone = mod_staging.clone();
    let actual_names = tokio::task::spawn_blocking(move || {
        import::copy_files_to_staging(&source_pairs, &staging_clone)
    })
    .await
    .map_err(|e| AppError::IoError(format!("Spawn blocking failed: {}", e)))??;

    // Insert new file entries
    for (i, actual_name) in actual_names.iter().enumerate() {
        let dest = &files[i].destination_path;
        queries::insert_file_entry_with_destination(
            &pool,
            mod_record.id,
            actual_name,
            None,
            Some(dest),
        )
        .await?;
    }

    let _ = app;

    Ok(actual_names.len())
}

/// Remove a single file from a loose-file mod (deletes from staging/game dir and DB).
#[tauri::command]
#[specta::specta]
pub async fn remove_file_from_mod(
    pool: tauri::State<'_, SqlitePool>,
    file_entry_id: i64,
) -> Result<(), AppError> {
    // Get the file entry
    let pool_ref: &SqlitePool = &pool;
    let entry: FileEntry = sqlx::query_as::<_, FileEntry>(
        "SELECT id, mod_id, relative_path, sub_mod_id, destination_path FROM file_entries WHERE id=$1",
    )
    .bind(file_entry_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ModNotFound(format!("File entry ID {} not found", file_entry_id)))?;

    // Get the mod record to find staging path and check if enabled
    let mod_record = queries::get_mod(&pool, entry.mod_id).await?;

    if mod_record.enabled {
        // File is in the game directory — compute game-side path
        let game = queries::get_game(&pool, mod_record.game_id).await?;
        let dest_dir = entry.destination_path.as_deref().unwrap_or("/");
        let game_path = if dest_dir == "/" {
            PathBuf::from(&game.mod_dir).join(&entry.relative_path)
        } else {
            PathBuf::from(&game.mod_dir).join(dest_dir).join(&entry.relative_path)
        };
        if game_path.exists() {
            std::fs::remove_file(&game_path)?;
        }
    } else {
        // File is in staging
        let staging_path = PathBuf::from(&mod_record.staged_path).join(&entry.relative_path);
        if staging_path.exists() {
            std::fs::remove_file(&staging_path)?;
        }
    }

    // Delete DB record
    queries::delete_file_entry(&pool, file_entry_id).await?;

    Ok(())
}

/// Convert a mod name to a filesystem-safe slug for staging subdir path.
pub(crate) fn slug_from_name(name: &str) -> String {
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
