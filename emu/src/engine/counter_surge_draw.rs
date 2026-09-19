use crate::{Fault, operation};

use super::{AppContext, Entity, draw_context, draw_model, get_drawable_width, maanim_execute};

const SITE: &str = "counter_surge_draw";

pub fn counter_surge_draw(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    if ctx.counter_surge_events.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    while index < ctx.counter_surge_events.len() {
        let event = &ctx.counter_surge_events[index];

        if event.faction != faction || event.slot != slot {
            index += 1;
            continue;
        }

        let frame = event.frame;
        let (model, anim) = match faction {
            0 => (&mut ctx.demonsummon_model, &ctx.counter_surge_anims[0]),
            1 => (&mut ctx.demonsummon_e_model, &ctx.counter_surge_anims[1]),
            _ => {
                return Err(Fault::IndexOutOfRange {
                    site: SITE,
                    index: faction as i64,
                    limit: 2,
                });
            }
        };

        maanim_execute(model, Some(anim), frame, 0)?;

        let event = ctx
            .counter_surge_events
            .get(index)
            .ok_or(Fault::OutOfRange { site: SITE })?;
        let pos = operation::div_10(event.x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let x = operation::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pos);
        let y = (ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))? << 2)
            .wrapping_add(0x1cc);
        let model = if faction == 0 {
            &ctx.demonsummon_model
        } else {
            &ctx.demonsummon_e_model
        };

        draw_model(draw_context(&mut ctx.draw)?, model, x, y);
        index += 1;
    }

    Ok(())
}
