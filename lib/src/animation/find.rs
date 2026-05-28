pub use super::utils::boundary::{calculate_showcase_bounds as bounds, calculate_initial_view as view};

use crate::animation::data::mamodel::Model;
use crate::animation::data::maanim::Animation;
use crate::animation::utils::math;
use crate::animation::graphics::{timeline, transform};
use super::utils::periodicity::calculate_difference;
use crate::common::utils::csv;

/// Synchronously scans for a perfect animation loop.
pub fn cycle(
    model: &Model,
    anim: &Animation,
    tolerance: f32,
    min_frame: Option<i32>,
    max_frame: Option<i32>,
) -> Option<(i32, i32)> {
    let mut frame_states: Vec<Vec<([f32; 9], f32)>> = Vec::new();
    let mut state_buffer = model.parts.clone();
    let mut current_frame = 0;

    let minimum_loop_length = min_frame.unwrap_or(1);

    loop {
        if let Some(max) = max_frame {
            if current_frame > max as usize {
                return None;
            }
        }

        let frame = current_frame as f32;
        let _ = timeline::animate(model, anim, frame, &mut state_buffer);
        let world_parts = transform::solve_hierarchy(&state_buffer, model);

        let mut current_state = Vec::with_capacity(world_parts.len());
        for part in &world_parts {
            current_state.push((part.matrix, part.opacity));
        }

        for (past_frame_idx, past_state) in frame_states.iter().enumerate() {
            let loop_len = current_frame as i32 - past_frame_idx as i32;

            if loop_len < minimum_loop_length { continue; }

            let diff = calculate_difference(&current_state, past_state);

            if diff <= tolerance {
                return Some((past_frame_idx as i32, current_frame as i32));
            }
        }

        frame_states.push(current_state);
        current_frame += 1;
    }
}

/// Calculates the Least Common Multiple (LCM) of all looping curves to find the true animation duration.
pub fn loop_frames(anim: &Animation) -> Option<i32> {
    let mut overall_lcm: i64 = 1;
    let mut found_looping_part = false;

    for curve in &anim.curves {
        if curve.loop_count == 1 {
            return None; // Infinite/Single loop
        }

        if curve.loop_count != 1 {
            if let (Some(first_keyframe), Some(last_keyframe)) = (curve.keyframes.first(), curve.keyframes.last()) {
                let duration = (last_keyframe.frame - first_keyframe.frame) as i32;
                if duration > 0 {
                    overall_lcm = math::lcm(overall_lcm as i32, duration);
                    if overall_lcm > 999_999 {
                        return None;
                    }
                    found_looping_part = true;
                }
            }
        }
    }

    if !found_looping_part {
        return Some(anim.max_frame);
    }

    Some(std::cmp::max(overall_lcm as i32, anim.max_frame))
}

/// Public Generic API to blindly scan a file for its duration without building a Rig
#[inline(always)]
pub fn scan_duration(bytes: impl AsRef<[u8]>) -> i32 {
    scan_duration_inner(bytes.as_ref())
}

pub fn scan_duration_inner(bytes: &[u8]) -> i32 {
    let Ok(file_content) = std::str::from_utf8(bytes) else { return 0; };
    let delimiter = csv::detect_separator(file_content);

    // Filter empty lines exactly like the old logic to maintain index integrity
    let lines: Vec<&str> = file_content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() { return 0; }

    let mut max_frame_count = 0;
    let mut i = 0;

    // Skip standard headers
    if i < lines.len() && lines[i].starts_with('[') { i += 1; }
    if i < lines.len() { i += 1; }
    if i < lines.len() { i += 1; }

    while i < lines.len() {
        let parts: Vec<&str> = lines[i].split(delimiter).collect();
        i += 1;

        // Still skip malformed lines, but do it safely
        if parts.len() < 5 { continue; }

        let loop_count: i32 = parts[2].trim().parse().unwrap_or(1);
        let repeats = std::cmp::max(loop_count, 1);

        if i >= lines.len() { break; }

        let count_line = lines[i];
        let keyframe_count: usize = count_line.split(delimiter)
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        i += 1;

        if keyframe_count > 0 {
            let first_frame_line = lines[i];
            let first_frame: i32 = first_frame_line.split(delimiter)
                .next()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            let last_idx = i + keyframe_count - 1;
            let last_frame: i32 = if last_idx < lines.len() {
                lines[last_idx].split(delimiter)
                    .next()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0)
            } else {
                0
            };

            let duration = last_frame - first_frame;
            max_frame_count = std::cmp::max((duration * repeats) + first_frame, max_frame_count);

            // CRITICAL: Jump the cursor past the keyframes!
            i += keyframe_count;
        }
    }

    max_frame_count
}