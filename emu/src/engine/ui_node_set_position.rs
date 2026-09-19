use super::Sprite;

pub fn ui_node_set_position(sprite: &mut Sprite, x: f32, y: f32) {
    sprite.x = x;
    sprite.y = y;
}
