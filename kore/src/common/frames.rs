const FRAMES_PER_SECOND: f32 = 30.0;

pub fn label(frames: i32) -> String {
    format!("{:.2}s^{}f", frames as f32 / FRAMES_PER_SECOND, frames)
}
