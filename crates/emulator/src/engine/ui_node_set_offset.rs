use super::Sprite;

pub fn ui_node_set_offset(sprite: &mut Sprite, x: f32, y: f32) {
    sprite.offset_x = x;
    sprite.offset_y = y;
}
