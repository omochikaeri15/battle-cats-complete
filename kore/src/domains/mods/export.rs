pub mod apk;
pub mod bcm;
pub mod pack;

use std::path::PathBuf;

use crate::common::region::Region;

#[derive(Clone, PartialEq, Default, Debug)]
pub enum ExportType {
    #[default]
    Apk,
    Bcm,
    Pack,
}

#[derive(Clone, Debug)]
pub struct ExportState {
    pub tab: ExportType,
    pub target_region: Region,
    pub app_title: String,
    pub package_suffix: String,
    pub pack_name: String,
    pub selected_apk: Option<PathBuf>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            tab: ExportType::Apk,
            target_region: Region::En,
            app_title: String::new(),
            package_suffix: String::new(),
            pack_name: String::new(),
            selected_apk: None,
        }
    }
}
