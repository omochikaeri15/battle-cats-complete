use std::fs;
use std::path::Path;
use std::collections::HashSet;
use std::io::{Read, Write, Cursor};
use zip::{ZipArchive, ZipWriter};

use resand::{
    res_value::{ResValue, ResValueType},
    table::ResTable,
    xmltree::XMLTree,
};

#[derive(Debug, thiserror::Error)]
pub enum ResError {
    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("Manifest parse error: {0}")]
    Manifest(String),
    #[error("Missing required element: {0}")]
    MissingElement(&'static str),
}

pub struct ApkEditor {
    pub manifest: XMLTree,
    pub res_table: Option<ResTable>,
}

impl ApkEditor {
    pub fn from_paths(manifest_path: &Path, table_path: Option<&Path>) -> Result<Self, ResError> {
        let manifest = XMLTree::read(&mut fs::File::open(manifest_path)?)
            .map_err(|e| ResError::Manifest(e.to_string()))?;

        let res_table = match table_path {
            Some(path) if path.exists() => {
                Some(ResTable::read_all(&mut fs::File::open(path)?)
                    .map_err(|e| ResError::Manifest(e.to_string()))?)
            }
            _ => None,
        };

        Ok(Self { manifest, res_table })
    }

    pub fn save_to_paths(self, manifest_path: &Path, table_path: Option<&Path>) -> Result<(), ResError> {
        self.manifest.write(&mut fs::File::create(manifest_path)?)
            .map_err(|e| ResError::Manifest(e.to_string()))?;

        if let (Some(path), Some(table)) = (table_path, self.res_table) {
            table.write_all(&mut fs::File::create(path)?)
                .map_err(|e| ResError::Manifest(e.to_string()))?;
        }
        Ok(())
    }

    pub fn apply_patches(&mut self, suffix: &str, app_title: &str) -> Result<String, ResError> {
        let root = self.manifest.root.get_element_mut(&["manifest"], &self.manifest.string_pool)
            .ok_or(ResError::MissingElement("manifest root"))?;

        let package_attr = root.get_attribute_mut("package", &self.manifest.string_pool)
            .ok_or(ResError::MissingElement("package attribute"))?;

        let original_package = match package_attr.typed_value.data {
            ResValueType::String(ref s) => s.resolve(&mut self.manifest.string_pool).unwrap_or_default().to_string(),
            _ => return Err(ResError::MissingElement("Invalid package string format")),
        };

        let mut parts: Vec<&str> = original_package.split('.').collect();
        if !parts.is_empty() {
            parts.pop();
        }
        let new_tail = format!("battlecats{}", suffix.trim());
        parts.push(&new_tail);
        let new_package_name = parts.join(".");

        package_attr.write_string(new_package_name.as_str().into(), &mut self.manifest.string_pool);

        if let Some(app_elem) = self.manifest.root.get_element_mut(&["manifest", "application"], &self.manifest.string_pool) {
            app_elem.insert_attribute(
                "extractNativeLibs".into(),
                ResValue::new_bool(true),
                &mut self.manifest.string_pool,
                self.manifest.resource_map.as_mut(),
                Some(0x010104ea.into()),
            );

            if !app_title.trim().is_empty() {
                if let Some(label_attr) = app_elem.get_attribute_mut("label", &self.manifest.string_pool) {
                    label_attr.write_string(app_title.trim().into(), &mut self.manifest.string_pool);
                } else {
                    app_elem.insert_attribute(
                        "label".into(),
                        ResValue::new_str(app_title.trim().into(), &mut self.manifest.string_pool),
                        &mut self.manifest.string_pool,
                        self.manifest.resource_map.as_mut(),
                        Some(0x01010001.into()),
                    );
                }
            }
        }

        if let Some(ref mut table) = self.res_table {
            if let Some(package) = table.packages.first_mut() {
                package.name = new_package_name.clone();
            }
        }

        Ok(new_package_name)
    }
}

pub fn inject_and_build_apk(
    source_apk: &Path,
    output_apk: &Path,
    assets_dir: &Path,
    icons_dir: &Path,
    loose_dir: &Path,
    patched_manifest: Option<&Path>,
    patched_arsc: Option<&Path>,
) -> Result<usize, String> {
    let source_file = fs::File::open(source_apk).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(source_file).map_err(|e| e.to_string())?;

    let destination_file = fs::File::create(output_apk).map_err(|e| e.to_string())?;
    let mut zip_writer = ZipWriter::new(destination_file);

    let mut injected_count = 0;

    // Pre-calculate all files we plan to inject
    let mut files_to_inject = HashSet::new();
    if patched_manifest.is_some() {
        files_to_inject.insert("AndroidManifest.xml".to_string());
    }
    if patched_arsc.is_some() {
        files_to_inject.insert("resources.arsc".to_string());
    }

    if assets_dir.exists() {
        for entry in fs::read_dir(assets_dir).unwrap().flatten() {
            if entry.path().is_file() {
                files_to_inject.insert(format!("assets/{}", entry.file_name().to_string_lossy()));
            }
        }
    }

    if loose_dir.exists() {
        for entry in fs::read_dir(loose_dir).unwrap().flatten() {
            if entry.path().is_file() {
                files_to_inject.insert(format!("assets/{}", entry.file_name().to_string_lossy()));
            }
        }
    }

    let has_custom_icon = icons_dir.join("icon.png").exists();
    let has_custom_foreground = icons_dir.join("icon_foreground.png").exists();
    let has_custom_push = icons_dir.join("push_icon.png").exists();

    // Track which resource folders actually exist in the original APK
    let mut existing_res_folders = HashSet::new();

    // First Pass: Copy all original files EXCEPT the ones we are replacing
    for index in 0..archive.len() {
        let archive_file = archive.by_index(index).unwrap();
        let file_name = archive_file.name().to_string();

        if file_name.starts_with("META-INF/") {
            continue;
        }

        // Map existing resource directories so we don't inject unmapped folders later
        if file_name.starts_with("res/") {
            if let Some(parent) = Path::new(&file_name).parent() {
                existing_res_folders.insert(parent.to_string_lossy().replace("\\", "/"));
            }
        }

        if files_to_inject.contains(&file_name) {
            continue;
        }

        let short_name = Path::new(&file_name).file_name().unwrap_or_default().to_string_lossy();
        if file_name.starts_with("res/") {
            if short_name == "icon.png" && has_custom_icon { continue; }
            if short_name == "icon_foreground.png" && has_custom_foreground { continue; }
            if short_name == "push_icon.png" && has_custom_push { continue; }
        }

        zip_writer.raw_copy_file(archive_file).map_err(|e| e.to_string())?;
    }

    let mut inject_file = |local_path: &Path, zip_path: &str, store: bool| -> Result<(), String> {
        if !local_path.exists() { return Ok(()); }

        let file_data = fs::read(local_path).map_err(|e| e.to_string())?;
        let compression = if store { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };
        let options = zip::write::SimpleFileOptions::default().compression_method(compression);

        zip_writer.start_file(zip_path, options).map_err(|e| e.to_string())?;
        zip_writer.write_all(&file_data).map_err(|e| e.to_string())?;
        injected_count += 1;
        Ok(())
    };

    // Inject Patched Binaries
    if let Some(manifest) = patched_manifest {
        inject_file(manifest, "AndroidManifest.xml", false)?;
    }
    if let Some(arsc) = patched_arsc {
        inject_file(arsc, "resources.arsc", true)?;
    }

    // Inject Assets & Packs
    if assets_dir.exists() {
        for entry in fs::read_dir(assets_dir).unwrap().flatten() {
            if entry.path().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let store = name.ends_with(".pack") || name.ends_with(".list");
                inject_file(&entry.path(), &format!("assets/{}", name), store)?;
            }
        }
    }

    // Inject Loose Files
    if loose_dir.exists() {
        for entry in fs::read_dir(loose_dir).unwrap().flatten() {
            if entry.path().is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                inject_file(&entry.path(), &format!("assets/{}", name), true)?;
            }
        }
    }

    // Safe In-Memory Icon Injection
    if icons_dir.exists() {
        let icon_blueprints = vec![
            ("icon.png", 192, 144, 96, has_custom_icon),
            ("icon_foreground.png", 432, 324, 216, has_custom_foreground),
            ("push_icon.png", 96, 72, 48, has_custom_push),
        ];

        for (file_name, xxxhdpi, xxhdpi, xhdpi, exists) in icon_blueprints {
            if !exists { continue; }

            let source_path = icons_dir.join(file_name);
            if let Ok(source_image) = image::open(&source_path) {
                let target_resolutions = vec![
                    ("drawable-xxxhdpi", xxxhdpi),
                    ("drawable-xxhdpi", xxhdpi),
                    ("drawable-xhdpi", xhdpi),
                    ("mipmap-xxxhdpi", xxxhdpi),
                    ("mipmap-xxhdpi", xxhdpi),
                    ("mipmap-xhdpi", xhdpi),
                ];

                for (folder, size) in target_resolutions {
                    let res_folder = format!("res/{}", folder);

                    if !existing_res_folders.contains(&res_folder) {
                        continue;
                    }

                    let zip_path = format!("{}/{}", res_folder, file_name);
                    let scaled_image = source_image.resize_exact(size, size, image::imageops::FilterType::Lanczos3);

                    let mut cursor = Cursor::new(Vec::new());
                    if scaled_image.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
                        if zip_writer.start_file(&zip_path, options).is_ok() {
                            let _ = zip_writer.write_all(&cursor.into_inner());
                            injected_count += 1;
                        }
                    }
                }
            }
        }
    }

    zip_writer.finish().map_err(|e| e.to_string())?;
    Ok(injected_count)
}

pub fn normalize_apk(input_apk: &Path, output_apk: &Path, original_apk: &Path) -> Result<(), String> {
    let mut stored_files_map = HashSet::new();

    let original_file = fs::File::open(original_apk).map_err(|error| format!("Failed to open original APK: {}", error))?;
    let mut original_archive = ZipArchive::new(original_file).map_err(|error| format!("Failed to read original APK: {}", error))?;

    for index in 0..original_archive.len() {
        let mut archive_file = original_archive.by_index(index).map_err(|error| error.to_string())?;
        let file_name = archive_file.name().to_string();

        if !file_name.ends_with(".apk") {
            if archive_file.compression() == zip::CompressionMethod::Stored {
                stored_files_map.insert(file_name);
            }
            continue;
        }

        let mut apk_data = Vec::new();
        archive_file.read_to_end(&mut apk_data).map_err(|error| error.to_string())?;

        let cursor = Cursor::new(apk_data);
        let mut nested_archive = ZipArchive::new(cursor).map_err(|error| error.to_string())?;

        for nested_index in 0..nested_archive.len() {
            let nested_file = nested_archive.by_index(nested_index).map_err(|error| error.to_string())?;
            if nested_file.compression() == zip::CompressionMethod::Stored {
                stored_files_map.insert(nested_file.name().to_string());
            }
        }
    }

    let source_file = fs::File::open(input_apk).map_err(|error| format!("Failed to open APK: {}", error))?;
    let mut archive = ZipArchive::new(source_file).map_err(|error| format!("Failed to read APK archive: {}", error))?;

    let destination_file = fs::File::create(output_apk).map_err(|error| format!("Failed to create normalized APK: {}", error))?;
    let mut zip_writer = ZipWriter::new(destination_file);

    let uncompressed_extensions = ["dex", "arsc", "so", "pack", "list", "ogg"];

    for index in 0..archive.len() {
        let mut archive_file = archive.by_index(index).unwrap();
        let file_name = archive_file.name().to_string();
        let file_extension = Path::new(&file_name).extension().and_then(|extension_str| extension_str.to_str()).unwrap_or("");

        let force_store = uncompressed_extensions.contains(&file_extension);
        let is_already_stored = stored_files_map.contains(&file_name);

        if !force_store && !is_already_stored {
            zip_writer.raw_copy_file(archive_file).map_err(|error| error.to_string())?;
            continue;
        }

        let mut file_data = Vec::new();
        archive_file.read_to_end(&mut file_data).map_err(|error| format!("Failed reading {}: {}", file_name, error))?;

        let byte_alignment = if file_extension == "so" { 4096 } else { 4 };

        let write_options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .with_alignment(byte_alignment);

        zip_writer.start_file(&file_name, write_options).map_err(|error| error.to_string())?;
        zip_writer.write_all(&file_data).map_err(|error| error.to_string())?;
    }

    zip_writer.finish().map_err(|error| error.to_string())?;
    Ok(())
}