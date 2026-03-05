use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::db::queries::{self, ProfileRecord};
use crate::error::AppError;
use crate::services::profiles::{self, ApplyProfileResult};

#[tauri::command]
#[specta::specta]
pub async fn save_profile_cmd(
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
    name: String,
) -> Result<ProfileRecord, AppError> {
    profiles::save_profile(&pool, game_id, &name).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_profiles_cmd(
    pool: tauri::State<'_, SqlitePool>,
    game_id: i64,
) -> Result<Vec<ProfileRecord>, AppError> {
    queries::list_profiles_for_game(&pool, game_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_profile_cmd(
    pool: tauri::State<'_, SqlitePool>,
    profile_id: i64,
) -> Result<(), AppError> {
    queries::delete_profile(&pool, profile_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_profile_cmd(
    app: AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    profile_id: i64,
) -> Result<ApplyProfileResult, AppError> {
    profiles::apply_profile(&app, &pool, profile_id).await
}
