use std::rc::Rc;

use crate::{Fault, operation};

use super::{
    AppContext, Imgcut, UNIT_BUY, UnitBuy, query_localizable, string_format_int,
    string_format_int_text_int, string_format_int2, texture_cache_load,
};

pub fn load_unit_icon(
    ctx: &mut AppContext,
    unit_id: i32,
    form: i32,
    variant: i32,
) -> Result<Option<Rc<Imgcut>>, Fault> {
    if unit_id as u32 <= 0x36b && form as u32 <= 1 {
        let row = (unit_id as u32 as i64) << 8;
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(&ctx.block_at::<4>(
            (row + (UNIT_BUY + UnitBuy::ALT_ART) as i64 + (form as u32 as i64) * 4) as usize,
        )?);
        pair[4..].copy_from_slice(
            &ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?,
        );

        let art = operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32;

        if art != -1 {
            let png = string_format_int2(ctx, b"uni%03d_m%02d.png", art, form)?;

            return texture_cache_load(ctx, &png, b"uni.imgcut", 0x2601);
        }
    }

    let key = string_format_int(ctx, b"unit_evol_%d", form)?;
    let suffix = query_localizable(ctx, &key);
    let png = string_format_int_text_int(ctx, b"uni%03d_%@%02d.png", unit_id, &suffix, variant)?;

    texture_cache_load(ctx, &png, b"uni.imgcut", 0x2601)
}
