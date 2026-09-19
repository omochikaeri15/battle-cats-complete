use super::{
    Sprite, SpriteKind, matrix_rotate, matrix_set_translation_f, matrix_translate_f,
    scale9_image_sprite_update, transform_point_f,
};

pub fn image_sprite_update(sprite: &mut Sprite, parent: Option<([f32; 6], [f32; 2])>) {
    let mut scale = [sprite.scale_x, sprite.scale_y];
    let mut combined = [scale[0] * sprite.zoom_x, scale[1] * sprite.zoom_y];

    sprite.world_scale = scale;

    if let Some((_, parent_scale)) = parent {
        combined = [combined[0] * parent_scale[0], combined[1] * parent_scale[1]];
        scale = [scale[0] * parent_scale[0], scale[1] * parent_scale[1]];
        sprite.world_scale = scale;
    }

    let size = [
        if sprite.size[0] == -10000 {
            sprite.cut[2]
        } else {
            sprite.size[0]
        },
        if sprite.size[1] == -10000 {
            sprite.cut[3]
        } else {
            sprite.size[1]
        },
    ];
    let pivot = match sprite.anchor {
        1 => {
            sprite.pivot_x = (size[0] / 2) as f32;
            sprite.pivot_y = (size[1] / 2) as f32;
            [sprite.pivot_x, sprite.pivot_y]
        }
        2 => {
            sprite.pivot_x = 0.0;
            sprite.pivot_y = size[1] as f32;
            [0.0, sprite.pivot_y]
        }
        3 => {
            sprite.pivot_x = 0.0;
            sprite.pivot_y = 0.0;
            [0.0, 0.0]
        }
        4 => {
            sprite.pivot_x = size[0] as f32;
            sprite.pivot_y = size[1] as f32;
            [sprite.pivot_x, sprite.pivot_y]
        }
        5 => {
            sprite.pivot_x = size[0] as f32;
            sprite.pivot_y = 0.0;
            [sprite.pivot_x, 0.0]
        }
        6 => {
            sprite.pivot_x = size[0] as f32;
            sprite.pivot_y = (size[1] / 2) as f32;
            [sprite.pivot_x, sprite.pivot_y]
        }
        7 => {
            sprite.pivot_x = 0.0;
            sprite.pivot_y = (size[1] / 2) as f32;
            [0.0, sprite.pivot_y]
        }
        8 => {
            sprite.pivot_x = (size[0] / 2) as f32;
            sprite.pivot_y = 0.0;
            [sprite.pivot_x, 0.0]
        }
        9 => {
            sprite.pivot_x = (size[0] / 2) as f32;
            sprite.pivot_y = size[1] as f32;
            [sprite.pivot_x, sprite.pivot_y]
        }
        _ => [sprite.pivot_x, sprite.pivot_y],
    };
    let near = [-pivot[0] * combined[0], -pivot[1] * combined[1]];
    let far = [
        size[0] as f32 * combined[0] + near[0],
        size[1] as f32 * combined[1] + near[1],
    ];

    sprite.quad[0] = near[0];
    sprite.quad[1] = near[1];
    sprite.quad[2] = near[0];
    sprite.quad[3] = far[1];
    sprite.quad[4] = far[0];
    sprite.quad[5] = far[1];
    sprite.quad[6] = far[0];
    sprite.quad[7] = near[1];

    match parent {
        Some((parent_transform, parent_scale)) => {
            sprite.transform = parent_transform;

            let x = (sprite.x + sprite.offset_x) * parent_scale[0];
            let y = (sprite.y + sprite.offset_y) * parent_scale[1];

            matrix_translate_f(&mut sprite.transform, x, y);
        }
        None => matrix_set_translation_f(
            &mut sprite.transform,
            sprite.x + sprite.offset_x,
            sprite.y + sprite.offset_y,
        ),
    }

    matrix_rotate(&mut sprite.transform, sprite.rotation);

    for corner in 0..4usize {
        let mut out = [0i32; 2];

        transform_point_f(
            &sprite.transform,
            sprite.quad[corner * 2],
            sprite.quad[corner * 2 + 1],
            &mut out,
        );
        sprite.quad[corner * 2] = out[0] as f32;
        sprite.quad[corner * 2 + 1] = out[1] as f32;
    }

    let handed = (sprite.transform, sprite.world_scale);

    for child in sprite.children.iter_mut() {
        match child.kind {
            SpriteKind::Image => image_sprite_update(child, Some(handed)),
            SpriteKind::Scale9 => scale9_image_sprite_update(child, Some(handed)),
        }
    }
}
