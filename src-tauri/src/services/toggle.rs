use std::path::Path;

use sqlx::SqlitePool;
use tauri::AppHandle;
use tokio::fs;

use crate::db::queries::{self, FileEntry, SubModRecord};
use crate::error::AppError;
use crate::services::journal::FilePair;
use crate::services::file_ops;

/// Build file pairs for a list of file entries, given source and destination base dirs.
/// Each FileEntry's relative_path is joined to src_base (source) and dst_base (destination).
pub fn build_file_pairs(entries: &[FileEntry], src_base: &Path, dst_base: &Path) -> Vec<FilePair> {
    entries
        .iter()
        .map(|entry| {
            // Use display() for OS-native path rendering (move_file expects OS paths)
            let src = src_base.join(&entry.relative_path);
            let dst = dst_base.join(&entry.relative_path);
            FilePair {
                src: src.display().to_string(),
                dst: dst.display().to_string(),
                done: false,
            }
        })
        .collect()
}

/// Filter file entries to only those belonging to a specific sub-mod.
fn entries_for_sub_mod(entries: &[FileEntry], sub_mod_id: i64) -> Vec<FileEntry> {
    entries
        .iter()
        .filter(|e| e.sub_mod_id == Some(sub_mod_id))
        .cloned()
        .collect()
}

/// Filter file entries to only parent mod files (sub_mod_id is None).
fn parent_entries(entries: &[FileEntry]) -> Vec<FileEntry> {
    entries
        .iter()
        .filter(|e| e.sub_mod_id.is_none())
        .cloned()
        .collect()
}

/// Get sub-mod IDs that should be restored when parent is re-enabled.
/// Returns sub-mods where user_enabled is true.
pub fn get_sub_mod_states_to_restore(sub_mods: &[SubModRecord]) -> Vec<i64> {
    sub_mods
        .iter()
        .filter(|sm| sm.user_enabled)
        .map(|sm| sm.id)
        .collect()
}

/// Execute a journal-wrapped file move operation.
/// Begins a journal, moves each file pair, marks each done, then completes the journal.
async fn journal_move_files(
    app: &AppHandle,
    pool: &SqlitePool,
    mod_id: i64,
    operation: &str,
    pairs: &[FilePair],
) -> Result<(), AppError> {
    if pairs.is_empty() {
        return Ok(());
    }
    let journal_id = queries::begin_toggle(pool, mod_id, operation, pairs).await?;
    for (i, pair) in pairs.iter().enumerate() {
        file_ops::move_file(app, Path::new(&pair.src), Path::new(&pair.dst)).await?;
        queries::mark_file_done(pool, journal_id, i).await?;
    }
    queries::complete_journal(pool, journal_id).await?;
    Ok(())
}

/// Toggle a mod on or off. Wraps all file moves in a journal for crash safety.
/// When disabling: also disables all sub-mods (preserving user_enabled).
/// When enabling: restores sub-mods where user_enabled=true.
pub async fn toggle_mod(
    app: &AppHandle,
    pool: &SqlitePool,
    mod_id: i64,
    enable: bool,
) -> Result<(), AppError> {
    let mod_rec = queries::get_mod(pool, mod_id).await?;
    let game = queries::get_game(pool, mod_rec.game_id).await?;
    let all_entries = queries::list_file_entries(pool, mod_id).await?;
    let sub_mods = queries::list_sub_mods(pool, mod_id).await?;

    let staging = Path::new(&mod_rec.staged_path);
    let game_dir = Path::new(&game.mod_dir);

    let operation = if enable { "enable" } else { "disable" };

    if enable {
        // Move parent files: staging -> game dir
        let parent = parent_entries(&all_entries);
        let pairs = build_file_pairs(&parent, staging, game_dir);
        journal_move_files(app, pool, mod_id, operation, &pairs).await?;

        // Restore sub-mods where user_enabled=true
        let restore_ids = get_sub_mod_states_to_restore(&sub_mods);
        for sm in &sub_mods {
            if restore_ids.contains(&sm.id) {
                let sm_entries = entries_for_sub_mod(&all_entries, sm.id);
                let sm_pairs = build_file_pairs(&sm_entries, staging, game_dir);
                journal_move_files(app, pool, mod_id, operation, &sm_pairs).await?;
                queries::update_sub_mod_enabled(pool, sm.id, true, true).await?;
            }
        }
    } else {
        // Move sub-mod files back to staging first
        for sm in &sub_mods {
            if sm.enabled {
                let sm_entries = entries_for_sub_mod(&all_entries, sm.id);
                let sm_pairs = build_file_pairs(&sm_entries, game_dir, staging);
                journal_move_files(app, pool, mod_id, operation, &sm_pairs).await?;
            }
            // Set enabled=false but preserve user_enabled
            queries::update_sub_mod_enabled(pool, sm.id, false, sm.user_enabled).await?;
        }

        // Move parent files: game dir -> staging
        let parent = parent_entries(&all_entries);
        let pairs = build_file_pairs(&parent, game_dir, staging);
        journal_move_files(app, pool, mod_id, operation, &pairs).await?;
    }

    queries::update_mod_enabled(pool, mod_id, enable).await?;
    Ok(())
}

/// Toggle a single sub-mod on or off. Parent must be enabled.
/// Sets both enabled and user_enabled to the target state.
pub async fn toggle_sub_mod(
    app: &AppHandle,
    pool: &SqlitePool,
    sub_mod_id: i64,
    enable: bool,
) -> Result<(), AppError> {
    let sm = queries::get_sub_mod(pool, sub_mod_id).await?;
    let mod_rec = queries::get_mod(pool, sm.mod_id).await?;

    if !mod_rec.enabled {
        return Err(AppError::ModNotFound(
            "Cannot toggle sub-mod: parent mod is disabled".to_string(),
        ));
    }

    let game = queries::get_game(pool, mod_rec.game_id).await?;
    let sm_entries = queries::list_file_entries_for_sub_mod(pool, sub_mod_id).await?;
    let staging = Path::new(&mod_rec.staged_path);
    let game_dir = Path::new(&game.mod_dir);

    let operation = if enable { "enable" } else { "disable" };

    let pairs = if enable {
        build_file_pairs(&sm_entries, staging, game_dir)
    } else {
        build_file_pairs(&sm_entries, game_dir, staging)
    };

    journal_move_files(app, pool, sm.mod_id, operation, &pairs).await?;
    queries::update_sub_mod_enabled(pool, sub_mod_id, enable, enable).await?;

    Ok(())
}

/// Permanently delete a mod: remove files from disk (wherever they are), delete DB records.
pub async fn delete_mod(
    app: &AppHandle,
    pool: &SqlitePool,
    mod_id: i64,
) -> Result<(), AppError> {
    let mod_rec = queries::get_mod(pool, mod_id).await?;
    let game = queries::get_game(pool, mod_rec.game_id).await?;
    let all_entries = queries::list_file_entries(pool, mod_id).await?;

    let staging = Path::new(&mod_rec.staged_path);
    let game_dir = Path::new(&game.mod_dir);

    // Remove each file from whichever location it exists in
    for entry in &all_entries {
        let staging_file = staging.join(&entry.relative_path);
        let game_file = game_dir.join(&entry.relative_path);

        if staging_file.exists() {
            let _ = fs::remove_file(&staging_file).await;
        }
        if game_file.exists() {
            let _ = fs::remove_file(&game_file).await;
        }
    }

    // Clean up empty directories under staging path
    let _ = cleanup_empty_dirs(staging).await;

    // Delete from DB (cascade handles file_entries and sub_mods)
    queries::delete_mod_db(pool, mod_id).await?;

    // Suppress unused variable warning -- app handle available for future use (e.g., progress events)
    let _ = app;

    Ok(())
}

/// Recursively remove empty directories under the given path.
/// Silently ignores errors (best-effort cleanup).
async fn cleanup_empty_dirs(path: &Path) -> Result<(), std::io::Error> {
    if !path.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(path).await?;
    let mut has_children = false;

    while let Some(entry) = entries.next_entry().await? {
        has_children = true;
        if entry.file_type().await?.is_dir() {
            let _ = Box::pin(cleanup_empty_dirs(&entry.path())).await;
        }
    }

    // Re-check if directory is now empty (after recursive cleanup)
    if has_children {
        let mut re_entries = fs::read_dir(path).await?;
        if re_entries.next_entry().await?.is_none() {
            let _ = fs::remove_dir(path).await;
        }
    } else {
        let _ = fs::remove_dir(path).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::queries::{FileEntry, SubModRecord};
    use std::path::PathBuf;

    /// Normalize path string to forward slashes for cross-platform test assertions.
    fn normalize(s: &str) -> String {
        s.replace('\\', "/")
    }

    #[test]
    fn test_build_file_pairs_enable() {
        let entries = vec![
            FileEntry {
                id: 1,
                mod_id: 1,
                relative_path: "textures/skin.pak".to_string(),
                sub_mod_id: None,
            },
            FileEntry {
                id: 2,
                mod_id: 1,
                relative_path: "meshes/body.pak".to_string(),
                sub_mod_id: None,
            },
        ];
        let staging = PathBuf::from("C:/staging/mymod");
        let game = PathBuf::from("C:/game/Mods");

        let pairs = build_file_pairs(&entries, &staging, &game);
        assert_eq!(pairs.len(), 2);
        assert_eq!(normalize(&pairs[0].src), "C:/staging/mymod/textures/skin.pak");
        assert_eq!(normalize(&pairs[0].dst), "C:/game/Mods/textures/skin.pak");
        assert!(!pairs[0].done);
    }

    #[test]
    fn test_build_file_pairs_disable() {
        let entries = vec![FileEntry {
            id: 1,
            mod_id: 1,
            relative_path: "data.pak".to_string(),
            sub_mod_id: None,
        }];
        let staging = PathBuf::from("C:/staging/mod");
        let game = PathBuf::from("C:/game/Mods");

        // For disable, game_dir is src, staging is dst
        let pairs = build_file_pairs(&entries, &game, &staging);
        assert_eq!(normalize(&pairs[0].src), "C:/game/Mods/data.pak");
        assert_eq!(normalize(&pairs[0].dst), "C:/staging/mod/data.pak");
    }

    #[test]
    fn test_build_file_pairs_sub_mod_only() {
        let all_entries = vec![
            FileEntry {
                id: 1,
                mod_id: 1,
                relative_path: "main.pak".to_string(),
                sub_mod_id: None,
            },
            FileEntry {
                id: 2,
                mod_id: 1,
                relative_path: "Option_A/extra.pak".to_string(),
                sub_mod_id: Some(10),
            },
            FileEntry {
                id: 3,
                mod_id: 1,
                relative_path: "Option_B/other.pak".to_string(),
                sub_mod_id: Some(20),
            },
        ];

        // Filter to sub-mod 10 only
        let sub_entries = entries_for_sub_mod(&all_entries, 10);
        assert_eq!(sub_entries.len(), 1);
        assert_eq!(sub_entries[0].relative_path, "Option_A/extra.pak");

        let staging = PathBuf::from("C:/staging/mod");
        let game = PathBuf::from("C:/game/Mods");
        let pairs = build_file_pairs(&sub_entries, &staging, &game);
        assert_eq!(pairs.len(), 1);
        assert_eq!(normalize(&pairs[0].src), "C:/staging/mod/Option_A/extra.pak");
    }

    #[test]
    fn test_parent_entries_excludes_sub_mods() {
        let all_entries = vec![
            FileEntry {
                id: 1,
                mod_id: 1,
                relative_path: "main.pak".to_string(),
                sub_mod_id: None,
            },
            FileEntry {
                id: 2,
                mod_id: 1,
                relative_path: "Option_A/extra.pak".to_string(),
                sub_mod_id: Some(10),
            },
        ];

        let parent = parent_entries(&all_entries);
        assert_eq!(parent.len(), 1);
        assert_eq!(parent[0].relative_path, "main.pak");
    }

    #[test]
    fn test_get_sub_mod_states_to_restore() {
        let sub_mods = vec![
            SubModRecord {
                id: 1,
                mod_id: 1,
                name: "Option A".to_string(),
                folder_name: "Option_A".to_string(),
                enabled: false,
                user_enabled: true,
            },
            SubModRecord {
                id: 2,
                mod_id: 1,
                name: "Option B".to_string(),
                folder_name: "Option_B".to_string(),
                enabled: false,
                user_enabled: false,
            },
            SubModRecord {
                id: 3,
                mod_id: 1,
                name: "Option C".to_string(),
                folder_name: "Option_C".to_string(),
                enabled: false,
                user_enabled: true,
            },
        ];

        let restore = get_sub_mod_states_to_restore(&sub_mods);
        assert_eq!(restore.len(), 2);
        assert!(restore.contains(&1));
        assert!(restore.contains(&3));
        assert!(!restore.contains(&2), "Option B has user_enabled=false");
    }

    #[test]
    fn test_build_file_pairs_empty() {
        let entries: Vec<FileEntry> = vec![];
        let pairs = build_file_pairs(&entries, Path::new("a"), Path::new("b"));
        assert!(pairs.is_empty());
    }
}
