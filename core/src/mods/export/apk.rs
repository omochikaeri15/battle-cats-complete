use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use zip::ZipArchive;
use tracing::{debug, error, info, trace, warn};
use resand::res_value::ResValueType;

use crate::mods::logic::state::ModDataState;
use crate::global::region::Region;
use crate::data::utilities::keys;
use crate::settings::logic::state::{Settings, ExportBehavior};
use crate::addons::apkeditor::xapk;
use crate::mods::export::{modify, sign, pack};
use crate::mods::export::patch::{EVENT_RECEIVER, ExportEvent, spawn_log_adapter};

pub fn start_export(state: &mut ModDataState, settings: &Settings) {
    if state.export.is_busy {
        warn!("Export requested, but an export is already in progress. Ignoring request.");
        return;
    }

    info!("Starting new APK export process.");
    state.export.log_content.clear();
    state.export.is_busy = true;

    let app_title = state.export.app_title.clone();
    let suffix = state.export.package_suffix.clone();
    let export_behavior = settings.mods.export_behavior.clone();
    let enforce_keys = settings.game_data.enforce_key_validation;

    let Some(mod_folder) = state.selected_mod.clone() else {
        error!("Export aborted: No mod selected.");
        state.export.is_busy = false;
        return;
    };
    let Some(input_apk_path) = state.export.selected_apk.clone() else {
        error!("Export aborted: No APK selected.");
        state.export.is_busy = false;
        return;
    };
    let detected_region = state.export.target_region;

    let (transmitter, receiver) = mpsc::channel();
    if let Ok(mut guard) = EVENT_RECEIVER.lock() { *guard = Some(receiver); }

    thread::spawn(move || {
        let string_transmitter = spawn_log_adapter(transmitter.clone());
        let log_callback = |message: String| {
            info!("Export UI Log: {}", message);
            let _ = transmitter.send(ExportEvent::Log(message));
        };

        debug!("Verifying encryption keys. Enforce: {}", enforce_keys);
        let user_keys = match keys::verify(enforce_keys, &string_transmitter) {
            Ok(keys) => {
                trace!("Keys successfully verified.");
                keys
            },
            Err(error) => {
                error!("Key verification failed: {}", error);
                let _ = transmitter.send(ExportEvent::Error(error));
                return;
            }
        };

        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("../../../.."));
        let mod_dir = base_dir.join("mods").join(&mod_folder);
        let export_dir = base_dir.join("exports");
        let app_dir = mod_dir.join("app");
        let temp_bin_dir = app_dir.join("binaries");
        let assets_dir = app_dir.join("assets");
        let xapk_dir = app_dir.join("xapk");

        let icons_dir = mod_dir.join("icons");

        debug!("Preparing file structure in {}", app_dir.display());
        let _ = fs::remove_dir_all(&app_dir);
        let _ = fs::create_dir_all(&export_dir);
        let _ = fs::create_dir_all(&temp_bin_dir);
        let _ = fs::create_dir_all(&assets_dir);

        let mut working_apk = input_apk_path.clone();
        let extension = input_apk_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        if extension == "xapk" || extension == "apkm" || extension == "apks" {
            log_callback("Merging split APKs...".to_string());
            let _ = fs::create_dir_all(&xapk_dir);
            let merged_temp_path = xapk_dir.join("merged_xapk.apk");

            if let Err(error) = xapk::merge_xapk(&working_apk, &merged_temp_path, &log_callback) {
                error!("XAPK Merge failed: {}", error);
                let _ = transmitter.send(ExportEvent::Error(error.to_string()));
                return;
            }
            trace!("XAPK successfully merged to {:?}", merged_temp_path);
            working_apk = merged_temp_path;
        }

        log_callback("Analyzing APK identity...".to_string());
        let manifest_path = temp_bin_dir.join("AndroidManifest.xml");
        let arsc_path = temp_bin_dir.join("resources.arsc");
        let mut extracted_arsc = false;

        let source_file = match fs::File::open(&working_apk) {
            Ok(file) => file,
            Err(error) => {
                error!("Failed to open APK {:?}: {}", working_apk, error);
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to open APK: {}", error)));
                return;
            }
        };

        let mut archive = match ZipArchive::new(source_file) {
            Ok(archive_instance) => archive_instance,
            Err(error) => {
                error!("Failed to read APK archive: {}", error);
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to read APK archive: {}", error)));
                return;
            }
        };

        debug!("Extracting Manifest & ARSC for look-ahead check...");
        for index in 0..archive.len() {
            let Ok(mut archive_file) = archive.by_index(index) else { continue; };
            let file_name = archive_file.name();

            if file_name == "AndroidManifest.xml" {
                if let Ok(mut output_file) = fs::File::create(&manifest_path) {
                    let _ = std::io::copy(&mut archive_file, &mut output_file);
                    trace!("Extracted AndroidManifest.xml");
                }
            } else if file_name == "resources.arsc" {
                if let Ok(mut output_file) = fs::File::create(&arsc_path) {
                    let _ = std::io::copy(&mut archive_file, &mut output_file);
                    extracted_arsc = true;
                    trace!("Extracted resources.arsc");
                }
            }
        }
        drop(archive);

        let mut apk_editor = match modify::ApkEditor::from_paths(&manifest_path, if extracted_arsc { Some(arsc_path.as_path()) } else { None }) {
            Ok(editor) => {
                trace!("ApkEditor initialized successfully.");
                editor
            },
            Err(error) => {
                error!("APK Editor initialization failed: {}", error);
                let _ = transmitter.send(ExportEvent::Error(format!("Failed to parse APK binaries: {}", error)));
                return;
            }
        };

        let mut is_update = false;
        let target_package = format!("jp.co.ponos.battlecats{}", suffix.trim());

        match export_behavior {
            ExportBehavior::Automatic => {
                trace!("Using Automatic export behavior. Scanning package identity...");
                let root_elem = apk_editor.manifest.root.get_element(&["manifest"], &apk_editor.manifest.string_pool);
                let pkg_attr = root_elem.and_then(|root| root.get_attribute("package", &apk_editor.manifest.string_pool));

                if let Some(attr) = pkg_attr
                    && let ResValueType::String(ref string_value) = attr.typed_value.data {
                    let current_pkg = string_value.resolve(&apk_editor.manifest.string_pool).unwrap_or_default().to_string();
                    debug!("Found package: {} | Target package: {}", current_pkg, target_package);
                    if current_pkg == target_package {
                        is_update = true;
                        info!("Package identity already matches: {}", target_package);
                    }
                }
            },
            ExportBehavior::Create => {
                info!("Export behavior strictly set to Create. Bypassing scan.");
                is_update = false;
            },
            ExportBehavior::Update => {
                info!("Export behavior strictly set to Update. Bypassing scan.");
                is_update = true;
            }
        }

        let final_id = if is_update {
            if export_behavior == ExportBehavior::Automatic {
                log_callback("Package identity matches target APK.".to_string());
            }
            log_callback("Updating target APK...".to_string());
            target_package
        } else {
            if export_behavior == ExportBehavior::Automatic {
                log_callback("New package identity found.".to_string());
            }
            log_callback("Creating new APK...".to_string());

            match apk_editor.apply_patches(&suffix, &app_title) {
                Ok(new_package_id) => {
                    debug!("Patches applied successfully. Saving to paths.");
                    if let Err(error) = apk_editor.save_to_paths(&manifest_path, if extracted_arsc { Some(arsc_path.as_path()) } else { None }) {
                        error!("Failed saving patched binaries: {}", error);
                        let _ = transmitter.send(ExportEvent::Error(format!("Failed to save binaries: {}", error)));
                        return;
                    }
                    new_package_id
                },
                Err(error) => {
                    error!("ApkEditor failed to apply patches: {}", error);
                    let _ = transmitter.send(ExportEvent::Error(format!("Patch Error: {}", error)));
                    return;
                }
            }
        };

        let region_key = match detected_region {
            Region::En => &user_keys.en,
            Region::Ja => &user_keys.ja,
            Region::Ko => &user_keys.ko,
            Region::Tw => &user_keys.tw,
        };

        log_callback("Packing modded game data...".to_string());
        if let Err(error) = pack::stream_pack_and_list(&mod_dir.join("patch"), &assets_dir, "DownloadLocal", region_key, &log_callback) {
            error!("Data packing failed: {}", error);
            let _ = transmitter.send(ExportEvent::Error(error));
            return;
        }

        log_callback("Rebuilding APK with patch...".to_string());
        let unsigned_apk_path = app_dir.join("unsigned_final.apk");

        match modify::inject_and_build_apk(
            &working_apk,
            &unsigned_apk_path,
            &assets_dir,
            &icons_dir,
            &mod_dir.join("loose"),
            if is_update { None } else { Some(manifest_path.as_path()) },
            if is_update || !extracted_arsc { None } else { Some(arsc_path.as_path()) }
        ) {
            Ok(count) => {
                debug!("Injection successful. {} files altered.", count);
                log_callback(format!("Injected {} files.", count));
            },
            Err(error) => {
                error!("Injection build failed: {}", error);
                let _ = transmitter.send(ExportEvent::Error(format!("Build Error: {}", error)));
                return;
            }
        }

        log_callback("Normalizing binaries...".to_string());
        let normalized_apk_path = app_dir.join("normalized_final.apk");
        if let Err(error) = modify::normalize_apk(&unsigned_apk_path, &normalized_apk_path, &working_apk) {
            error!("Normalization failed: {}", error);
            let _ = transmitter.send(ExportEvent::Error(format!("Normalization Error: {}", error)));
            return;
        }

        log_callback("Signing APK...".to_string());
        if let Err(error) = sign::sign(&normalized_apk_path, None) {
            error!("APK Signing failed: {}", error);
            let _ = transmitter.send(ExportEvent::Error(format!("Native Signing Error: {}", error)));
            return;
        }
        trace!("APK successfully signed.");

        let output_name = if app_title.trim().is_empty() { final_id } else { app_title.trim().to_string() };

        let final_apk_path = if is_update {
            input_apk_path.clone()
        } else {
            export_dir.join(format!("{}.apk", output_name))
        };

        debug!("Moving final APK to {:?}", final_apk_path);
        if let Err(error) = fs::copy(&normalized_apk_path, &final_apk_path) {
            error!("Failed copying final APK to destination: {}", error);
            let _ = transmitter.send(ExportEvent::Error(format!("Filesystem Error: {}", error)));
            return;
        }

        let _ = fs::remove_dir_all(&app_dir);

        let final_filename = final_apk_path.file_name().unwrap_or_default().to_string_lossy();
        let success_message = if is_update {
            format!("Successfully Updated {}!", final_filename)
        } else {
            format!("Successfully Built {}!", final_filename)
        };

        info!("Export completed successfully: {}", success_message);
        let _ = transmitter.send(ExportEvent::Success(success_message));
    });
}