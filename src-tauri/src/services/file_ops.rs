use std::path::Path;
use tauri::{AppHandle, Emitter};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::AppError;

#[derive(Clone, serde::Serialize)]
pub struct MoveProgressEvent {
    pub file: String,
    pub percent: u32,
}

/// Check if two paths are on the same volume.
/// Uses drive-letter prefix comparison (Windows-primary, works for absolute paths).
/// Returns true if both paths have the same root component.
pub fn same_volume(path_a: &Path, path_b: &Path) -> bool {
    let root_a = path_a.components().next();
    let root_b = path_b.components().next();
    root_a == root_b
}

/// Detects whether an IO error is a cross-device rename failure.
/// Windows: OS error 17 (ERROR_NOT_SAME_DEVICE)
/// Linux:   OS error 18 (EXDEV)
pub fn is_cross_device_error(e: &std::io::Error) -> bool {
    matches!(e.raw_os_error(), Some(17) | Some(18))
}

/// Idempotent staging directory creation. Call before every file move, not just at game-add time.
/// Uses create_dir_all — safe to call on an already-existing directory.
pub async fn create_staging_dir(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path).await.map_err(AppError::from)
}

/// Move a file from src to dst.
/// Attempts atomic rename first. On cross-device error (OS 17/18), falls back to
/// copy-with-progress + delete. Emits "file-move-progress" events during fallback.
///
/// IMPORTANT: Caller must ensure dst parent directory exists (call create_staging_dir first).
pub async fn move_file(
    app: &AppHandle,
    src: &Path,
    dst: &Path,
) -> Result<(), AppError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).await?;
    }

    match fs::rename(src, dst).await {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device_error(&e) => {
            copy_with_progress(app, src, dst).await?;
            fs::remove_file(src).await?;
            Ok(())
        }
        Err(e) => Err(AppError::from(e)),
    }
}

async fn copy_with_progress(
    app: &AppHandle,
    src: &Path,
    dst: &Path,
) -> Result<(), AppError> {
    let file_size = fs::metadata(src).await?.len();
    let mut reader = fs::File::open(src).await?;
    let mut writer = fs::File::create(dst).await?;
    let mut buf = vec![0u8; 256 * 1024]; // 256KB chunks
    let mut bytes_copied: u64 = 0;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n]).await?;
        bytes_copied += n as u64;

        let pct = (bytes_copied * 100 / file_size.max(1)) as u32;
        let _ = app.emit(
            "file-move-progress",
            MoveProgressEvent {
                file: src
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                percent: pct,
            },
        );
    }

    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_same_volume_same_drive() {
        let a = PathBuf::from("C:/Users/foo/mods");
        let b = PathBuf::from("C:/modtoggler/staging");
        assert!(same_volume(&a, &b));
    }

    #[test]
    fn test_same_volume_different_drives() {
        let a = PathBuf::from("C:/Users/foo/mods");
        let b = PathBuf::from("D:/modtoggler/staging");
        assert!(!same_volume(&a, &b));
    }

    #[test]
    fn test_is_cross_device_error_windows() {
        // Simulate OS error 17 (ERROR_NOT_SAME_DEVICE on Windows)
        let err = std::io::Error::from_raw_os_error(17);
        assert!(is_cross_device_error(&err));
    }

    #[test]
    fn test_is_cross_device_error_not_found() {
        let err = std::io::Error::from_raw_os_error(2); // NOT_FOUND
        assert!(!is_cross_device_error(&err));
    }

    /// OS 17 (Windows ERROR_NOT_SAME_DEVICE) and OS 18 (Linux EXDEV) must both trigger
    /// the copy+delete fallback. Other errors must not.
    #[test]
    fn test_cross_drive_fallback() {
        let err_17 = std::io::Error::from_raw_os_error(17);
        assert!(is_cross_device_error(&err_17), "OS 17 must trigger fallback");
        let err_18 = std::io::Error::from_raw_os_error(18);
        assert!(is_cross_device_error(&err_18), "OS 18 must trigger fallback");
        let err_13 = std::io::Error::from_raw_os_error(13); // EACCES
        assert!(!is_cross_device_error(&err_13), "OS 13 must NOT trigger fallback");
    }
}
