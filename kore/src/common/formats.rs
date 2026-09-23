pub mod imgcut;

use std::collections::HashMap;
use std::sync::Arc;

use image::{imageops, RgbaImage};
use nyanko::graphics::rig::SpriteCut;

#[derive(Default, Clone)]
pub struct SpriteSheet {
    pub image_data: Option<Arc<RgbaImage>>,
    pub cuts_map: HashMap<usize, SpriteCut>,
    pub sheet_name: String,
}

impl SpriteSheet {
    pub fn crop(&self, icon: usize) -> Option<RgbaImage> {
        let image = self.image_data.as_ref()?;
        let (x, y, width, height) = fitting_cut(self.cuts_map.get(&icon)?, image.width(), image.height())?;

        Some(imageops::crop_imm(image.as_ref(), x, y, width, height).to_image())
    }
}

pub fn fitting_cut(cut: &SpriteCut, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let (x, y) = (cut.x.max(0) as u32, cut.y.max(0) as u32);
    let (w, h) = (cut.width.max(0) as u32, cut.height.max(0) as u32);

    (w > 0 && h > 0 && x + w <= width && y + h <= height).then_some((x, y, w, h))
}
