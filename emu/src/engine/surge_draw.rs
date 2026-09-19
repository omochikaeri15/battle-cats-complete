use crate::{operation, Fault};

use super::{draw_context, draw_model, get_drawable_width, maanim_execute, AppContext, Entity, SURGE_TIMING};

const SITE: &str = "surge_draw";

pub fn surge_draw(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<(), Fault> {
    if ctx.surge_events.is_empty() {
        return Ok(());
    }

    let mut index = 0usize;

    while index < ctx.surge_events.len() {
        let event = &ctx.surge_events[index];

        if event.faction != faction || event.slot != slot {
            index += 1;
            continue;
        }

        let pos = event.x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);
        let width = get_drawable_width(ctx)?;
        let event = ctx.surge_events.get(index).ok_or(Fault::OutOfRange { site: SITE })?;
        let mini = event.mini;
        let elapsed = event.frame.wrapping_sub(SURGE_TIMING[1]);
        let (phase, frame) = if event.frame >= SURGE_TIMING[1] {
            let span = event.level.wrapping_mul(SURGE_TIMING[2]);

            if elapsed < span { (1, elapsed) } else { (2, elapsed.wrapping_sub(span)) }
        } else {
            (0, event.frame)
        };
        let x = operation::div_2(width.wrapping_add(-0x3c0)).wrapping_add(operation::div_10(pos));
        let y = (ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))? << 2).wrapping_add(0x1cc);
        let (model, anims) = match (faction, mini) {
            (0, false) => (&mut ctx.volcano_model, &ctx.volcano_anims),
            (0, true) => (&mut ctx.smallvolcano_model, &ctx.smallvolcano_anims),
            (1, false) => (&mut ctx.volcano_e_model, &ctx.volcano_anims),
            (1, true) => (&mut ctx.smallvolcano_e_model, &ctx.smallvolcano_anims),
            _ => return Err(Fault::IndexOutOfRange { site: SITE, index: faction as i64, limit: 2 }),
        };

        maanim_execute(model, Some(&anims[phase]), frame, 0)?;
        draw_model(draw_context(&mut ctx.draw)?, model, x, y);
        index += 1;
    }

    Ok(())
}
