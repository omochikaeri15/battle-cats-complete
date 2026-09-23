use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::error;

use crate::common::process;

use super::{get_ffmpeg_path, vp9_crf, vp9_speed, x264_crf, x264_preset};

const PROGRESS_KEY: &str = "out_time_us=";
const AAC_BITRATE: &str = "192k";
const OPUS_BITRATE: &str = "160k";

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Mkv,
    Webm,
}

impl VideoFormat {
    pub const ALL: [Self; 3] = [Self::Mp4, Self::Mkv, Self::Webm];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
            Self::Webm => "webm",
        }
    }

    fn container(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "matroska",
            Self::Webm => "webm",
        }
    }

    fn video_codec(self, quality: Quality) -> Vec<String> {
        match self {
            Self::Mp4 | Self::Mkv => ["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf"].into_iter().map(str::to_owned).chain([x264_crf(quality.quality), "-preset".to_owned(), x264_preset(quality.compression).to_owned()]).collect(),
            Self::Webm => ["-c:v", "libvpx-vp9", "-pix_fmt", "yuv420p", "-b:v", "0", "-deadline", "good", "-row-mt", "1", "-crf"]
                .into_iter()
                .map(str::to_owned)
                .chain([vp9_crf(quality.quality), "-cpu-used".to_owned(), vp9_speed(quality.compression)])
                .collect(),
        }
    }

    fn audio_codec(self) -> [&'static str; 4] {
        match self {
            Self::Mp4 | Self::Mkv => ["-c:a", "aac", "-b:a", AAC_BITRATE],
            Self::Webm => ["-c:a", "libopus", "-b:a", OPUS_BITRATE],
        }
    }
}

impl fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mp4 => "MP4",
            Self::Mkv => "MKV",
            Self::Webm => "WebM",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quality {
    pub quality: u32,
    pub compression: u32,
}

impl Quality {
    pub const DEFAULT_QUALITY: u32 = 80;
    pub const DEFAULT_COMPRESSION: u32 = 30;
}

impl Default for Quality {
    fn default() -> Self {
        Self { quality: Self::DEFAULT_QUALITY, compression: Self::DEFAULT_COMPRESSION }
    }
}

pub struct VideoWriter {
    child: Child,
    stdin: Option<ChildStdin>,
}

impl VideoWriter {
    pub fn start(width: u32, height: u32, fps: u32, format: VideoFormat, quality: Quality, out: &Path) -> Result<Self, String> {
        let ffmpeg = get_ffmpeg_path().ok_or("the ffmpeg add-on is not installed")?;
        let size = format!("{width}x{height}");
        let rate = fps.to_string();
        let target = out.to_string_lossy().into_owned();
        let mut arguments: Vec<&str> =
            vec!["-nostdin", "-f", "rawvideo", "-pixel_format", "rgba", "-video_size", &size, "-framerate", &rate, "-i", "-"];

        let codec = format.video_codec(quality);

        arguments.extend(codec.iter().map(String::as_str));
        arguments.extend(["-f", format.container(), "-y", &target]);

        let mut child = process::command(ffmpeg)
            .args(&arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|failure| format!("ffmpeg could not be started: {failure}"))
            .inspect_err(|message| error!("{message}"))?;
        let stdin = child.stdin.take();

        Ok(Self { child, stdin })
    }

    pub fn frame(&mut self, rgba: &[u8]) -> Result<(), String> {
        let stdin = self.stdin.as_mut().ok_or("ffmpeg's input is already closed")?;

        stdin.write_all(rgba).map_err(|failure| format!("a frame could not be handed to ffmpeg: {failure}"))
    }

    pub fn finish(mut self) -> Result<(), String> {
        drop(self.stdin.take());

        let status = self.child.wait().map_err(|failure| format!("ffmpeg could not be awaited: {failure}"))?;

        if status.success() {
            return Ok(());
        }

        error!("ffmpeg finished the replay video with {status}");

        Err(format!("ffmpeg finished with {status}"))
    }

    pub fn abort(mut self) {
        drop(self.stdin.take());

        if let Err(failure) = self.child.kill() {
            error!("ffmpeg could not be stopped: {failure}");
        }

        let _ = self.child.wait();
    }
}

pub fn mux(video: &Path, audio: &Path, format: VideoFormat, out: &Path, length: Duration, emit: impl Fn(f32), abort: &AtomicBool) -> Result<bool, String> {
    let ffmpeg = get_ffmpeg_path().ok_or("the ffmpeg add-on is not installed")?;
    let video = video.to_string_lossy().into_owned();
    let audio = audio.to_string_lossy().into_owned();
    let target = out.to_string_lossy().into_owned();
    let mut arguments: Vec<&str> =
        vec!["-nostdin", "-nostats", "-progress", "pipe:1", "-i", &video, "-i", &audio, "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy"];

    arguments.extend(format.audio_codec());

    if format == VideoFormat::Mp4 {
        arguments.extend(["-movflags", "+faststart"]);
    }

    arguments.extend(["-f", format.container(), "-y", &target]);

    let mut child = process::command(ffmpeg)
        .args(&arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|failure| format!("ffmpeg could not be started: {failure}"))?;
    let total = length.as_micros().max(1) as f64;

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if abort.load(Ordering::Relaxed) {
                if let Err(failure) = child.kill() {
                    error!("ffmpeg could not be stopped: {failure}");
                }

                let _ = child.wait();

                if let Err(failure) = fs::remove_file(out) {
                    error!("Partial video {} could not be removed: {failure}", out.display());
                }

                return Ok(false);
            }

            if let Some(done) = line.strip_prefix(PROGRESS_KEY).and_then(|value| value.trim().parse::<f64>().ok()) {
                emit((done / total).clamp(0.0, 1.0) as f32);
            }
        }
    }

    let status = child.wait().map_err(|failure| format!("ffmpeg could not be awaited: {failure}"))?;

    if status.success() {
        return Ok(true);
    }

    error!("ffmpeg could not add the replay's sound: {status}");

    Err(format!("ffmpeg could not add the sound ({status})"))
}
