use crate::Fault;

use super::{AppContext, dialog_close, play_sound, sound_manager};

const SITE: &str = "game_win_update_lambda_7";

pub fn game_win_update_lambda_7(
    ctx: &mut AppContext,
    dialog: u64,
    event: i32,
    _button: i32,
) -> Result<(), Fault> {
    if event == 2 {
        play_sound(sound_manager(ctx)?, 0xb, None);

        return dialog_close(ctx, dialog);
    }

    if event != 5 {
        return Ok(());
    }

    let head = ctx
        .reward_queue
        .first_mut()
        .ok_or(Fault::NullPointer { site: SITE })?;

    if head.len() as i32 >= 2 {
        let mut cleared = 0;
        let mut index = 1usize;

        while (index as i64) < head.len() as i32 as i64 {
            if head[index] != 0 {
                if cleared > 3 {
                    break;
                }

                cleared += 1;
                head[index] = 0;
            }

            index += 1;
        }
    }

    let head = ctx
        .reward_queue
        .first()
        .ok_or(Fault::NullPointer { site: SITE })?;
    let mut remaining = false;

    if head.len() as i32 >= 2 {
        for value in head.iter().skip(1) {
            if *value != 0 {
                remaining = true;
            }
        }
    }

    if !remaining {
        ctx.reward_queue.remove(0);
    }

    ctx.set_i32_at(AppContext::OUTRO_PHASE, 3)?;
    ctx.set_i32_at(AppContext::OUTRO_FRAME, 0x1e)
}
