use crate::Fault;

use super::{draw_context, draw_deck_button, recount_deploy_rarities, set_color, AppContext};

pub fn draw_deck(ctx: &mut AppContext, overlay: u8) -> Result<(), Fault> {
    recount_deploy_rarities(ctx)?;

    if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        for slot in 0..10 {
            draw_deck_button(ctx, slot, 3, overlay)?;
        }
    } else {
        let shown = ctx.i32_at(AppContext::DECK_ROW_SHOWN)?;

        if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? != 0 {
            let frame = ctx.i32_at(AppContext::DECK_ROW_SWAP_FRAME)?;

            if (shown == 0 && frame < 4) || (shown == 1 && frame > 3) {
                for slot in [5, 6, 7, 8, 9, 0, 1, 2, 3, 4] {
                    draw_deck_button(ctx, slot, 2, overlay)?;
                }
            } else {
                for slot in 0..10 {
                    draw_deck_button(ctx, slot, 2, overlay)?;
                }
            }
        } else if shown == 0 {
            if ctx.u8_at(AppContext::DECK_BACK_ROW_ENABLED)? != 0 {
                for slot in 5..10 {
                    draw_deck_button(ctx, slot, 1, overlay)?;
                }
            }

            for slot in 0..5 {
                draw_deck_button(ctx, slot, 0, overlay)?;
            }
        } else if shown == 1 {
            for slot in 0..5 {
                draw_deck_button(ctx, slot, 1, overlay)?;
            }

            for slot in 5..10 {
                draw_deck_button(ctx, slot, 0, overlay)?;
            }
        }
    }

    set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    Ok(())
}
