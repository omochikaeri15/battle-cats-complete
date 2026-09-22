use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, error, info, info_span, trace, warn};

use crate::common::job::JobEvent;
use crate::common::solid;

pub const BCM_COMPRESSION_MIN: i64 = solid::MIN_LEVEL as i64;
pub const BCM_COMPRESSION_MAX: i64 = solid::MAX_LEVEL as i64;
pub const BCM_COMPRESSION_DEFAULT: i64 = solid::DEFAULT_LEVEL as i64;

pub fn run(mod_folder: String, app_title: String, compression: i64, emit: impl Fn(JobEvent) + Sync) -> Result<(), String> {
    let _span = info_span!("bcm_export_worker", mod_id = %mod_folder).entered();

    info!("Initializing BCM Export for mod: {}", mod_folder);

    let export_name = if app_title.trim().is_empty() {
        mod_folder.clone()
    } else {
        app_title.trim().to_string()
    };

    let log_callback = |message: String| emit(JobEvent::Log(message));

    log_callback("Preparing files...".to_string());

    let current_dir_res = std::env::current_dir();
    if let Err(error) = &current_dir_res {
        error!("Failed to determine current directory: {}", error);
        return Err("Filesystem anchor lost".to_string());
    }
    let base_dir = current_dir_res.unwrap_or_else(|_| PathBuf::from("../../../../.."));

    let mod_dir = base_dir.join("mods").join(&mod_folder);
    let export_dir = base_dir.join("exports");

    if !mod_dir.exists() {
        error!("Target mod directory does not exist: {}", mod_dir.display());
        return Err("Mod directory missing".to_string());
    }

    if let Err(error) = fs::create_dir_all(&export_dir) {
        error!("Failed to ensure exports directory exists: {}", error);
        return Err(format!("Export dir error: {}", error));
    }

    let mut bcm_filename = format!("{}.bcm", export_name);
    let mut bcm_path = export_dir.join(&bcm_filename);
    let mut counter = 1;

    while bcm_path.exists() {
        bcm_filename = format!("{}{}.bcm", export_name, counter);
        bcm_path = export_dir.join(&bcm_filename);
        counter += 1;
    }

    debug!("Target BCM package path: {}", bcm_path.display());
    log_callback("Packaging mod...".to_string());

    match build_bcm_archive(&mod_dir, &bcm_path, compression, &log_callback) {
        Ok(_) => {
            let success_message = format!("\nSuccessfully Created {}!", bcm_filename);
            info!("{}", success_message);
            log_callback(success_message);
            Ok(())
        },
        Err(error) => {
            error!("BCM Packaging failed: {}", error);
            Err(format!("Packaging Error: {}", error))
        }
    }
}

fn count_target_files(source_dir: &Path, target_directories: &[&str]) -> usize {
    let mut count = 0;
    for dir_name in target_directories {
        let dir_path = source_dir.join(dir_name);
        if dir_path.is_dir()
            && let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    count += 1;
                }
            }
        }
    }
    count
}

fn build_bcm_archive(
    source_dir: &Path,
    output_file: &Path,
    compression_level: i64,
    log_callback: &impl Fn(String)
) -> Result<usize, String> {
    let clamped_level = compression_level.clamp(BCM_COMPRESSION_MIN, BCM_COMPRESSION_MAX);
    trace!("Using compression level: {}", clamped_level);

    let target_directories = ["", "patch", "loose", "icons"];
    let total_files = count_target_files(source_dir, &target_directories);

    if total_files == 0 {
        return Err("No files found to package.".to_string());
    }

    let mut archive = solid::Writer::create(output_file, clamped_level as i32).map_err(|e| format!("Failed to create BCM file: {}", e))?;
    let log_interval = (total_files / 10).max(1);
    let mut processed_files = 0usize;

    for dir_name in target_directories {
        let dir_path = source_dir.join(dir_name);

        if !dir_path.exists() || !dir_path.is_dir() {
            debug!("Skipping missing target directory: '{}'", dir_name);
            continue;
        }

        trace!("Indexing target directory: '{}'", if dir_name.is_empty() { "Mod Root" } else { dir_name });

        let entries = fs::read_dir(&dir_path).map_err(|e| format!("Failed to read dir {}: {}", dir_path.display(), e))?;

        for entry_result in entries {
            let Ok(entry) = entry_result else {
                warn!("Skipped unreadable entry in {}", dir_path.display());
                continue;
            };

            let path = entry.path();

            if !path.is_file() {
                trace!("Ignoring subfolder to enforce flat structure: {}", path.display());
                continue;
            }

            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            let archive_path = if dir_name.is_empty() {
                file_name.to_string()
            } else {
                format!("{}/{}", dir_name, file_name)
            };

            trace!("Packing file into archive: {}", archive_path);

            if let Err(e) = archive.add(&archive_path, &path) {
                warn!("Directory '{}' packaging encountered an error: {}", dir_name, e);
                return Err(format!("Failed writing archive data {}: {}", archive_path, e));
            }

            processed_files += 1;

            if processed_files.is_multiple_of(log_interval) || processed_files == total_files {
                log_callback(format!("Packed {} files | Streaming: {}", processed_files, file_name));
            }
        }
    }

    debug!("Finalizing archive stream...");
    if let Err(e) = archive.finish() {
        error!("Archive closure failed: {}", e);
        return Err(format!("Archive finalization failed: {}", e));
    }

    Ok(processed_files)
}
