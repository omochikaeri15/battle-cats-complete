use nyanko::graphics::rig::SpriteSheet as NyankoSpriteSheet;

use crate::Source;

pub fn parse(png: &Source, imgcut: &Source) -> Option<NyankoSpriteSheet> {
    let png_data = png.read().unwrap_or_default();
    let cut_data = imgcut.read().unwrap_or_default();

    NyankoSpriteSheet::parse(&png_data, &cut_data).ok()
}
