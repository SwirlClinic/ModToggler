use serde::Serialize;
use specta::Type;

/// All error types that can be returned from Tauri commands.
/// Tagged union serialized as {"kind": "VariantName", "message": "..."} for frontend consumption.
/// Every variant has a String message for display.
#[derive(Debug, Serialize, Type, Clone)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    /// std::io::Error that doesn't fit a more specific category
    IoError(String),
    /// File or directory operation rejected — may need elevation
    PermissionDenied(String),
    /// SQLite or tauri-plugin-sql error
    DatabaseError(String),
    /// Requested game ID not found in DB
    GameNotFound(String),
    /// Requested mod ID not found in DB
    ModNotFound(String),
    /// Staging directory already claimed by another game/mod
    StagingDirConflict(String),
    /// Staging path is on a different drive than the mod directory — not an error, a warning surfaced to UI
    CrossDriveWarning(String),
    /// toggle_journal entry is in a state that cannot be automatically recovered
    JournalCorrupt(String),
    /// serde_json serialization/deserialization failure
    SerializationError(String),
    /// Catch-all
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                AppError::PermissionDenied(e.to_string())
            }
            std::io::ErrorKind::NotFound => {
                AppError::IoError(format!("File not found: {}", e))
            }
            _ => AppError::IoError(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerializationError(e.to_string())
    }
}

// Note: AppError implements Serialize, so Tauri's blanket impl<T: Serialize> From<T> for InvokeError
// already covers conversion. No explicit From impl needed.

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_permission_denied_mapping() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::PermissionDenied(_)));
    }

    #[test]
    fn test_not_found_mapping() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let app_err = AppError::from(io_err);
        match app_err {
            AppError::IoError(msg) => assert!(msg.contains("File not found")),
            other => panic!("Expected IoError, got {:?}", other),
        }
    }

    #[test]
    fn test_generic_io_mapping() {
        let io_err = io::Error::new(io::ErrorKind::TimedOut, "timed out");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::IoError(_)));
    }

    #[test]
    fn test_error_serializes_with_tag() {
        let err = AppError::PermissionDenied("cannot write".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\""));
        assert!(json.contains("PermissionDenied"));
    }
}
