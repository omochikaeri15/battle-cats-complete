use std::collections::HashMap;
use std::sync::Arc;
use image::{self, RgbaImage};
use crate::common::utils::csv;

#[derive(Clone, Copy, Debug, Default)]
pub struct ImgVec2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, Debug, Default)]
pub struct ImgRect { pub min: ImgVec2, pub max: ImgVec2 }

#[derive(Clone, Debug)]
pub struct SpriteCut {
    pub uv_coordinates: ImgRect,
    pub original_size: ImgVec2,
    pub name: String,
}

#[derive(Clone, Default)]
pub struct SpriteSheet {
    pub image_data: Option<Arc<RgbaImage>>,
    pub cuts_map: HashMap<usize, SpriteCut>,
}

impl SpriteSheet {
    pub fn is_ready(&self) -> bool {
        self.image_data.is_some()
    }

    #[inline(always)]
    pub fn parse(png: impl AsRef<[u8]>, imgcut: impl AsRef<[u8]>) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8]) -> Option<Self> {
        let image = image::load_from_memory(png).ok()?.to_rgba8();
        let image_width = image.width() as f32;
        let image_height = image.height() as f32;

        let content = csv::scrub(imgcut);
        let delimiter = ',';

        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();

        let mut sprite_count = 0;
        let mut data_start_index = 0;
        let mut found_header = false;

        for (index, line) in lines.iter().enumerate() {
            if !line.contains(delimiter) {
                if let Ok(count_val) = line.trim().parse::<usize>() {
                    if count_val > 0 && count_val < 5000 {
                        sprite_count = count_val;
                        data_start_index = index + 1;
                        found_header = true;
                    }
                }
            } else if found_header { break; }
        }

        if !found_header || sprite_count == 0 {
            data_start_index = 0;
            sprite_count = lines.len();
        }

        let mut parsed_cuts = HashMap::new();

        for i in 0..sprite_count {
            let line_index = data_start_index + i;
            if line_index >= lines.len() { break; }

            let line = lines[line_index];
            let parts: Vec<&str> = line.split(delimiter).collect();

            if parts.len() >= 4 {
                if let (Ok(x), Ok(y), Ok(cut_width), Ok(cut_height)) = (
                    parts[0].trim().parse::<f32>(), parts[1].trim().parse::<f32>(),
                    parts[2].trim().parse::<f32>(), parts[3].trim().parse::<f32>(),
                ) {
                    let uv_min = ImgVec2 { x: x / image_width, y: y / image_height };
                    let uv_max = ImgVec2 { x: (x + cut_width) / image_width, y: (y + cut_height) / image_height };
                    let cut_name = if parts.len() > 4 { parts[4].trim().to_string() } else { String::new() };

                    parsed_cuts.insert(i, SpriteCut {
                        uv_coordinates: ImgRect { min: uv_min, max: uv_max },
                        original_size: ImgVec2 { x: cut_width, y: cut_height },
                        name: cut_name,
                    });
                }
            }
        }

        Some(Self { image_data: Some(Arc::new(image)), cuts_map: parsed_cuts })
    }
}