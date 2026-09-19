use crate::Fault;

use super::{
    AppContext, FormatArg, add_resource, analytics_record, app_on_draw, base_shake_reset,
    get_global_map_id, get_stage_index, get_star_level, is_ex_option_target, log_analytics_event,
    save_battle_snapshot,
};

pub fn battle_continue(ctx: &mut AppContext) -> Result<(), Fault> {
    base_shake_reset(&mut ctx.base_shake);
    app_on_draw(ctx)?;

    if ctx.u8_at(AppContext::RESULT_VIDEO_WATCHED)? != 0 {
        ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 1)?;
        ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;
    } else {
        add_resource(ctx, 0x16, -0x1e, 0)?;
        ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
        ctx.set_i32_at(AppContext::CURTAIN_STYLE, 1)?;
        ctx.set_i32_at(AppContext::REVIVE_REQUESTED, 1)?;
        ctx.set_i32_at(AppContext::REWARD_POP_COUNTER, 0)?;

        if ctx.u8_at(AppContext::RESULT_VIDEO_WATCHED)? == 0 {
            let map = get_global_map_id(ctx, 0)?;
            let stage = get_stage_index(ctx)?;
            let star = get_star_level(ctx)?;

            analytics_record(
                ctx,
                0x13157fc,
                0x1e,
                0,
                &[
                    (b"sec1_type", FormatArg::Text(b"MapID")),
                    (b"sec1_id", FormatArg::Int(map)),
                    (b"sec2_type", FormatArg::Text(b"StageIdx")),
                    (b"sec2_id", FormatArg::Int(stage)),
                    (b"ex_type", FormatArg::Text(b"StageLv")),
                    (b"ex_id", FormatArg::Int(star)),
                ],
            )?;
        }
    }

    log_analytics_event(ctx, 0x14, 0, 0, 0, 0)?;

    if ctx.i32_at(AppContext::CHAPTER_MODE)? != 0x63 || is_ex_option_target(ctx)? {
        ctx.set_i32_at(AppContext::BATTLE_RESUMED, 1)?;
        ctx.set_i32_at(AppContext::BATTLE_CONTINUED, 1)?;
        save_battle_snapshot(ctx)?;
    }

    Ok(())
}
