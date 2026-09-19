use crate::Fault;

use super::{
    AppContext, RankingRecord, connecting_indicator_hide, dialog_show_alt, play_sound,
    query_localizable, ranking_record_rank, request_save_data, scored_map_pays_money,
    sound_manager, trial_win_update_lambda_0_0,
};

const SITE: &str = "trial_win_update_lambda_0";

pub fn trial_win_update_lambda_0(
    ctx: &mut AppContext,
    status: i32,
    record: Option<&RankingRecord>,
) -> Result<(), Fault> {
    connecting_indicator_hide(ctx)?;

    match status {
        1 => {
            let text = query_localizable(ctx, b"ranking_error01");

            dialog_show_alt(ctx, &text, 0, 0, 4, Some(trial_win_update_lambda_0_0))?;
        }
        0 => {
            if ranking_record_rank(record.ok_or(Fault::NullPointer { site: SITE })?) <= 0x63 {
                let sound = if scored_map_pays_money(ctx)? {
                    0xbe
                } else {
                    0x40
                };

                play_sound(sound_manager(ctx)?, sound, None);
            }

            request_save_data(ctx)?;
            ctx.set_i32_at(
                AppContext::OUTRO_PHASE,
                ctx.i32_at(AppContext::OUTRO_PHASE)?.wrapping_add(1),
            )?;
        }
        _ => {}
    }

    Ok(())
}
