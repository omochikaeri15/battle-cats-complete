#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct EffectSprite {
    pub pos_x: i32,
    pub pos_y: i32,
    pub frame: i32,
}

pub fn effect_sprite_init(sprite: &mut EffectSprite, pos_x: i32, pos_y: i32) {
    sprite.pos_x = pos_x;
    sprite.pos_y = pos_y;
    sprite.frame = 0;
}
