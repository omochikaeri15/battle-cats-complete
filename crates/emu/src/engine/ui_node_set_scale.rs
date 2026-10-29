use super::Sprite;

pub fn ui_node_set_scale(sprite: &mut Sprite, x: f32, y: f32) {
    sprite.scale_x = x;
    sprite.scale_y = y;
}
