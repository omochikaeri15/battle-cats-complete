use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use tracing::{error, info, trace};
use zip::ZipArchive;

use crate::common::solid;
use crate::domains::settings::UserKeys;

use super::decrypt;

const SOLID_LOG_INTERVAL: usize = 50;

fn format_log_name(name: &str, path: &Path) -> String {
    let cleaned = name.trim_start_matches("...\\").trim_start_matches(".../");
    if cleaned.len() > 35 {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        let trunc_stem: String = stem.chars().take(20).collect();
        if !ext.is_empty() && ext.len() <= 5 {
            format!("{}....{}", trunc_stem, ext)
        } else {
            format!("{}...", trunc_stem)
        }
    } else {
        cleaned.to_string()
    }
}

pub(crate) fn run_archive(archive_path: &Path, workspace_dir: &Path, emit_log: &(dyn Fn(String) + Sync), user_keys: &UserKeys) -> Result<(), String> {
    emit_log("Opening archive...".to_string());
    info!("Opening archive for extraction: {:?}", archive_path);

    let file = fs::File::open(archive_path).map_err(|e| {
        error!("Failed to open archive: {}", e);
        e.to_string()
    })?;

    let mut archive = ZipArchive::new(file).map_err(|e| {
        error!("Failed to parse zip archive: {}", e);
        e.to_string()
    })?;

    let mut has_pack_data = false;

    for icon in ["icon.png", "icon_foreground.png", "push_icon.png"] {
        for dpi in ["xxxhdpi", "xxhdpi", "xhdpi"] {
            let zip_path = format!("res/drawable-{}/{}", dpi, icon);

            let Ok(mut file_in_zip) = archive.by_name(&zip_path) else { continue };

            if let Ok(mut out_file) = fs::File::create(workspace_dir.join("icons").join(icon)) {
                let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                trace!("Extracted APK icon: {}", icon);
            }
            break;
        }
    }

    let total_entries = archive.len();
    let log_interval = (total_entries / 10).max(1);
    let mut extracted_count = 0;

    for i in 0..total_entries {
        let Ok(mut file_in_zip) = archive.by_index(i) else { continue };
        if file_in_zip.is_dir() { continue; }

        let name = file_in_zip.name().to_string();
        let Some((out_path, pack_part)) = destination(&name, workspace_dir) else { continue };

        has_pack_data |= pack_part;

        if write_out(&out_path, &mut file_in_zip) {
            extracted_count += 1;

            if extracted_count % log_interval == 0 || i == total_entries - 1 {
                log_extracted(&name, extracted_count, emit_log);
            }
        }
    }

    finish_pack(has_pack_data, workspace_dir, emit_log, user_keys)
}

pub(crate) fn run_solid(archive_path: &Path, workspace_dir: &Path, emit_log: &(dyn Fn(String) + Sync), user_keys: &UserKeys) -> Result<(), String> {
    emit_log("Opening archive...".to_string());
    info!("Opening solid archive for extraction: {:?}", archive_path);

    let mut archive = solid::open(archive_path).map_err(|e| {
        error!("Failed to open archive: {}", e);
        e.to_string()
    })?;
    let entries = archive.entries().map_err(|e| {
        error!("Failed to read solid archive: {}", e);
        e.to_string()
    })?;

    let mut has_pack_data = false;
    let mut extracted_count = 0usize;

    for entry in entries {
        let mut entry = entry.map_err(|e| {
            error!("Solid archive entry is damaged: {}", e);
            e.to_string()
        })?;

        if !entry.header().entry_type().is_file() { continue; }

        let Some(name) = entry.path().ok().map(|path| path.to_string_lossy().replace('\\', "/")) else { continue };
        let Some((out_path, pack_part)) = destination(&name, workspace_dir) else { continue };

        has_pack_data |= pack_part;

        if write_out(&out_path, &mut entry) {
            extracted_count += 1;

            if extracted_count.is_multiple_of(SOLID_LOG_INTERVAL) {
                log_extracted(&name, extracted_count, emit_log);
            }
        }
    }

    emit_log(format!("Extracted {} Files", extracted_count));

    finish_pack(has_pack_data, workspace_dir, emit_log, user_keys)
}

fn destination(name: &str, workspace_dir: &Path) -> Option<(PathBuf, bool)> {
    let path = Path::new(name);
    let safe_name = path.file_name()?;
    let safe_name_str = safe_name.to_string_lossy();

    if name.starts_with("patch/") || name.starts_with("loose/") || name.starts_with("icons/") {
        let nested = path.components().all(|part| matches!(part, Component::Normal(_)));

        return nested.then(|| (workspace_dir.join(name), false));
    }

    if name == "assets/DownloadLocal.list" || name == "assets/DownloadLocal.pack" {
        return Some((workspace_dir.join(safe_name), true));
    }

    if safe_name_str == "metadata.json" {
        return Some((workspace_dir.join("patch").join(safe_name), false));
    }

    if path.parent() != Some(Path::new("assets")) {
        return None;
    }

    let is_download_tsv = safe_name_str.starts_with("download") && safe_name_str.ends_with(".tsv");
    let is_junk = safe_name_str.ends_with(".dat")
        || safe_name_str.ends_with(".lock")
        || safe_name_str.ends_with(".pack")
        || safe_name_str.ends_with(".list")
        || safe_name_str.ends_with(".dex");

    (!is_download_tsv && !is_junk).then(|| (workspace_dir.join("loose").join(safe_name), false))
}

fn write_out(out_path: &Path, source: &mut impl Read) -> bool {
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let Ok(mut out_file) = fs::File::create(out_path) else { return false };

    io::copy(source, &mut out_file).is_ok()
}

fn log_extracted(name: &str, extracted_count: usize, emit_log: &(dyn Fn(String) + Sync)) {
    let path = Path::new(name);
    let safe_name_str = path.file_name().unwrap_or_default().to_string_lossy();
    let display_name = format_log_name(&safe_name_str, path);

    trace!("Extracted file: {}", display_name);
    emit_log(format!("Extracted {} Files | Streaming: {}", extracted_count, display_name));
}

fn finish_pack(has_pack_data: bool, workspace_dir: &Path, emit_log: &(dyn Fn(String) + Sync), user_keys: &UserKeys) -> Result<(), String> {
    if has_pack_data && workspace_dir.join("DownloadLocal.list").exists() && workspace_dir.join("DownloadLocal.pack").exists() {
        emit_log("Decrypting embedded pack data...".to_string());
        info!("Found embedded pack data, delegating to decrypt module.");

        let pack_file = workspace_dir.join("DownloadLocal.pack");
        decrypt::run(&pack_file, workspace_dir, emit_log, user_keys)?;

        let _ = fs::remove_file(workspace_dir.join("DownloadLocal.list"));
        let _ = fs::remove_file(workspace_dir.join("DownloadLocal.pack"));
    }

    Ok(())
}