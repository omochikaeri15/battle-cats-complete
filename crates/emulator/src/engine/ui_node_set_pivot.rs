use super::Sprite;

pub fn ui_node_set_pivot(sprite: &mut Sprite, x: f32, y: f32) {
    sprite.pivot_x = x;
    sprite.pivot_y = y;
}
