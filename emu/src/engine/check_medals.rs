use crate::Fault;

use super::{
    AppContext, log_analytics_event, medal_award_set, medal_condition_met, unlock_popup_is_unlocked,
};

pub fn check_medals(ctx: &mut AppContext, kind: i32) -> Result<(), Fault> {
    if !unlock_popup_is_unlocked(ctx, 0x4c) {
        return Ok(());
    }

    if ctx.medals.is_empty() {
        return Ok(());
    }

    let mut pass = 0usize;

    loop {
        let mut awarded_any = false;
        let mut index = 0i32;

        loop {
            if !ctx.medals_awarded.contains(&index) {
                let medal = ctx
                    .medals
                    .get(index as i64 as usize)
                    .cloned()
                    .ok_or(Fault::index_out_of_range(index as i64, ctx.medals.len() as i64))?;

                if medal_condition_met(ctx, &medal, kind)? {
                    medal_award_set(ctx, index, 1);
                    *ctx.medals_awarded_flags.entry(index).or_insert(0) = 1;
                    log_analytics_event(ctx, 0x47, index, 0, 0, 0)?;
                    ctx.medals_pending.push(index);
                    awarded_any = true;
                }
            }

            index = index.wrapping_add(1);

            if ctx.medals.len() as i64 <= index as i64 {
                break;
            }
        }

        if !awarded_any {
            break;
        }

        pass += 1;

        if ctx.medals.len() <= pass {
            break;
        }
    }

    Ok(())
}
