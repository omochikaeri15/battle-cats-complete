pub mod imgcut;

use std::collections::HashMap;
use std::sync::Arc;

use codex::graphics::rig::SpriteCut;
use image::{RgbaImage, imageops};

use crate::Vfs;

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

pub fn sheets(vfs: &Vfs, name: &str) -> Vec<SpriteSheet> {
    let sheet = format!("{name}.png");

    vfs.list(&sheet)
        .into_iter()
        .filter_map(|png| {
            let stem = png.file_stem().map_or_else(|| name.to_owned(), |stem| stem.to_string_lossy().into_owned());
            let cut = vfs.locate(&format!("{stem}.imgcut")).or_else(|| vfs.locate(&format!("{name}.imgcut")))?;
            let parsed = imgcut::parse(&vfs.source(&png), &vfs.source(&cut))?;

            Some(SpriteSheet { image_data: parsed.image_data, cuts_map: parsed.cuts.into_iter().enumerate().collect(), sheet_name: stem })
        })
        .collect()
}
