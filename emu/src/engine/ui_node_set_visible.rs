use super::Sprite;

pub fn ui_node_set_visible(node: &mut Sprite, visible: u8) -> &mut Sprite {
    node.visible = visible;

    node
}
