use crate::Fault;

use super::{
    AppContext, bc_log_defeated, dialog_close, get_text_texture, pick_lose_tip, request_save_data,
    text_texture_cache,
};

const SITE: &str = "game_lose_update_lambda_0";

pub fn game_lose_update_lambda_0(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    _button: i32,
) -> Result<(), Fault> {
    match event {
        5 => {
            ctx.set_i32_at(AppContext::LOSE_NO_PRESS, 0)?;
            ctx.set_i32_at(AppContext::OUTRO_PHASE, 4)?;
            ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;

            if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0
                && ctx.i32_at(AppContext::STAGE_ROW)? != 0
                && ctx.i32_at(AppContext::LOSE_RECORDED)? == 0
            {
                ctx.set_i32_at(AppContext::LOSE_RECORDED, 1)?;
            }

            ctx.set_i32_at(AppContext::LOSE_TIP_SHOWN, 1)?;

            let tip = pick_lose_tip(ctx)?;

            ctx.set_i32_at(AppContext::LOSE_TIP, tip)?;

            let mut line = 0i64;

            loop {
                let tip = ctx.i32_at(AppContext::LOSE_TIP)? as i64 as usize;
                let row = ctx.lose_rows.get(tip).ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: tip as i64,
                    limit: ctx.lose_rows.len() as i64,
                })?;

                if line >= row.len() as i32 as i64 {
                    break;
                }

                let text = row[line as usize].clone();
                let font = ctx.default_font.clone();
                let texture = get_text_texture(text_texture_cache(ctx)?, &text, &font, 0x1e, 1, 0);
                let slot = 2 + line as usize;

                *ctx.label_texts
                    .get_mut(slot)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: slot as i64,
                        limit: 0x434,
                    })? = Some(texture);

                line += 1;
            }

            bc_log_defeated(ctx, 0)?;
            ctx.set_i32_at(AppContext::BATTLE_RESUMED, 0)?;
            ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 0)?;

            request_save_data(ctx)
        }
        2 => dialog_close(ctx, dialog),
        _ => Ok(()),
    }
}
