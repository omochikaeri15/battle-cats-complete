use super::Sprite;

pub fn ui_node_set_color(sprite: &mut Sprite, red: i32, green: i32, blue: i32) {
    sprite.color[0] = red;
    sprite.color[1] = green;
    sprite.color[2] = blue;

    for child in sprite.children.iter_mut() {
        ui_node_set_color(child, red, green, blue);
    }
}
