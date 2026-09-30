const FRAMES_PER_SECOND: f32 = 30.0;

pub fn label(frames: i32) -> String {
    format!("{:.2}s^{}f", frames as f32 / FRAMES_PER_SECOND, frames)
}

pub fn span(low: i32, high: i32) -> String {
    if low == high { label(low) } else { format!("{}~{}", label(low), label(high)) }
}
