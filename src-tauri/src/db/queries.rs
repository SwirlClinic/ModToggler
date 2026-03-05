use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

use crate::error::AppError;
use crate::services::journal::{deserialize_files, serialize_files, FilePair, IncompleteJournalEntry};

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
    pub sub_mod_id: Option<i64>,
}

impl<'r> FromRow<'r, SqliteRow> for FileEntry {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(FileEntry {
            id: row.try_get("id")?,
            mod_id: row.try_get("mod_id")?,
            relative_path: row.try_get("relative_path")?,
            sub_mod_id: row.try_get("sub_mod_id")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct SubModRecord {
    pub id: i64,
    pub mod_id: i64,
    pub name: String,
    pub folder_name: String,
    pub enabled: bool,
    pub user_enabled: bool,
}

impl<'r> FromRow<'r, SqliteRow> for SubModRecord {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(SubModRecord {
            id: row.try_get("id")?,
            mod_id: row.try_get("mod_id")?,
            name: row.try_get("name")?,
            folder_name: row.try_get("folder_name")?,
            enabled: {
                let val: i64 = row.try_get("enabled")?;
                val != 0
            },
            user_enabled: {
                let val: i64 = row.try_get("user_enabled")?;
                val != 0
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ConflictInfo {
    pub conflicting_mod_id: i64,
    pub conflicting_mod_name: String,
    pub relative_path: String,
}

impl<'r> FromRow<'r, SqliteRow> for ConflictInfo {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(ConflictInfo {
            conflicting_mod_id: row.try_get("conflicting_mod_id")?,
            conflicting_mod_name: row.try_get("conflicting_mod_name")?,
            relative_path: row.try_get("relative_path")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ProfileRecord {
    pub id: i64,
    pub game_id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl<'r> FromRow<'r, SqliteRow> for ProfileRecord {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(ProfileRecord {
            id: row.try_get("id")?,
            game_id: row.try_get("game_id")?,
            name: row.try_get("name")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
pub struct ProfileEntryRecord {
    pub id: i64,
    pub profile_id: i64,
    pub mod_id: i64,
    pub enabled: bool,
    pub sub_mod_states: Option<String>,
}

impl<'r> FromRow<'r, SqliteRow> for ProfileEntryRecord {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(ProfileEntryRecord {
            id: row.try_get("id")?,
            profile_id: row.try_get("profile_id")?,
            mod_id: row.try_get("mod_id")?,
            enabled: {
                let val: i64 = row.try_get("enabled")?;
                val != 0
            },
            sub_mod_states: row.try_get("sub_mod_states")?,
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

// ─── Game Queries (get by ID) ───

pub async fn get_game(pool: &SqlitePool, game_id: i64) -> Result<GameRecord, AppError> {
    sqlx::query_as::<_, GameRecord>(
        "SELECT id, name, mod_dir, staging_dir, mod_structure, requires_elevation FROM games WHERE id=$1",
    )
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::GameNotFound(format!("Game ID {} not found", game_id)))
}

// ─── Mod Queries ───

pub async fn list_all_mods(pool: &SqlitePool) -> Result<Vec<ModRecord>, AppError> {
    let rows = sqlx::query_as::<_, ModRecord>(
        "SELECT id, game_id, name, enabled, staged_path FROM mods",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn insert_mod(
    pool: &SqlitePool,
    game_id: i64,
    name: &str,
    staged_path: &str,
) -> Result<ModRecord, AppError> {
    let result = sqlx::query(
        "INSERT INTO mods (game_id, name, staged_path) VALUES ($1, $2, $3)",
    )
    .bind(game_id)
    .bind(name)
    .bind(staged_path)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result.last_insert_rowid();

    Ok(ModRecord {
        id,
        game_id,
        name: name.to_string(),
        enabled: false,
        staged_path: staged_path.to_string(),
    })
}

pub async fn get_mod(pool: &SqlitePool, mod_id: i64) -> Result<ModRecord, AppError> {
    sqlx::query_as::<_, ModRecord>(
        "SELECT id, game_id, name, enabled, staged_path FROM mods WHERE id=$1",
    )
    .bind(mod_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ModNotFound(format!("Mod ID {} not found", mod_id)))
}

pub async fn list_mods_for_game(
    pool: &SqlitePool,
    game_id: i64,
) -> Result<Vec<ModRecord>, AppError> {
    let rows = sqlx::query_as::<_, ModRecord>(
        "SELECT id, game_id, name, enabled, staged_path FROM mods WHERE game_id=$1 ORDER BY enabled DESC, name ASC",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn update_mod_enabled(
    pool: &SqlitePool,
    mod_id: i64,
    enabled: bool,
) -> Result<(), AppError> {
    let val: i64 = if enabled { 1 } else { 0 };
    sqlx::query("UPDATE mods SET enabled=$1 WHERE id=$2")
        .bind(val)
        .bind(mod_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub async fn delete_mod_db(pool: &SqlitePool, mod_id: i64) -> Result<(), AppError> {
    // toggle_journal FK on mod_id has no CASCADE — delete journal entries first
    sqlx::query("DELETE FROM toggle_journal WHERE mod_id=$1")
        .bind(mod_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    sqlx::query("DELETE FROM mods WHERE id=$1")
        .bind(mod_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

// ─── File Entry Queries ───

pub async fn list_file_entries(pool: &SqlitePool, mod_id: i64) -> Result<Vec<FileEntry>, AppError> {
    let rows = sqlx::query_as::<_, FileEntry>(
        "SELECT id, mod_id, relative_path, sub_mod_id FROM file_entries WHERE mod_id=$1",
    )
    .bind(mod_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn list_file_entries_for_sub_mod(
    pool: &SqlitePool,
    sub_mod_id: i64,
) -> Result<Vec<FileEntry>, AppError> {
    let rows = sqlx::query_as::<_, FileEntry>(
        "SELECT id, mod_id, relative_path, sub_mod_id FROM file_entries WHERE sub_mod_id=$1",
    )
    .bind(sub_mod_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn insert_file_entry(
    pool: &SqlitePool,
    mod_id: i64,
    relative_path: &str,
    sub_mod_id: Option<i64>,
) -> Result<FileEntry, AppError> {
    let result = sqlx::query(
        "INSERT INTO file_entries (mod_id, relative_path, sub_mod_id) VALUES ($1, $2, $3)",
    )
    .bind(mod_id)
    .bind(relative_path)
    .bind(sub_mod_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result.last_insert_rowid();

    Ok(FileEntry {
        id,
        mod_id,
        relative_path: relative_path.to_string(),
        sub_mod_id,
    })
}

// ─── Sub-Mod Queries ───

pub async fn insert_sub_mod(
    pool: &SqlitePool,
    mod_id: i64,
    name: &str,
    folder_name: &str,
) -> Result<SubModRecord, AppError> {
    let result = sqlx::query(
        "INSERT INTO sub_mods (mod_id, name, folder_name) VALUES ($1, $2, $3)",
    )
    .bind(mod_id)
    .bind(name)
    .bind(folder_name)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result.last_insert_rowid();

    Ok(SubModRecord {
        id,
        mod_id,
        name: name.to_string(),
        folder_name: folder_name.to_string(),
        enabled: false,
        user_enabled: false,
    })
}

pub async fn get_sub_mod(pool: &SqlitePool, sub_mod_id: i64) -> Result<SubModRecord, AppError> {
    sqlx::query_as::<_, SubModRecord>(
        "SELECT id, mod_id, name, folder_name, enabled, user_enabled FROM sub_mods WHERE id=$1",
    )
    .bind(sub_mod_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ModNotFound(format!("Sub-mod ID {} not found", sub_mod_id)))
}

pub async fn list_sub_mods(
    pool: &SqlitePool,
    mod_id: i64,
) -> Result<Vec<SubModRecord>, AppError> {
    let rows = sqlx::query_as::<_, SubModRecord>(
        "SELECT id, mod_id, name, folder_name, enabled, user_enabled FROM sub_mods WHERE mod_id=$1",
    )
    .bind(mod_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn update_sub_mod_enabled(
    pool: &SqlitePool,
    sub_mod_id: i64,
    enabled: bool,
    user_enabled: bool,
) -> Result<(), AppError> {
    let en: i64 = if enabled { 1 } else { 0 };
    let ue: i64 = if user_enabled { 1 } else { 0 };
    sqlx::query("UPDATE sub_mods SET enabled=$1, user_enabled=$2 WHERE id=$3")
        .bind(en)
        .bind(ue)
        .bind(sub_mod_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

// ─── Conflict Detection ───

pub async fn check_conflicts(
    pool: &SqlitePool,
    mod_id: i64,
    game_id: i64,
) -> Result<Vec<ConflictInfo>, AppError> {
    let rows = sqlx::query_as::<_, ConflictInfo>(
        "SELECT
            fe_other.mod_id AS conflicting_mod_id,
            m_other.name AS conflicting_mod_name,
            fe_target.relative_path
        FROM file_entries fe_target
        JOIN file_entries fe_other
            ON fe_target.relative_path = fe_other.relative_path
            AND fe_target.mod_id != fe_other.mod_id
        JOIN mods m_other ON fe_other.mod_id = m_other.id
        WHERE fe_target.mod_id = $1
            AND m_other.game_id = $2
            AND m_other.enabled = 1
            AND (fe_other.sub_mod_id IS NULL OR EXISTS (
                SELECT 1 FROM sub_mods sm WHERE sm.id = fe_other.sub_mod_id AND sm.enabled = 1
            ))
        ORDER BY fe_target.relative_path",
    )
    .bind(mod_id)
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

// ─── Journal Queries (async DB functions) ───

pub async fn begin_toggle(
    pool: &SqlitePool,
    mod_id: i64,
    operation: &str,
    files: &[FilePair],
) -> Result<i64, AppError> {
    let files_json = serialize_files(files)?;
    let result = sqlx::query(
        "INSERT INTO toggle_journal (mod_id, operation, status, files_json) VALUES ($1, $2, 'in_progress', $3)",
    )
    .bind(mod_id)
    .bind(operation)
    .bind(&files_json)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

pub async fn mark_file_done(
    pool: &SqlitePool,
    journal_id: i64,
    file_index: usize,
) -> Result<(), AppError> {
    // Read-modify-write the files_json blob
    let row: (String,) = sqlx::query_as(
        "SELECT files_json FROM toggle_journal WHERE id=$1",
    )
    .bind(journal_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut files = deserialize_files(&row.0)?;
    if file_index >= files.len() {
        return Err(AppError::JournalCorrupt(format!(
            "File index {} out of bounds (journal has {} files)",
            file_index,
            files.len()
        )));
    }
    files[file_index].done = true;
    let updated_json = serialize_files(&files)?;

    sqlx::query("UPDATE toggle_journal SET files_json=$1 WHERE id=$2")
        .bind(&updated_json)
        .bind(journal_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn complete_journal(pool: &SqlitePool, journal_id: i64) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE toggle_journal SET status='done', completed_at=unixepoch() WHERE id=$1",
    )
    .bind(journal_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
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

// ─── Profile Queries ───

pub async fn insert_profile(
    pool: &SqlitePool,
    game_id: i64,
    name: &str,
) -> Result<ProfileRecord, AppError> {
    let result = sqlx::query(
        "INSERT INTO profiles (game_id, name) VALUES ($1, $2)",
    )
    .bind(game_id)
    .bind(name)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result.last_insert_rowid();

    // Fetch the full record to get server-generated defaults
    get_profile(pool, id).await
}

pub async fn list_profiles_for_game(
    pool: &SqlitePool,
    game_id: i64,
) -> Result<Vec<ProfileRecord>, AppError> {
    let rows = sqlx::query_as::<_, ProfileRecord>(
        "SELECT id, game_id, name, created_at, updated_at FROM profiles WHERE game_id=$1 ORDER BY name ASC",
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

pub async fn get_profile(pool: &SqlitePool, profile_id: i64) -> Result<ProfileRecord, AppError> {
    sqlx::query_as::<_, ProfileRecord>(
        "SELECT id, game_id, name, created_at, updated_at FROM profiles WHERE id=$1",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::ModNotFound(format!("Profile ID {} not found", profile_id)))
}

pub async fn get_profile_by_name(
    pool: &SqlitePool,
    game_id: i64,
    name: &str,
) -> Result<Option<ProfileRecord>, AppError> {
    sqlx::query_as::<_, ProfileRecord>(
        "SELECT id, game_id, name, created_at, updated_at FROM profiles WHERE game_id=$1 AND name=$2",
    )
    .bind(game_id)
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub async fn delete_profile(pool: &SqlitePool, profile_id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM profiles WHERE id=$1")
        .bind(profile_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub async fn insert_profile_entry(
    pool: &SqlitePool,
    profile_id: i64,
    mod_id: i64,
    enabled: bool,
    sub_mod_states: Option<&str>,
) -> Result<(), AppError> {
    let en: i64 = if enabled { 1 } else { 0 };
    sqlx::query(
        "INSERT INTO profile_entries (profile_id, mod_id, enabled, sub_mod_states) VALUES ($1, $2, $3, $4)",
    )
    .bind(profile_id)
    .bind(mod_id)
    .bind(en)
    .bind(sub_mod_states)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub async fn list_profile_entries(
    pool: &SqlitePool,
    profile_id: i64,
) -> Result<Vec<ProfileEntryRecord>, AppError> {
    let rows = sqlx::query_as::<_, ProfileEntryRecord>(
        "SELECT id, profile_id, mod_id, enabled, sub_mod_states FROM profile_entries WHERE profile_id=$1",
    )
    .bind(profile_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::journal::FilePair;

    /// Create an in-memory SQLite pool with all migrations applied.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory pool");

        // Enable foreign keys (SQLite requires this per-connection)
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("Failed to enable foreign keys");

        // Run all migrations
        for migration in crate::db::migrations::get_migrations() {
            sqlx::raw_sql(migration.sql)
                .execute(&pool)
                .await
                .expect(&format!("Failed migration: {}", migration.description));
        }

        pool
    }

    /// Helper: insert a game and return it.
    async fn seed_game(pool: &SqlitePool) -> GameRecord {
        insert_game(pool, "Tekken 8", "C:/game/Mods", "C:/staging", "structured", false)
            .await
            .unwrap()
    }

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

    // ─── Mod CRUD Tests ───

    #[tokio::test]
    async fn test_insert_mod_creates_record() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Cool Mod", "C:/staging/cool-mod").await.unwrap();
        assert_eq!(m.name, "Cool Mod");
        assert_eq!(m.game_id, game.id);
        assert!(!m.enabled, "New mod should be disabled by default");
        assert_eq!(m.staged_path, "C:/staging/cool-mod");
    }

    #[tokio::test]
    async fn test_get_mod_returns_record() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Test Mod", "C:/staging/test").await.unwrap();
        let fetched = get_mod(&pool, m.id).await.unwrap();
        assert_eq!(fetched.id, m.id);
        assert_eq!(fetched.name, "Test Mod");
    }

    #[tokio::test]
    async fn test_get_mod_missing_returns_error() {
        let pool = test_pool().await;
        let result = get_mod(&pool, 999).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ModNotFound(_) => {}
            other => panic!("Expected ModNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_list_mods_for_game_ordered() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let _m1 = insert_mod(&pool, game.id, "Zebra Mod", "C:/staging/z").await.unwrap();
        let m2 = insert_mod(&pool, game.id, "Alpha Mod", "C:/staging/a").await.unwrap();
        let _m3 = insert_mod(&pool, game.id, "Beta Mod", "C:/staging/b").await.unwrap();
        // Enable Alpha Mod -- it should appear first
        update_mod_enabled(&pool, m2.id, true).await.unwrap();

        let mods = list_mods_for_game(&pool, game.id).await.unwrap();
        assert_eq!(mods.len(), 3);
        assert_eq!(mods[0].name, "Alpha Mod", "Enabled mod first");
        assert_eq!(mods[1].name, "Beta Mod", "Then alphabetical");
        assert_eq!(mods[2].name, "Zebra Mod");
    }

    #[tokio::test]
    async fn test_list_mods_for_game_filters_by_game() {
        let pool = test_pool().await;
        let game1 = seed_game(&pool).await;
        let game2 = insert_game(&pool, "Other Game", "D:/other", "D:/staging", "loose", false).await.unwrap();
        insert_mod(&pool, game1.id, "Game1 Mod", "C:/staging/g1").await.unwrap();
        insert_mod(&pool, game2.id, "Game2 Mod", "D:/staging/g2").await.unwrap();

        let mods = list_mods_for_game(&pool, game1.id).await.unwrap();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Game1 Mod");
    }

    #[tokio::test]
    async fn test_update_mod_enabled() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        assert!(!m.enabled);

        update_mod_enabled(&pool, m.id, true).await.unwrap();
        let fetched = get_mod(&pool, m.id).await.unwrap();
        assert!(fetched.enabled);

        update_mod_enabled(&pool, m.id, false).await.unwrap();
        let fetched = get_mod(&pool, m.id).await.unwrap();
        assert!(!fetched.enabled);
    }

    #[tokio::test]
    async fn test_delete_mod_cascade() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let sm = insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();
        insert_file_entry(&pool, m.id, "main.pak", None).await.unwrap();
        insert_file_entry(&pool, m.id, "option.pak", Some(sm.id)).await.unwrap();

        delete_mod_db(&pool, m.id).await.unwrap();

        // Mod gone
        assert!(get_mod(&pool, m.id).await.is_err());
        // File entries gone
        let files = list_file_entries(&pool, m.id).await.unwrap();
        assert!(files.is_empty());
        // Sub-mods gone
        let subs = list_sub_mods(&pool, m.id).await.unwrap();
        assert!(subs.is_empty());
    }

    // ─── Sub-Mod Tests ───

    #[tokio::test]
    async fn test_insert_sub_mod_defaults() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let sm = insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();
        assert_eq!(sm.mod_id, m.id);
        assert_eq!(sm.name, "Option A");
        assert_eq!(sm.folder_name, "Option_A");
        assert!(!sm.enabled, "Sub-mod enabled should default to false");
        assert!(!sm.user_enabled, "Sub-mod user_enabled should default to false");
    }

    #[tokio::test]
    async fn test_list_sub_mods() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();
        insert_sub_mod(&pool, m.id, "Option B", "Option_B").await.unwrap();

        let subs = list_sub_mods(&pool, m.id).await.unwrap();
        assert_eq!(subs.len(), 2);
    }

    #[tokio::test]
    async fn test_update_sub_mod_enabled() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let sm = insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();

        update_sub_mod_enabled(&pool, sm.id, true, true).await.unwrap();
        let fetched = get_sub_mod(&pool, sm.id).await.unwrap();
        assert!(fetched.enabled);
        assert!(fetched.user_enabled);

        // Disable effective state but keep user_enabled
        update_sub_mod_enabled(&pool, sm.id, false, true).await.unwrap();
        let fetched = get_sub_mod(&pool, sm.id).await.unwrap();
        assert!(!fetched.enabled);
        assert!(fetched.user_enabled, "user_enabled should be preserved");
    }

    // ─── File Entry Tests ───

    #[tokio::test]
    async fn test_insert_file_entry_without_sub_mod() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let fe = insert_file_entry(&pool, m.id, "textures/skin.pak", None).await.unwrap();
        assert_eq!(fe.mod_id, m.id);
        assert_eq!(fe.relative_path, "textures/skin.pak");
        assert!(fe.sub_mod_id.is_none());
    }

    #[tokio::test]
    async fn test_insert_file_entry_with_sub_mod() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let sm = insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();
        let fe = insert_file_entry(&pool, m.id, "Option_A/extra.pak", Some(sm.id)).await.unwrap();
        assert_eq!(fe.sub_mod_id, Some(sm.id));
    }

    #[tokio::test]
    async fn test_list_file_entries_for_sub_mod() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let sm = insert_sub_mod(&pool, m.id, "Option A", "Option_A").await.unwrap();
        insert_file_entry(&pool, m.id, "main.pak", None).await.unwrap();
        insert_file_entry(&pool, m.id, "Option_A/opt.pak", Some(sm.id)).await.unwrap();

        let sub_files = list_file_entries_for_sub_mod(&pool, sm.id).await.unwrap();
        assert_eq!(sub_files.len(), 1);
        assert_eq!(sub_files[0].relative_path, "Option_A/opt.pak");
    }

    // ─── Conflict Detection Tests ───

    #[tokio::test]
    async fn test_check_conflicts_finds_overlapping_paths() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m1 = insert_mod(&pool, game.id, "Mod A", "C:/staging/a").await.unwrap();
        let m2 = insert_mod(&pool, game.id, "Mod B", "C:/staging/b").await.unwrap();
        // Both mods have the same relative path
        insert_file_entry(&pool, m1.id, "shared.pak", None).await.unwrap();
        insert_file_entry(&pool, m2.id, "shared.pak", None).await.unwrap();
        insert_file_entry(&pool, m1.id, "unique_a.pak", None).await.unwrap();

        // Enable Mod B so it shows up as a conflict when checking Mod A
        update_mod_enabled(&pool, m2.id, true).await.unwrap();

        let conflicts = check_conflicts(&pool, m1.id, game.id).await.unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflicting_mod_name, "Mod B");
        assert_eq!(conflicts[0].relative_path, "shared.pak");
    }

    #[tokio::test]
    async fn test_check_conflicts_no_conflict_when_other_disabled() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m1 = insert_mod(&pool, game.id, "Mod A", "C:/staging/a").await.unwrap();
        let m2 = insert_mod(&pool, game.id, "Mod B", "C:/staging/b").await.unwrap();
        insert_file_entry(&pool, m1.id, "shared.pak", None).await.unwrap();
        insert_file_entry(&pool, m2.id, "shared.pak", None).await.unwrap();
        // Mod B disabled -- no conflict
        let conflicts = check_conflicts(&pool, m1.id, game.id).await.unwrap();
        assert!(conflicts.is_empty());
    }

    // ─── Journal Tests ───

    #[tokio::test]
    async fn test_begin_toggle_creates_journal() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let pairs = vec![
            FilePair { src: "C:/staging/m/a.pak".into(), dst: "C:/game/Mods/a.pak".into(), done: false },
        ];
        let journal_id = begin_toggle(&pool, m.id, "enable", &pairs).await.unwrap();
        assert!(journal_id > 0);

        // Verify it shows up as incomplete
        let incomplete = scan_incomplete_journals(&pool).await.unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].id, journal_id);
        assert_eq!(incomplete[0].operation, "enable");
    }

    #[tokio::test]
    async fn test_mark_file_done_updates_json() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let pairs = vec![
            FilePair { src: "a".into(), dst: "b".into(), done: false },
            FilePair { src: "c".into(), dst: "d".into(), done: false },
        ];
        let jid = begin_toggle(&pool, m.id, "enable", &pairs).await.unwrap();

        mark_file_done(&pool, jid, 0).await.unwrap();

        // Verify via scan
        let incomplete = scan_incomplete_journals(&pool).await.unwrap();
        assert_eq!(incomplete[0].files[0].done, true);
        assert_eq!(incomplete[0].files[1].done, false);
    }

    #[tokio::test]
    async fn test_complete_journal_sets_done() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let pairs = vec![
            FilePair { src: "a".into(), dst: "b".into(), done: false },
        ];
        let jid = begin_toggle(&pool, m.id, "enable", &pairs).await.unwrap();

        complete_journal(&pool, jid).await.unwrap();

        // Should no longer be incomplete
        let incomplete = scan_incomplete_journals(&pool).await.unwrap();
        assert!(incomplete.is_empty(), "Completed journal should not appear in scan");
    }

    // ─── Profile Tests ───

    #[tokio::test]
    async fn test_insert_profile_creates_record() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let profile = insert_profile(&pool, game.id, "Tournament Setup").await.unwrap();
        assert_eq!(profile.game_id, game.id);
        assert_eq!(profile.name, "Tournament Setup");
        assert!(profile.created_at > 0);
        assert!(profile.updated_at > 0);
    }

    #[tokio::test]
    async fn test_list_profiles_for_game_filters() {
        let pool = test_pool().await;
        let game1 = seed_game(&pool).await;
        let game2 = insert_game(&pool, "Other Game", "D:/other", "D:/staging", "loose", false).await.unwrap();
        insert_profile(&pool, game1.id, "Profile A").await.unwrap();
        insert_profile(&pool, game1.id, "Profile B").await.unwrap();
        insert_profile(&pool, game2.id, "Other Profile").await.unwrap();

        let profiles = list_profiles_for_game(&pool, game1.id).await.unwrap();
        assert_eq!(profiles.len(), 2);
        // Ordered by name ASC
        assert_eq!(profiles[0].name, "Profile A");
        assert_eq!(profiles[1].name, "Profile B");

        let profiles2 = list_profiles_for_game(&pool, game2.id).await.unwrap();
        assert_eq!(profiles2.len(), 1);
    }

    #[tokio::test]
    async fn test_get_profile_by_name_returns_match() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        insert_profile(&pool, game.id, "My Profile").await.unwrap();

        let found = get_profile_by_name(&pool, game.id, "My Profile").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "My Profile");

        let not_found = get_profile_by_name(&pool, game.id, "Nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_unique_game_name_constraint() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        insert_profile(&pool, game.id, "Duplicate").await.unwrap();
        let result = insert_profile(&pool, game.id, "Duplicate").await;
        assert!(result.is_err(), "Duplicate name for same game should fail");
    }

    #[tokio::test]
    async fn test_delete_profile_cascades_to_entries() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let profile = insert_profile(&pool, game.id, "To Delete").await.unwrap();
        insert_profile_entry(&pool, profile.id, m.id, true, None).await.unwrap();

        // Entries exist
        let entries = list_profile_entries(&pool, profile.id).await.unwrap();
        assert_eq!(entries.len(), 1);

        // Delete profile
        delete_profile(&pool, profile.id).await.unwrap();

        // Entries gone
        let entries = list_profile_entries(&pool, profile.id).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_profile_entry_roundtrip_with_sub_mod_states() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let profile = insert_profile(&pool, game.id, "Test Profile").await.unwrap();

        let sub_states = r#"[{"sub_mod_id":1,"enabled":true},{"sub_mod_id":2,"enabled":false}]"#;
        insert_profile_entry(&pool, profile.id, m.id, true, Some(sub_states)).await.unwrap();

        let entries = list_profile_entries(&pool, profile.id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].mod_id, m.id);
        assert!(entries[0].enabled);
        assert_eq!(entries[0].sub_mod_states.as_deref(), Some(sub_states));
    }

    #[tokio::test]
    async fn test_delete_mod_cascades_to_profile_entries() {
        let pool = test_pool().await;
        let game = seed_game(&pool).await;
        let m = insert_mod(&pool, game.id, "Mod", "C:/staging/m").await.unwrap();
        let profile = insert_profile(&pool, game.id, "Test").await.unwrap();
        insert_profile_entry(&pool, profile.id, m.id, true, None).await.unwrap();

        // Delete the mod
        delete_mod_db(&pool, m.id).await.unwrap();

        // Profile still exists but entries are gone
        let profile_still = get_profile(&pool, profile.id).await.unwrap();
        assert_eq!(profile_still.name, "Test");
        let entries = list_profile_entries(&pool, profile.id).await.unwrap();
        assert!(entries.is_empty(), "Mod deletion should cascade to profile_entries");
    }
}
