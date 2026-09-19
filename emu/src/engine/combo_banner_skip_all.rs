use crate::Fault;

use super::{combo_banner_pending, set_combo_banner_pending, AppContext};

pub fn combo_banner_skip_all(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<16>(AppContext::COMBO_BANNER_STATE, [0; 16])?;
    ctx.set_block_at::<16>(AppContext::COMBO_BANNER_UNITS, [0xff; 16])?;
    ctx.set_i32_at(AppContext::COMBO_BANNER_UNITS.wrapping_add(0x10), -1)?;

    ctx.combo_banner_texts[0] = None;
    ctx.combo_banner_texts[1] = None;
    ctx.combo_banner_texts[2] = None;

    for record in ctx.combo_store.records.iter_mut() {
        if combo_banner_pending(record) != 0 {
            set_combo_banner_pending(record, 0);
        }
    }

    Ok(())
}
