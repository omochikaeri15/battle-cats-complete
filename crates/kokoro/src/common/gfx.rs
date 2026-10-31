use image::{DynamicImage, Pixel, RgbaImage, imageops};

use crate::Source;
use crate::common::assets;

const BANNER_HEIGHT: u32 = 100;
const ENEMY_ICON_SCALE: f32 = 2.6;
const ENEMY_ICON_OFFSET_X: i64 = 8;
const ENEMY_SHADOW_MARGIN: u32 = 8;
const ENEMY_ROW_HEIGHT: u32 = 50;
const SOLID_ALPHA: u8 = 150;
const BORDER_INK: u8 = 25;
const BORDER_ALPHA: u8 = 200;
const TRANSPARENT_PROBE: (u32, u32) = (311, 2);
const TRANSPARENT_OFFSET: (i64, i64) = (-3, 9);

pub fn open_image(source: &Source) -> Option<DynamicImage> {
    image::load_from_memory(&source.read().ok()?).ok()
}

pub fn trimmed(image: RgbaImage) -> Option<RgbaImage> {
    let cropped = autocrop(image);
    let (width, height) = cropped.dimensions();

    (width > 1 || height > 1).then_some(cropped)
}

pub fn boxed(image: RgbaImage, canvas: u32) -> Option<RgbaImage> {
    let cropped = autocrop(image);
    let (width, height) = cropped.dimensions();

    if width == 0 || height == 0 || canvas == 0 {
        return None;
    }

    let fitted = if width > canvas || height > canvas {
        let scale = canvas as f32 / width.max(height) as f32;
        let target_width = ((width as f32 * scale).round() as u32).clamp(1, canvas);
        let target_height = ((height as f32 * scale).round() as u32).clamp(1, canvas);

        imageops::resize(&cropped, target_width, target_height, imageops::FilterType::Lanczos3)
    } else {
        cropped
    };
    let mut boxed = RgbaImage::new(canvas, canvas);

    imageops::overlay(&mut boxed, &fitted, i64::from((canvas - fitted.width()) / 2), i64::from((canvas - fitted.height()) / 2));

    Some(boxed)
}

pub fn banner_frame() -> Option<RgbaImage> {
    image::load_from_memory(assets::UDI_F).ok().map(|frame| frame.to_rgba8())
}

pub fn cat_banner(frame: &RgbaImage, mut unit: RgbaImage) -> RgbaImage {
    let mut composed = frame.clone();
    let (probe_x, probe_y) = TRANSPARENT_PROBE;
    let (width, height) = unit.dimensions();
    let transparent = width > probe_x && height > probe_y && unit.get_pixel(probe_x, probe_y)[3] == 0;
    let (x, y) = if transparent {
        TRANSPARENT_OFFSET
    } else {
        unit = autocrop(unit);

        ((i64::from(composed.width()) - i64::from(unit.width())) / 2, (i64::from(composed.height()) - i64::from(unit.height())) / 2)
    };

    imageops::overlay(&mut composed, &unit, x, y);

    let ratio = BANNER_HEIGHT as f32 / composed.height() as f32;
    let target_width = ((composed.width() as f32 * ratio) as u32).max(1);

    imageops::resize(&composed, target_width, BANNER_HEIGHT, imageops::FilterType::Lanczos3)
}

pub fn enemy_banner(frame: &RgbaImage, unit: RgbaImage) -> RgbaImage {
    let scaled_w = ((unit.width() as f32 * ENEMY_ICON_SCALE).round() as u32).max(1);
    let scaled_h = ((unit.height() as f32 * ENEMY_ICON_SCALE).round() as u32).max(1);
    let unit = imageops::resize(&unit, scaled_w, scaled_h, imageops::FilterType::Lanczos3);
    let mut composed = frame.clone();
    let (bg_w, bg_h) = (i64::from(composed.width()), i64::from(composed.height()));
    let height = unit.height();
    let shadow_cutoff = height.saturating_sub(ENEMY_SHADOW_MARGIN);
    let (mut min_y, mut max_y, mut found) = (height, 0, false);

    for (_, y, pixel) in unit.enumerate_pixels() {
        if y < shadow_cutoff && pixel[3] > SOLID_ALPHA {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            found = true;
        }
    }

    let center_y = if found { i64::from(min_y + max_y) / 2 } else { i64::from(height) / 2 };
    let offset_y = bg_h / 2 - center_y;

    for (x, y, pixel) in unit.enumerate_pixels() {
        let dest_x = ENEMY_ICON_OFFSET_X + i64::from(x);
        let dest_y = offset_y + i64::from(y);

        if dest_x >= 0 && dest_x < bg_w && dest_y >= 0 && dest_y < bg_h {
            let under = composed.get_pixel_mut(dest_x as u32, dest_y as u32);
            let border = under[0] < BORDER_INK && under[1] < BORDER_INK && under[2] < BORDER_INK && under[3] > BORDER_ALPHA;

            if !border {
                under.blend(pixel);
            }
        }
    }

    let ratio = ENEMY_ROW_HEIGHT as f32 / composed.height() as f32;
    let target_width = ((composed.width() as f32 * ratio) as u32).max(1);

    imageops::resize(&composed, target_width, ENEMY_ROW_HEIGHT, imageops::FilterType::Lanczos3)
}

pub fn autocrop(img: RgbaImage) -> RgbaImage {
    let (width, height) = img.dimensions();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (width, height, 0, 0);
    let mut found = false;

    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel[3] > 0 {
            if x < min_x { min_x = x; }
            if x > max_x { max_x = x; }
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
            found = true;
        }
    }
    if !found { return img; }
    imageops::crop_imm(&img, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image()
}