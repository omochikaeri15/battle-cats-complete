use super::Sprite;

pub fn ui_node_set_zoom(node: &mut Sprite, x: f32, y: f32) {
    node.zoom_x = x;
    node.zoom_y = y;
}
