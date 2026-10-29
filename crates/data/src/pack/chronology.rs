//! Load-order weighting for pack files that overlay one another.

use std::path::{Path, PathBuf};

/// Calculates a pack file's load-order weight from its path.
///
/// The engine's naming convention decides which packs overwrite others once
/// loaded. Sorting by this weight reproduces that order.
///
/// # Arguments
/// * `path` - The path of the file being weighted.
/// * `temp_apk_dirs` - Update directories whose contents take precedence.
///
/// # Returns
/// A `u64` holding the sorting weight; higher values load last and win.
pub fn calculate_weight(path: &Path, temp_apk_dirs: &[PathBuf]) -> u64 {
    let mut weight = 5_000;
    let mut found_version = false;

    let components: Vec<_> = path.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
    
    for comp in components.iter().rev() {
        let stem = comp.rsplit_once('.').map_or(comp.as_str(), |(name, _ext)| name);
        let parts: Vec<&str> = stem.split('_').collect();

        let len = parts.len();
        if len >= 3 {
            let maybe_minor = parts[len - 1].parse::<u64>();
            let maybe_major = parts[len - 2].parse::<u64>();

            if let (Ok(major), Ok(minor)) = (maybe_major, maybe_minor) {
                weight = 100_000_000 + (major * 100) + minor;
                found_version = true;
            } else if let (Ok(major), Ok(minor)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                weight = 100_000_000 + (major * 100) + minor;
                found_version = true;
            }
        }

        if found_version {
            break;
        }
    }

    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();

    if weight == 5_000 && file_stem.ends_with("Server") {
        let chars: Vec<char> = file_stem.chars().collect();
        weight = if chars.len() > 1 && chars[0].is_ascii_uppercase() && chars[1].is_ascii_uppercase() {
            20_000 + (chars[0] as u64)
        } else {
            10_000
        };
    }

    if weight == 5_000 && (file_stem == "DataLocal" || file_stem == "UpdateLocal" || file_stem.ends_with("Local")) {
        weight = 0;
    }

    if temp_apk_dirs.iter().any(|dir| path.starts_with(dir)) {
        weight += 500_000_000;
    }

    weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_chronology_weights() {
        let temp_apk_dirs = vec![PathBuf::from("/tmp/nyanko_apk_extract")];
        assert_eq!(calculate_weight(Path::new("DataLocal.pack"), &temp_apk_dirs), 0);
        assert_eq!(calculate_weight(Path::new("UpdateLocal.pack"), &temp_apk_dirs), 0);
        assert_eq!(calculate_weight(Path::new("ENServer.pack"), &temp_apk_dirs), 20_069);
        assert_eq!(calculate_weight(Path::new("game_14_2.pack"), &temp_apk_dirs), 100_001_402);
        assert_eq!(calculate_weight(Path::new("patch/custom_ui.pack"), &temp_apk_dirs), 5_000);
        let timestamp_path = Path::new("/tmp/nyanko_apk_extract/15040000.pack");
        assert_eq!(calculate_weight(timestamp_path, &temp_apk_dirs), 500_005_000);
    }
}