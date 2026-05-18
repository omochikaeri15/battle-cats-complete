use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use zip::ZipArchive;
use crate::features::mods::import::decrypt;
use crate::features::settings::logic::keys::UserKeys;

pub fn run_archive(archive_path: &Path, _target_dir: &Path, tx: Sender<String>, user_keys: &UserKeys) -> Result<(), String> {
    let _ = tx.send("Opening archive...".to_string());
    
    let mods_root = Path::new("mods");
    if !mods_root.exists() { fs::create_dir_all(mods_root).map_err(|e| e.to_string())?; }

    let mut mod_num = 1;
    while mods_root.join(format!("NewMod{}", mod_num)).exists() {
        mod_num += 1;
    }
    let mod_dir = mods_root.join(format!("NewMod{}", mod_num));
    fs::create_dir_all(&mod_dir).map_err(|e| e.to_string())?;

    let file = fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut has_pack_data = false;
    
    for i in 0..archive.len() {
        if let Ok(mut file_in_zip) = archive.by_index(i) {
            if file_in_zip.is_dir() { continue; }

            let name = file_in_zip.name().to_string();
            let path = Path::new(&name);
            let safe_name = path.file_name().unwrap();
            let safe_name_str = safe_name.to_string_lossy();
            
            if name == "assets/DownloadLocal.list" {
                let out_path = mod_dir.join(safe_name);
                if let Ok(mut out_file) = fs::File::create(&out_path) {
                    let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                }
                has_pack_data = true;
            } else if name == "assets/DownloadLocal.pack" {
                let out_path = mod_dir.join(safe_name);
                if let Ok(mut out_file) = fs::File::create(&out_path) {
                    let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                }
                has_pack_data = true;
                
            } else if name == "res/drawable-xxxhdpi/icon.png" || name == "res/drawable-xxxhdpi/icon_foreground.png" || name == "res/drawable-xxxhdpi/push_icon.png" {
                let icons_dir = mod_dir.join("icons");
                let _ = fs::create_dir_all(&icons_dir);
                let out_path = icons_dir.join(safe_name);
                if let Ok(mut out_file) = fs::File::create(&out_path) {
                    let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                }
                
            } else if safe_name_str == "metadata.json" {
                let patch_dir = mod_dir.join("patch");
                let _ = fs::create_dir_all(&patch_dir);
                let out_path = patch_dir.join(safe_name);
                if let Ok(mut out_file) = fs::File::create(&out_path) {
                    let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                }
                
            } else if path.parent() == Some(Path::new("assets")) {
                let is_download_tsv = safe_name_str.starts_with("download") && safe_name_str.ends_with(".tsv");
                
                let is_junk = safe_name_str.ends_with(".dat")
                    || safe_name_str.ends_with(".lock")
                    || safe_name_str.ends_with(".pack")
                    || safe_name_str.ends_with(".list")
                    || safe_name_str.ends_with(".dex");

                if !is_download_tsv && !is_junk {
                    let loose_dir = mod_dir.join("loose");
                    let _ = fs::create_dir_all(&loose_dir);
                    let out_path = loose_dir.join(safe_name);
                    if let Ok(mut out_file) = fs::File::create(&out_path) {
                        let _ = std::io::copy(&mut file_in_zip, &mut out_file);
                    }
                }
            }
        }
    }
    
    if has_pack_data && mod_dir.join("DownloadLocal.list").exists() && mod_dir.join("DownloadLocal.pack").exists() {
        let _ = tx.send("Decrypting pack data...".to_string());
        
        decrypt::run(&mod_dir, tx.clone(), user_keys)?;
        
        let _ = fs::remove_file(mod_dir.join("DownloadLocal.list"));
        let _ = fs::remove_file(mod_dir.join("DownloadLocal.pack"));
    }
    
    let mut final_name = mod_dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let meta_path = mod_dir.join("patch").join("metadata.json");

    if meta_path.exists() {
        let meta = crate::features::mods::logic::metadata::ModMetadata::load(&mod_dir);
        let safe_title = meta.title.replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..], "").trim().to_string();

        if !safe_title.is_empty() && safe_title != final_name {
            let mut attempt = safe_title.clone();
            let mut counter = 1;
            let mut new_path = mods_root.join(&attempt);

            while new_path.exists() {
                attempt = format!("{}{}", safe_title, counter);
                new_path = mods_root.join(&attempt);
                counter += 1;
            }
            if fs::rename(&mod_dir, &new_path).is_ok() {
                final_name = attempt;
            }
        }
    }

    let _ = tx.send(format!("Successfully imported mod as '{}'", final_name));
    Ok(())
}