use serde::{Deserialize, Serialize};
use crate::error::AppError;

/// A single file move pair tracked in the journal.
/// `done` is updated to true after each individual file is successfully moved.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FilePair {
    pub src: String,
    pub dst: String,
    pub done: bool,
}

/// A journal entry returned by scan_incomplete().
#[derive(Debug, Serialize, Deserialize)]
pub struct IncompleteJournalEntry {
    pub id: i64,
    pub mod_id: i64,
    pub operation: String,
    pub files: Vec<FilePair>,
}

/// Serialize FilePairs to JSON string for storage in toggle_journal.files_json.
pub fn serialize_files(files: &[FilePair]) -> Result<String, AppError> {
    serde_json::to_string(files).map_err(AppError::from)
}

/// Deserialize FilePairs from the stored JSON string.
pub fn deserialize_files(json: &str) -> Result<Vec<FilePair>, AppError> {
    serde_json::from_str(json).map_err(AppError::from)
}

/// Recovery logic: given a list of FilePairs from an in_progress journal,
/// return the pairs that still need to be processed (done == false).
pub fn pending_files(files: &[FilePair]) -> Vec<&FilePair> {
    files.iter().filter(|f| !f.done).collect()
}

// NOTE: The async DB functions (begin_toggle, mark_file_done, complete_journal,
// rollback_journal, scan_incomplete) require tauri_plugin_sql::Database.
// They are defined in commands/integrity.rs and called from toggle commands in Phase 2.
// The data structures and pure logic are here; the async DB calls live where the DB state is available.
//
// This separation keeps the service layer testable without a Tauri runtime.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_pair_round_trips_json() {
        let pairs = vec![
            FilePair {
                src: "C:/staging/mod/file.pak".into(),
                dst: "C:/game/Mods/file.pak".into(),
                done: false,
            },
            FilePair {
                src: "C:/staging/mod/file.ucas".into(),
                dst: "C:/game/Mods/file.ucas".into(),
                done: true,
            },
        ];
        let json = serialize_files(&pairs).unwrap();
        let decoded = deserialize_files(&json).unwrap();
        assert_eq!(pairs, decoded);
    }

    #[test]
    fn test_pending_files_filters_done() {
        let pairs = vec![
            FilePair { src: "a".into(), dst: "b".into(), done: false },
            FilePair { src: "c".into(), dst: "d".into(), done: true },
            FilePair { src: "e".into(), dst: "f".into(), done: false },
        ];
        let pending = pending_files(&pairs);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].src, "a");
        assert_eq!(pending[1].src, "e");
    }

    #[test]
    fn test_pending_files_all_done_returns_empty() {
        let pairs = vec![
            FilePair { src: "a".into(), dst: "b".into(), done: true },
        ];
        let pending = pending_files(&pairs);
        assert!(pending.is_empty(), "All done — should return empty vec");
    }

    #[test]
    fn test_serialize_empty_files() {
        // Edge case: empty file list (valid journal state before files are written)
        let pairs: Vec<FilePair> = vec![];
        let json = serialize_files(&pairs).unwrap();
        let decoded = deserialize_files(&json).unwrap();
        assert!(decoded.is_empty());
    }

    /// 'done' booleans must survive a JSON round-trip without being coerced.
    #[test]
    fn test_mod_enabled_persists() {
        let pairs = vec![
            FilePair { src: "a.pak".into(), dst: "b.pak".into(), done: false },
            FilePair { src: "c.pak".into(), dst: "d.pak".into(), done: true },
        ];
        let json = serialize_files(&pairs).unwrap();
        let decoded = deserialize_files(&json).unwrap();
        assert_eq!(decoded[0].done, false, "done=false must survive round-trip");
        assert_eq!(decoded[1].done, true,  "done=true must survive round-trip");
    }

    /// pending_files() must correctly surface in-progress (done=false) entries as the
    /// 'incomplete journal detected' signal consumed by the integrity scan.
    #[test]
    fn test_incomplete_journal_detected() {
        let pairs = vec![
            FilePair { src: "a.pak".into(), dst: "b.pak".into(), done: true  },
            FilePair { src: "c.pak".into(), dst: "d.pak".into(), done: false },
            FilePair { src: "e.pak".into(), dst: "f.pak".into(), done: false },
        ];
        let incomplete = pending_files(&pairs);
        assert_eq!(incomplete.len(), 2, "Should detect 2 incomplete file moves");
        // All-done journal must not be flagged as incomplete
        let all_done = vec![
            FilePair { src: "a.pak".into(), dst: "b.pak".into(), done: true },
        ];
        assert!(pending_files(&all_done).is_empty(), "All-done journal must not be flagged");
    }
}
