use crate::{Fault, operation};

use super::{
    AppContext, Entity, Surface, call_rng, cos_deg, draw_context, draw_surface_scaled,
    find_item_by_kind, get_base_pos_x, get_castle_id, get_castle_row, get_drawable_width,
    get_item_icon, imgcut_get_height, imgcut_get_width, query_localizable, sin_deg,
    std_string_from_cstr, string_format_int, texture_cache_load,
};

const SITE: &str = "item_drop_particles";

pub fn item_drop_particles(ctx: &mut AppContext) -> Result<(), Fault> {
    let mut queue = 0usize;

    while queue < ctx.item_drop_queue.len() {
        if ctx.item_drop_queue[queue].len() == 1 {
            let item = ctx.item_drop_queue[queue][0];

            if !ctx.drop_icons.contains_key(&item) {
                let index = find_item_by_kind(ctx, 7, item)?;
                let icon = get_item_icon(ctx, index)?;
                let name = string_format_int(ctx, b"gatyaitemD_%02d_f.png", icon)?;
                let png = query_localizable(ctx, &name);
                let cut = std_string_from_cstr(b"gatyaitem_000_f.imgcut");
                let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

                ctx.drop_icons.insert(item, sheet);
            }

            let spread = call_rng(ctx, 4);
            let lift = call_rng(ctx, 4);

            ctx.item_drop_queue[queue].push(
                lift.wrapping_mul(3)
                    .wrapping_add(spread << 2)
                    .wrapping_add(2),
            );

            let turn = call_rng(ctx, 0x14);

            ctx.item_drop_queue[queue].push(0x78i32.wrapping_sub(turn));
            ctx.item_drop_queue[queue].push(0);

            let base_x = get_base_pos_x(ctx, 1)?;
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let inset = operation::div_neg_100(size.wrapping_mul(0x49c));
            let origin = operation::div_10(
                base_x
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?)
                    .wrapping_add(inset),
            );
            let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let shifted = operation::div_100(offset_x.wrapping_mul(size))
                .wrapping_add(operation::div_10(ctx.i32_at(AppContext::CAMERA_X)?));
            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
            let width = operation::div_200((size << 7).wrapping_sub(size));

            ctx.item_drop_queue[queue].push(width.wrapping_add(shifted).wrapping_add(origin));
            ctx.item_drop_queue[queue].push(0x73);

            let delay = call_rng(ctx, 0x11);

            ctx.item_drop_queue[queue].push(delay);
        }

        let timer = ctx.item_drop_queue[queue][6];

        ctx.item_drop_queue[queue][6] = timer.wrapping_sub(1);

        if timer > 0 {
            queue += 1;

            continue;
        }

        let spin = ctx.item_drop_queue[queue][1];
        let frame = ctx.item_drop_queue[queue][3];
        let turn = ctx.item_drop_queue[queue][2] as f32;
        let base_y = ctx.item_drop_queue[queue][5];
        let launch_x = ctx.item_drop_queue[queue][4] as f32;
        let across = operation::cvttss2si(cos_deg(turn) * spin as f32 * frame as f32 + launch_x);
        let height;

        if spin != 0 {
            let rise = sin_deg(turn) * spin as f32 * frame as f32 + base_y as f32;
            let fall = frame as f64 * -3.0 * frame as f64;
            let lift = operation::cvttsd2si(fall + rise as f64);

            ctx.item_drop_queue[queue][3] = frame.wrapping_add(1);

            if lift < 0 {
                let spin = ctx.item_drop_queue[queue][1];

                ctx.item_drop_queue[queue][1] = operation::div_100((spin << 4).wrapping_mul(5));
                ctx.item_drop_queue[queue][3] = 0;
                ctx.item_drop_queue[queue][4] = across;
                ctx.item_drop_queue[queue][5] = 0;
                height = 0;
            } else {
                height = lift;
            }
        } else {
            ctx.item_drop_queue[queue][3] = frame.wrapping_add(1);
            height = 0;
        }

        let item = ctx.item_drop_queue[queue][0];
        let icon = ctx.drop_icons.entry(item).or_default().clone();
        let icon = icon.as_deref().ok_or(Fault::NullPointer { site: SITE })?;
        let wide = imgcut_get_width(icon).wrapping_mul(0x3c);
        let tall = imgcut_get_height(icon).wrapping_mul(0x3c);
        let width = operation::div_100(wide);
        let height_scaled = operation::div_100(tall);
        let camera = operation::div_neg_10(ctx.i32_at(AppContext::CAMERA_X)?);
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0))
            .wrapping_add(operation::div_neg_200(wide))
            .wrapping_add(across)
            .wrapping_add(camera);
        let ground =
            operation::div_10(ctx.i32_at(AppContext::entity_field(1, 0, Entity::POS_Y))?);
        let y = ground
            .wrapping_sub(height.wrapping_add(height_scaled))
            .wrapping_add(0x26);

        draw_surface_scaled(
            draw_context(&mut ctx.draw)?,
            Surface::Sheet(icon),
            x,
            y,
            width,
            height_scaled,
        );

        queue += 1;
    }

    Ok(())
}
