use crate::{Fault, operation};

use super::{
    AppContext, UNIT_BUY, UnitBuy, query_localizable, string_format_int, string_format_rank_comment,
};

pub fn get_unit_file_base(ctx: &mut AppContext, unit_id: i32, form: i32) -> Result<Vec<u8>, Fault> {
    if form == -1 {
        return string_format_int(ctx, b"%03d_e", unit_id);
    }

    if unit_id as u32 <= 0x36b && form as u32 <= 1 {
        let row = (unit_id as u32 as i64) << 8;
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(&ctx.block_at::<4>(
            (row + (UNIT_BUY + UnitBuy::ALT_ART) as i64 + (form as u32 as i64) * 4) as usize,
        )?);
        pair[4..].copy_from_slice(
            &ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?,
        );

        let art = operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange {
            site: "get_unit_file_base",
            index: 0,
            limit: 1,
        })? as i32;

        if art != -1 {
            return string_format_int(ctx, b"%03d_m", art);
        }
    }

    let key = string_format_int(ctx, b"unit_evol_%d", form)?;
    let suffix = query_localizable(ctx, &key);

    string_format_rank_comment(ctx, b"%03d_%s", unit_id, &suffix)
}
