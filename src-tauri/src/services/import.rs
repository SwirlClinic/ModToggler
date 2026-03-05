use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use zip::ZipArchive;

use crate::error::AppError;

/// Extract zip to staging_dir. Returns manifest of relative file paths (forward-slash normalized).
/// Skips directory entries and unsafe paths (ZipSlip protection via enclosed_name).
pub fn extract_zip_to_staging(
    zip_path: &Path,
    staging_dir: &Path,
) -> Result<Vec<String>, AppError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::IoError(format!("Failed to open zip: {}", e)))?;

    let mut manifest = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::IoError(format!("Failed to read zip entry {}: {}", i, e)))?;

        // Skip directory entries
        if entry.is_dir() {
            continue;
        }

        // ZipSlip protection: enclosed_name returns None for paths with .. or absolute paths
        let relative = match entry.enclosed_name() {
            Some(p) => p.to_owned(),
            None => continue, // Skip unsafe paths
        };

        // Normalize to forward slashes for the manifest
        let normalized = relative
            .to_string_lossy()
            .replace('\\', "/");

        let dest = staging_dir.join(&relative);

        // Create parent directories
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut outfile = fs::File::create(&dest)?;
        io::copy(&mut entry, &mut outfile)?;

        manifest.push(normalized);
    }

    Ok(manifest)
}

/// Partition file paths into main_files (not under Option_* dirs) and sub_mods (keyed by option folder name).
/// Files inside Option_*/option_* directories belong ONLY to sub-mods, NOT the parent.
pub fn partition_files(paths: &[String]) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut main_files = Vec::new();
    let mut sub_mods: HashMap<String, Vec<String>> = HashMap::new();

    for path in paths {
        // Check if first path component starts with Option_ or option_
        let first_component = path.split('/').next().unwrap_or("");
        if first_component.starts_with("Option_") || first_component.starts_with("option_") {
            let folder_name = first_component.to_string();
            // The relative path within the sub-mod is everything after the first component
            let inner_path = path
                .strip_prefix(first_component)
                .unwrap_or(path)
                .strip_prefix('/')
                .unwrap_or(path);
            sub_mods
                .entry(folder_name)
                .or_default()
                .push(inner_path.to_string());
        } else {
            main_files.push(path.clone());
        }
    }

    (main_files, sub_mods)
}

/// Copy source files to a staging directory. Returns the list of actual staging filenames used.
/// On filename collision (two files with the same target name), appends a numeric suffix.
/// source_paths: slice of (absolute_source_path, desired_staging_filename) pairs.
pub fn copy_files_to_staging(
    source_paths: &[(String, String)],
    staging_dir: &Path,
) -> Result<Vec<String>, AppError> {
    fs::create_dir_all(staging_dir)?;

    let mut used_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut result = Vec::new();

    for (source_path, desired_name) in source_paths {
        let actual_name = if staging_dir.join(desired_name).exists() || used_names.contains(desired_name) {
            // Generate a unique name with numeric suffix
            let stem = Path::new(desired_name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = Path::new(desired_name)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            let mut counter = 1u32;
            loop {
                let candidate = format!("{}_{}{}", stem, counter, ext);
                if !staging_dir.join(&candidate).exists() && !used_names.contains(&candidate) {
                    break candidate;
                }
                counter += 1;
            }
        } else {
            desired_name.clone()
        };
        used_names.insert(actual_name.clone());

        let src = Path::new(source_path);
        let dst = staging_dir.join(&actual_name);
        fs::copy(src, &dst)?;
        result.push(actual_name);
    }

    Ok(result)
}

/// Check if any file in the manifest has a recognized mod extension (.pak, .ucas, .utoc).
pub fn has_recognized_mod_files(paths: &[String]) -> bool {
    let extensions = [".pak", ".ucas", ".utoc"];
    paths.iter().any(|p| {
        let lower = p.to_lowercase();
        extensions.iter().any(|ext| lower.ends_with(ext))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Helper: create a zip file with the given entries (path -> content bytes)
    fn create_test_zip(dir: &Path, name: &str, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
        let zip_path = dir.join(name);
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        for (path, content) in entries {
            writer.start_file(*path, options).unwrap();
            writer.write_all(content).unwrap();
        }

        writer.finish().unwrap();
        zip_path
    }

    #[test]
    fn extract_zip_returns_manifest_with_forward_slashes() {
        let tmp = TempDir::new().unwrap();
        let zip_dir = tmp.path().join("zips");
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&zip_dir).unwrap();
        fs::create_dir_all(&staging).unwrap();

        let zip_path = create_test_zip(
            &zip_dir,
            "test.zip",
            &[
                ("readme.txt", b"hello"),
                ("subdir/file.pak", b"pak data"),
            ],
        );

        let manifest = extract_zip_to_staging(&zip_path, &staging).unwrap();

        assert_eq!(manifest.len(), 2);
        assert!(manifest.contains(&"readme.txt".to_string()));
        assert!(manifest.contains(&"subdir/file.pak".to_string()));

        // Verify files actually exist on disk
        assert!(staging.join("readme.txt").exists());
        assert!(staging.join("subdir/file.pak").exists());
    }

    #[test]
    fn extract_zip_skips_directory_entries() {
        let tmp = TempDir::new().unwrap();
        let zip_dir = tmp.path().join("zips");
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&zip_dir).unwrap();
        fs::create_dir_all(&staging).unwrap();

        // Create zip with a directory entry and a file entry
        let zip_path = zip_dir.join("test.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        // Add directory entry
        writer.add_directory("mydir/", options).unwrap();
        // Add file inside directory
        writer.start_file("mydir/data.txt", options).unwrap();
        writer.write_all(b"data").unwrap();
        writer.finish().unwrap();

        let manifest = extract_zip_to_staging(&zip_path, &staging).unwrap();

        // Only the file, not the directory
        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest[0], "mydir/data.txt");
    }

    #[test]
    fn extract_zip_rejects_path_traversal() {
        let tmp = TempDir::new().unwrap();
        let zip_dir = tmp.path().join("zips");
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&zip_dir).unwrap();
        fs::create_dir_all(&staging).unwrap();

        // Create zip with path traversal entry using raw API
        let zip_path = zip_dir.join("evil.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        // Normal file
        writer.start_file("safe.txt", options).unwrap();
        writer.write_all(b"safe").unwrap();
        // Path traversal attempt
        writer.start_file("../../../etc/passwd", options).unwrap();
        writer.write_all(b"evil").unwrap();
        writer.finish().unwrap();

        let manifest = extract_zip_to_staging(&zip_path, &staging).unwrap();

        // Only the safe file should be in manifest
        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest[0], "safe.txt");

        // Evil file should NOT exist outside staging
        assert!(!tmp.path().join("etc").exists());
    }

    #[test]
    fn partition_files_separates_option_folders() {
        let paths = vec![
            "readme.txt".to_string(),
            "data/main.pak".to_string(),
            "Option_HighRes/textures/tex.pak".to_string(),
            "Option_HighRes/textures/tex.utoc".to_string(),
            "option_lowres/textures/tex.pak".to_string(),
        ];

        let (main_files, sub_mods) = partition_files(&paths);

        assert_eq!(main_files.len(), 2);
        assert!(main_files.contains(&"readme.txt".to_string()));
        assert!(main_files.contains(&"data/main.pak".to_string()));

        assert_eq!(sub_mods.len(), 2);
        assert_eq!(sub_mods["Option_HighRes"].len(), 2);
        assert!(sub_mods["Option_HighRes"].contains(&"textures/tex.pak".to_string()));
        assert!(sub_mods["Option_HighRes"].contains(&"textures/tex.utoc".to_string()));
        assert_eq!(sub_mods["option_lowres"].len(), 1);
        assert!(sub_mods["option_lowres"].contains(&"textures/tex.pak".to_string()));
    }

    #[test]
    fn partition_files_no_options_returns_all_main() {
        let paths = vec![
            "readme.txt".to_string(),
            "data/main.pak".to_string(),
        ];

        let (main_files, sub_mods) = partition_files(&paths);

        assert_eq!(main_files.len(), 2);
        assert!(sub_mods.is_empty());
    }

    #[test]
    fn copy_files_to_staging_basic() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("sources");
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&source_dir).unwrap();

        // Create source files
        fs::write(source_dir.join("config.ini"), b"config data").unwrap();
        fs::write(source_dir.join("readme.txt"), b"readme data").unwrap();

        let inputs = vec![
            (source_dir.join("config.ini").display().to_string(), "config.ini".to_string()),
            (source_dir.join("readme.txt").display().to_string(), "readme.txt".to_string()),
        ];

        let result = copy_files_to_staging(&inputs, &staging).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "config.ini");
        assert_eq!(result[1], "readme.txt");
        assert!(staging.join("config.ini").exists());
        assert!(staging.join("readme.txt").exists());
    }

    #[test]
    fn copy_files_to_staging_handles_name_collisions() {
        let tmp = TempDir::new().unwrap();
        let source_dir = tmp.path().join("sources");
        let staging = tmp.path().join("staging");
        fs::create_dir_all(&source_dir).unwrap();

        // Create source files with same desired names
        fs::write(source_dir.join("file_a.ini"), b"data a").unwrap();
        fs::write(source_dir.join("file_b.ini"), b"data b").unwrap();

        let inputs = vec![
            (source_dir.join("file_a.ini").display().to_string(), "config.ini".to_string()),
            (source_dir.join("file_b.ini").display().to_string(), "config.ini".to_string()),
        ];

        let result = copy_files_to_staging(&inputs, &staging).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "config.ini");
        assert_eq!(result[1], "config_1.ini");
        assert!(staging.join("config.ini").exists());
        assert!(staging.join("config_1.ini").exists());

        // Verify content is correct
        assert_eq!(fs::read_to_string(staging.join("config.ini")).unwrap(), "data a");
        assert_eq!(fs::read_to_string(staging.join("config_1.ini")).unwrap(), "data b");
    }

    #[test]
    fn has_recognized_mod_files_detects_extensions() {
        assert!(has_recognized_mod_files(&["file.pak".to_string()]));
        assert!(has_recognized_mod_files(&["file.ucas".to_string()]));
        assert!(has_recognized_mod_files(&["file.utoc".to_string()]));
        assert!(has_recognized_mod_files(&["dir/FILE.PAK".to_string()]));
        assert!(!has_recognized_mod_files(&["readme.txt".to_string()]));
        assert!(!has_recognized_mod_files(&["config.ini".to_string()]));
        assert!(!has_recognized_mod_files(&[]));
    }
}
