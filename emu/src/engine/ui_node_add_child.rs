use super::Sprite;

pub fn ui_node_add_child(parent: &mut Sprite, child: Box<Sprite>) -> &mut Sprite {
    parent.children.push(*child);

    let last = parent.children.len() - 1;

    &mut parent.children[last]
}
