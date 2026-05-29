use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;

use nyanko::pack::cryptology;
use nyanko::pack::cryptology::Region as NyankoRegion;
use crate::settings::logic::keys::UserKeys;

struct PackEntry {
    name: String,
    offset: usize,
    size: usize,
}

fn map_keys_to_nyanko(user_keys: &UserKeys) -> Result<cryptology::Keys, String> {
    let owned_tuples: Vec<(NyankoRegion, String, String)> = user_keys.as_tuples().into_iter().map(|(key_string, iv, region_enum)| {
        let nyanko_region = match region_enum {
            crate::global::region::Region::En => NyankoRegion::En,
            crate::global::region::Region::Ja => NyankoRegion::Jp,
            crate::global::region::Region::Ko => NyankoRegion::Kr,
            crate::global::region::Region::Tw => NyankoRegion::Tw,
        };
        (nyanko_region, key_string, iv)
    }).collect();

    let ref_tuples: Vec<(NyankoRegion, &str, &str)> = owned_tuples.iter()
        .map(|(region, key_string, iv)| (*region, key_string.as_str(), iv.as_str()))
        .collect();

    cryptology::Keys::parse(&ref_tuples).map_err(|error| error.to_string())
}

pub fn run(pack_dir: &Path, status_sender: Sender<String>, user_keys: &UserKeys) -> Result<(), String> {
    let list_path = pack_dir.join("DownloadLocal.list");
    let pack_path = pack_dir.join("DownloadLocal.pack");

    if !list_path.exists() || !pack_path.exists() {
        return Ok(());
    }

    let patch_dir = pack_dir.join("patch");
    fs::create_dir_all(&patch_dir).map_err(|error| error.to_string())?;

    let list_data = fs::read(&list_path).map_err(|error| error.to_string())?;
    let content = cryptology::decrypt_list(&list_data).map_err(|error| error.to_string())?;

    let mut entries = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 { continue; }

        entries.push(PackEntry {
            name: parts[0].to_string(),
            offset: parts[1].parse().unwrap_or(0),
            size: parts[2].parse().unwrap_or(0),
        });
    }

    let pack_data = fs::read(&pack_path).map_err(|error| error.to_string())?;
    let nyanko_keys = map_keys_to_nyanko(user_keys)?;

    let extracted_count = AtomicUsize::new(0);

    entries.into_par_iter().for_each(|entry| {
        let aligned_size = if entry.size % 16 == 0 { entry.size } else { ((entry.size / 16) + 1) * 16 };

        if entry.offset + aligned_size <= pack_data.len() {
            let chunk = &pack_data[entry.offset .. entry.offset + aligned_size];

            let (decrypted_bytes, _) = cryptology::decrypt_chunk(chunk, &entry.name, &nyanko_keys);

            let final_data = &decrypted_bytes[..std::cmp::min(entry.size, decrypted_bytes.len())];

            let out_file = patch_dir.join(&entry.name);
            if let Some(parent) = out_file.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::write(out_file, final_data);

            extracted_count.fetch_add(1, Ordering::Relaxed);
        }
    });

    let total = extracted_count.load(Ordering::Relaxed);
    let _ = status_sender.send(format!("Successfully extracted {} files.", total));

    Ok(())
}