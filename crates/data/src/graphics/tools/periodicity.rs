//! Comparison of resolved frames, used to detect when an animation repeats.

use crate::graphics::animate::FrameData;

/// The weight orientation contributes relative to a pixel of translation.
const ORIENTATION_WEIGHT: f32 = 100.0;

/// The weight opacity contributes, matching the eight bit range it is drawn at.
const OPACITY_WEIGHT: f32 = 255.0;

/// Measures how far two resolved frames differ.
///
/// Translation, orientation, quad geometry, and opacity are weighted to
/// contribute comparably despite differing units, so one scalar threshold can
/// decide whether two frames are visually identical. Frames that draw a
/// different set of parts never compare as equal.
///
/// # Arguments
/// * `current` - The parts of the first frame, in draw order.
/// * `past` - The parts of the frame being compared against.
///
/// # Returns
/// An `f32` containing the accumulated difference, zero for identical frames and
/// infinity for frames whose parts do not correspond.
pub fn calculate_difference(current: &[FrameData], past: &[FrameData]) -> f32 {
    if current.len() != past.len() { return f32::INFINITY; }

    let mut total = 0.0;

    for (current, past) in current.iter().zip(past) {
        if current.sprite_index != past.sprite_index || current.glow != past.glow {
            return f32::INFINITY;
        }

        total += (current.final_matrix[6] - past.final_matrix[6]).abs();
        total += (current.final_matrix[7] - past.final_matrix[7]).abs();

        for at in [0, 1, 3, 4] {
            total += (current.final_matrix[at] - past.final_matrix[at]).abs() * ORIENTATION_WEIGHT;
        }

        for at in 0..current.vertices.len() {
            total += (current.vertices[at] - past.vertices[at]).abs();
        }

        total += (current.opacity - past.opacity).abs() * OPACITY_WEIGHT;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(x: f32, opacity: f32) -> FrameData {
        FrameData {
            final_matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, 20.0, 1.0],
            opacity,
            ..FrameData::default()
        }
    }

    #[test]
    fn identical_frames_do_not_differ() {
        assert_eq!(calculate_difference(&[part(10.0, 1.0)], &[part(10.0, 1.0)]), 0.0);
    }

    #[test]
    fn translation_and_opacity_both_count() {
        assert_eq!(calculate_difference(&[part(10.0, 1.0)], &[part(15.0, 0.5)]), 132.5);
    }

    #[test]
    fn frames_of_differing_length_never_match() {
        assert_eq!(calculate_difference(&[part(10.0, 1.0)], &[]), f32::INFINITY);
    }
}
