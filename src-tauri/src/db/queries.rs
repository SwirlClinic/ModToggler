use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use crate::error::AppError;
use crate::services::journal::{deserialize_files, IncompleteJournalEntry};

// ─── Record Types (specta::Type required for tauri-specta bindings generation) ───

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct GameRecord {
    pub id: i64,
    pub name: String,
    pub mod_dir: String,
    pub staging_dir: String,
    pub mod_structure: String, // "structured" | "loose"
    pub requires_elevation: bool,
}

impl<'r> FromRow<'r, SqliteRow> for GameRecord {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(GameRecord {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            mod_dir: row.try_get("mod_dir")?,
            staging_dir: row.try_get("staging_dir")?,
            mod_structure: row.try_get("mod_structure")?,
            requires_elevation: {
                let val: i64 = row.try_get("requires_elevation")?;
                val != 0
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ModRecord {
    pub id: i64,
    pub game_id: i64,
    pub name: String,
    pub enabled: bool,
    pub staged_path: String,
}

impl<'r> FromRow<'r, SqliteRow> for ModRecord {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(ModRecord {
            id: row.try_get("id")?,
            game_id: row.try_get("game_id")?,
            name: row.try_get("name")?,
            enabled: {
                let val: i64 = row.try_get("enabled")?;
                val != 0
            },
            staged_path: row.try_get("staged_path")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct FileEntry {
    pub id: i64,
    pub mod_id: i64,
    pub relative_path: String,
}

impl<'r> FromRow<'r, SqliteRow> for FileEntry {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(FileEntry {
            id: row.try_get("id")?,
            mod_id: row.try_get("mod_id")?,
            relative_path: row.try_get("relative_path")?,
        })
    }
}

#[derive(Debug, Serialize, Type)]
pub struct IntegrityScanResult {
    pub missing_from_game: Vec<ModRecord>,
    pub missing_from_staging: Vec<ModRecord>,
    pub incomplete_journals: Vec<IncompleteJournalEntry>,
}

// ─── Game Queries ───

pub async fn insert_game(
    pool: &SqlitePool,
    name: &str,
    mod_dir: &str,
    staging_dir: &str,
    mod_structure: &str,
    requires_elevation: bool,
) -> Result<GameRecord, AppError> {
    let elev: i64 = if requires_elevation { 1 } else { 0 };
    let result = sqlx::query(
        "INSERT INTO games (name, mod_dir, staging_dir, mod_structure, requires_elevation) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(name)
    .bind(mod_dir)
    .bind(staging_dir)
    .bind(mod_structure)
    .bind(elev)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result.last_insert_rowid();

    Ok(GameRecord {
        id,
        name: name.to_string(),
        mod_dir: mod_dir.to_string(),
        staging_dir: staging_dir.to_string(),
        mod_structure: mod_structure.to_string(),
        requires_elevation,
    })
}

pub async fn list_games_db(pool: &SqlitePool) -> Result<Vec<GameRecord>, AppError> {
    let rows = sqlx::query_as::<_, GameRecord>(
        "SELECT id, name, mod_dir, staging_dir, mod_structure, requires_elevation FROM games ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn update_game(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    mod_dir: &str,
    staging_dir: &str,
    mod_structure: &str,
    requires_elevation: bool,
) -> Result<GameRecord, AppError> {
    let elev: i64 = if requires_elevation { 1 } else { 0 };
    let result = sqlx::query(
        "UPDATE games SET name=$1, mod_dir=$2, staging_dir=$3, mod_structure=$4, requires_elevation=$5 WHERE id=$6",
    )
    .bind(name)
    .bind(mod_dir)
    .bind(staging_dir)
    .bind(mod_structure)
    .bind(elev)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::GameNotFound(format!("Game ID {} not found", id)));
    }

    Ok(GameRecord {
        id,
        name: name.to_string(),
        mod_dir: mod_dir.to_string(),
        staging_dir: staging_dir.to_string(),
        mod_structure: mod_structure.to_string(),
        requires_elevation,
    })
}

pub async fn delete_game(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM games WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::GameNotFound(format!("Game ID {} not found", id)));
    }
    Ok(())
}

// ─── Mod Queries (used by integrity scan) ───

pub async fn list_all_mods(pool: &SqlitePool) -> Result<Vec<ModRecord>, AppError> {
    let rows = sqlx::query_as::<_, ModRecord>(
        "SELECT id, game_id, name, enabled, staged_path FROM mods",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn list_file_entries(pool: &SqlitePool, mod_id: i64) -> Result<Vec<FileEntry>, AppError> {
    let rows = sqlx::query_as::<_, FileEntry>(
        "SELECT id, mod_id, relative_path FROM file_entries WHERE mod_id=$1",
    )
    .bind(mod_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

// ─── Journal Queries ───

pub async fn scan_incomplete_journals(
    pool: &SqlitePool,
) -> Result<Vec<IncompleteJournalEntry>, AppError> {
    let rows: Vec<(i64, i64, String, String)> = sqlx::query_as(
        "SELECT id, mod_id, operation, files_json FROM toggle_journal WHERE status='in_progress'",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    rows.into_iter()
        .map(|(id, mod_id, operation, files_json)| {
            let files = deserialize_files(&files_json)?;
            Ok(IncompleteJournalEntry {
                id,
                mod_id,
                operation,
                files,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_record_is_serializable() {
        let rec = GameRecord {
            id: 1,
            name: "Tekken 8".into(),
            mod_dir: "C:/game/Mods".into(),
            staging_dir: "C:/staging".into(),
            mod_structure: "structured".into(),
            requires_elevation: false,
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("Tekken 8"));
    }

    #[test]
    fn mod_record_is_serializable() {
        let rec = ModRecord {
            id: 1,
            game_id: 1,
            name: "Cool Mod".into(),
            enabled: true,
            staged_path: "C:/staging/cool-mod".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("Cool Mod"));
        assert!(json.contains("true"));
    }

    #[test]
    fn integrity_scan_result_is_serializable() {
        let result = IntegrityScanResult {
            missing_from_game: vec![],
            missing_from_staging: vec![],
            incomplete_journals: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("missing_from_game"));
    }
}
