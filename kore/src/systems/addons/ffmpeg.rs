pub mod encoding;
pub mod video;

use std::fs;
use std::path::PathBuf;

use tracing::{error, info};

use crate::systems::addons::DownloadConfig;
use crate::systems::addons::paths::{get_tools_dir, AddonStatus, FFMPEG_BIN};

const X264_PRESETS: [&str; 9] = ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"];
const VP9_SLOWEST: f32 = 5.0;

pub(crate) fn x264_crf(quality: u32) -> String {
    format!("{:.0}", 51.0 - (quality as f32 / 100.0 * 33.0))
}

pub(crate) fn x264_preset(compression: u32) -> &'static str {
    let index = (compression as f32 / 100.0 * 8.0).round() as usize;

    X264_PRESETS.get(index).copied().unwrap_or("veryslow")
}

pub(crate) fn vp9_crf(quality: u32) -> String {
    format!("{:.0}", 63.0 - (quality as f32 / 100.0 * 63.0))
}

pub(crate) fn vp9_speed(compression: u32) -> String {
    format!("{:.0}", (VP9_SLOWEST - compression as f32 / 100.0 * VP9_SLOWEST).clamp(0.0, VP9_SLOWEST))
}

pub struct FfmpegManager {
    pub status: AddonStatus,
}

impl Default for FfmpegManager {
    fn default() -> Self {
        Self {
            status: if is_installed() { AddonStatus::Installed } else { AddonStatus::NotInstalled },
        }
    }
}

impl FfmpegManager {
    pub fn install(&mut self) -> DownloadConfig {
        info!("Starting FFmpeg installation...");
        let asset_name = if cfg!(target_os = "windows") {
            "ffmpeg_win.zip"
        } else if cfg!(target_os = "macos") {
            "ffmpeg_mac.zip"
        } else {
            "ffmpeg_linux.zip"
        };

        self.status = AddonStatus::Downloading(0.0, "Starting...".to_string());

        DownloadConfig {
            folder_name: "ffmpeg".to_string(),
            asset_name: asset_name.to_string(),
            binary_name: FFMPEG_BIN.to_string(),
        }
    }

    pub fn uninstall(&mut self) {
        info!("Uninstalling FFmpeg...");
        let dir = get_ffmpeg_dir();

        if dir.exists()
            && let Err(err) = fs::remove_dir_all(&dir) {
            error!("Failed to remove FFmpeg directory: {}", err);
        }

        self.status = AddonStatus::NotInstalled;
    }
}

fn get_ffmpeg_dir() -> PathBuf {
    get_tools_dir().join("ffmpeg")
}

pub(crate) fn get_ffmpeg_path() -> Option<PathBuf> {
    let bin = get_ffmpeg_dir().join(FFMPEG_BIN);
    if bin.exists() { Some(bin) } else { None }
}

pub fn is_installed() -> bool {
    get_ffmpeg_path().is_some()
}