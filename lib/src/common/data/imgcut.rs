use std::collections::HashMap;
use std::sync::Arc;
use image::{self, RgbaImage};
use crate::common::utils::csv;

/// A simple two-dimensional vector representing spatial coordinates or dimensions in floating-point precision.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImgVec2 {
    /// The position or magnitude along the X-axis.
    pub x: f32,
    /// The position or magnitude along the Y-axis.
    pub y: f32
}

/// A spatial bounding box defined by two points.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImgRect {
    /// The minimum (top-left) bounds of the rectangle.
    pub min: ImgVec2,
    /// The maximum (bottom-right) bounds of the rectangle.
    pub max: ImgVec2
}

/// Represents a discrete sub-region within a master texture atlas.
///
/// This structure details the specific spatial mapping required to extract a single
/// graphical element from a larger contiguous image buffer.
#[derive(Clone, Debug)]
pub struct SpriteCut {
    /// The normalized boundary coordinates mapped to the master atlas.
    pub uv_coordinates: ImgRect,
    /// The original pixel dimensions of the isolated sprite cut.
    pub original_size: ImgVec2,
    /// An optional internal identifier or name associated with the sprite component.
    pub name: String,
}

/// The comprehensive parsed representation of a graphical texture atlas and its spatial mappings.
///
/// This structure contains the raw, memory-mapped RGBA byte buffers required for external
/// hardware-accelerated texture allocation, alongside the dictionary defining how discrete
/// sprites are mathematically extracted from the atlas.
#[derive(Clone, Default)]
pub struct SpriteSheet {
    /// The decoded, gamma-corrected, and alpha-premultiplied raw pixel data of the atlas.
    pub image_data: Option<Arc<RgbaImage>>,
    /// A localized dictionary mapping absolute sprite ID integers to their respective UV coordinate bounds.
    pub cuts_map: HashMap<usize, SpriteCut>,
}

impl SpriteSheet {
    /// Parses raw image bytes and coordinate CSV bytes into a fully initialized `SpriteSheet`.
    ///
    /// The image parsing phase automatically applies a 1.9 gamma correction and pre-multiplies
    /// the RGB channels against the alpha channel to prevent dark-edge bleeding during bilinear filtering.
    ///
    /// # Arguments
    /// * `png` - The raw byte stream of the target image file.
    /// * `imgcut` - The raw byte stream of the associated CSV mapping file.
    ///
    /// # Returns
    /// Returns `Some(SpriteSheet)` upon successful parsing and allocation, or `None` if the byte streams are invalid.
    #[inline(always)]
    pub fn parse(png: impl AsRef<[u8]>, imgcut: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8]) -> Option<Self> {
        let mut image = image::load_from_memory(png).ok()?.to_rgba8();
        let image_width = image.width() as f32;
        let image_height = image.height() as f32;

        let gamma_value: f32 = 1.9;
        let inverse_gamma = 1.0 / gamma_value;
        let to_linear = |byte_value: u8| -> f32 { (byte_value as f32 / 255.0).powf(gamma_value) };
        let to_monitor = |value: f32| -> u8 { (value.powf(inverse_gamma) * 255.0 + 0.5).clamp(0.0, 255.0) as u8 };

        for pixel in image.pixels_mut() {
            let alpha_byte = pixel[3];

            if alpha_byte == 0 {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                continue;
            }

            let red_linear = to_linear(pixel[0]);
            let green_linear = to_linear(pixel[1]);
            let blue_linear = to_linear(pixel[2]);
            let alpha_linear = alpha_byte as f32 / 255.0;

            let red_monitor = to_monitor(red_linear) as f32;
            let green_monitor = to_monitor(green_linear) as f32;
            let blue_monitor = to_monitor(blue_linear) as f32;

            pixel[0] = (red_monitor * alpha_linear) as u8;
            pixel[1] = (green_monitor * alpha_linear) as u8;
            pixel[2] = (blue_monitor * alpha_linear) as u8;
        }

        let content = csv::scrub(imgcut);
        let delimiter = ',';
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        let mut sprite_count = 0;
        let mut data_start_index = 0;
        let mut found_header = false;

        for (index, line) in lines.iter().enumerate() {
            if line.contains(delimiter) {
                if found_header { break; }
                continue;
            }

            let Ok(count_val) = line.trim().parse::<usize>() else { continue; };

            if count_val > 0 && count_val < 5000 {
                sprite_count = count_val;
                data_start_index = index + 1;
                found_header = true;
            }
        }

        if !found_header || sprite_count == 0 {
            data_start_index = 0;
            sprite_count = lines.len();
        }

        let mut parsed_cuts = HashMap::new();

        for current_cut_index in 0..sprite_count {
            let line_index = data_start_index + current_cut_index;
            if line_index >= lines.len() { break; }

            let line = lines[line_index];
            let parts: Vec<&str> = line.split(delimiter).collect();

            if parts.len() < 4 { continue; }

            let Ok(cut_x) = parts[0].trim().parse::<f32>() else { continue; };
            let Ok(cut_y) = parts[1].trim().parse::<f32>() else { continue; };
            let Ok(cut_width) = parts[2].trim().parse::<f32>() else { continue; };
            let Ok(cut_height) = parts[3].trim().parse::<f32>() else { continue; };

            let uv_min = ImgVec2 { x: cut_x / image_width, y: cut_y / image_height };
            let uv_max = ImgVec2 { x: (cut_x + cut_width) / image_width, y: (cut_y + cut_height) / image_height };
            let cut_name = if parts.len() > 4 { parts[4].trim().to_string() } else { String::new() };

            parsed_cuts.insert(current_cut_index, SpriteCut {
                uv_coordinates: ImgRect { min: uv_min, max: uv_max },
                original_size: ImgVec2 { x: cut_width, y: cut_height },
                name: cut_name,
            });
        }

        Some(Self { image_data: Some(Arc::new(image)), cuts_map: parsed_cuts })
    }
}