use std::rc::Rc;

use super::{matrix_identity, rect_zero, Imgcut};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SpriteKind {
    #[default]
    Image,
    Scale9,
}

#[derive(Clone, Default)]
pub struct Sprite {
    pub kind: SpriteKind,
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub anchor: i32,
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub zoom_x: f32,
    pub zoom_y: f32,
    pub alpha: i32,
    pub visible: u8,
    pub children: Vec<Sprite>,
    pub transform: [f32; 6],
    pub world_scale: [f32; 2],
    pub quad: [f32; 8],
    pub sheet: Option<Rc<Imgcut>>,
    pub cut: [i32; 4],
    pub size: [i32; 2],
    pub color: [i32; 3],
    pub border: [i32; 2],
    pub border_scale: f32,
}

pub fn make_image_sprite() -> Box<Sprite> {
    let mut sprite = Box::new(Sprite::default());

    matrix_identity(&mut sprite.transform);
    sprite.offset_x = 0.0;
    sprite.offset_y = 0.0;
    sprite.anchor = 0;
    sprite.pivot_x = 0.0;
    sprite.pivot_y = 0.0;
    sprite.x = 0.0;
    sprite.y = 0.0;
    sprite.zoom_y = 1.0;
    sprite.rotation = 0.0;
    sprite.scale_x = 1.0;
    sprite.scale_y = 1.0;
    sprite.zoom_x = 1.0;
    sprite.alpha = 0xff;
    sprite.visible = 1;
    sprite.children.clear();
    sprite.kind = SpriteKind::Image;
    sprite.sheet = None;
    rect_zero(&mut sprite.cut);

    sprite
}
