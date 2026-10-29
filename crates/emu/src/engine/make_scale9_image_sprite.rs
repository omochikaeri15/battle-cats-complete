use super::{Sprite, SpriteKind, matrix_identity, rect_zero};

pub fn make_scale9_image_sprite() -> Box<Sprite> {
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
    sprite.kind = SpriteKind::Scale9;

    sprite
}
