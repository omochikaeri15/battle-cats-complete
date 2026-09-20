use crate::Fault;

use super::Sprite;

pub fn ui_node_get_child(sprite: &mut Sprite, index: i32) -> Result<&mut Sprite, Fault> {
    let limit = sprite.children.len() as i64;

    sprite
        .children
        .get_mut(index as i64 as usize)
        .ok_or(Fault::index_out_of_range(index as i64, limit))
}
