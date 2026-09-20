use crate::{Fault, operation};

use super::{
    AppContext, Entity, draw_context, draw_model, get_drawable_width, get_setting, maanim_execute,
};

pub fn explosion_draw(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    if ctx.explosion_events.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    while index < ctx.explosion_events.len() {
        let event = &ctx.explosion_events[index];

        if event.faction != faction || event.slot != slot {
            index += 1;
            continue;
        }

        let pos = event.x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
        let width = get_drawable_width(ctx)?;
        let frame = ctx
            .explosion_events
            .get(index)
            .ok_or(Fault::out_of_range())?
            .frame;

        if frame < 0 {
            index += 1;
            continue;
        }

        let depth = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))?;

        if frame < get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)? {
            let model = match faction {
                0 => &mut ctx.explosion_model,
                1 => &mut ctx.explosion_e_model,
                _ => {
                    return Err(Fault::index_out_of_range(faction as i64, 2));
                }
            };

            maanim_execute(model, Some(&ctx.explosion_anims[0]), frame, 0)?;
        } else {
            let frame =
                frame.wrapping_sub(get_setting(&ctx.settings, b"battle_explosion_frame1", 0xf)?);
            let model = match faction {
                0 => &mut ctx.explosion_model,
                1 => &mut ctx.explosion_e_model,
                _ => {
                    return Err(Fault::index_out_of_range(faction as i64, 2));
                }
            };

            maanim_execute(model, Some(&ctx.explosion_anims[1]), frame, 0)?;
        }

        let x = operation::div_2(width.wrapping_add(-0x3c0)).wrapping_add(operation::div_10(pos));
        let y = (depth << 2).wrapping_add(0x1cc);
        let model = if faction == 0 {
            &ctx.explosion_model
        } else {
            &ctx.explosion_e_model
        };

        draw_model(draw_context(&mut ctx.draw)?, model, x, y);
        index += 1;
    }

    Ok(())
}
