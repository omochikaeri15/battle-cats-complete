use crate::{Fault, operation};

use super::{AppContext, NyancomboRecord, UNIT_BUY, UnitBuy, get_tech_level, get_unit_level};

pub fn combo_unlocked(ctx: &AppContext, record: &NyancomboRecord) -> Result<bool, Fault> {
    if record.availability == -1 || record.state == 2 {
        return Ok(false);
    }

    for member in 0..5 {
        let unit = record.unit[member];

        if unit == -1 {
            return Ok(true);
        }

        let row = (unit as i64) << 8;

        match record.form[member] {
            0 => {
                let mut pair = [0u8; 8];

                pair[..4].copy_from_slice(&ctx.block_at::<4>(
                    AppContext::UNITS_OWNED.wrapping_add((unit as i64 as usize).wrapping_mul(4)),
                )?);
                pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::UNITS_OWNED_KEY)?);

                if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                    > 0
                {
                    continue;
                }

                return Ok(false);
            }
            1 => {
                if get_unit_level(ctx, unit)? > 9 {
                    continue;
                }

                return Ok(false);
            }
            2 => {
                let mut pair = [0u8; 8];

                pair[..4].copy_from_slice(&ctx.block_at::<4>(
                    (row + (UNIT_BUY + UnitBuy::TRUE_FORM_LEVEL) as i64) as usize,
                )?);
                pair[4..].copy_from_slice(
                    &ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?,
                );

                if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                    != -1
                {
                    let level = get_tech_level(
                        ctx,
                        AppContext::UNIT_LEVELS
                            .wrapping_add((unit as i64 as usize).wrapping_mul(8)),
                    )?
                    .wrapping_add(1);

                    pair[..4].copy_from_slice(&ctx.block_at::<4>(
                        (row + (UNIT_BUY + UnitBuy::TRUE_FORM_LEVEL) as i64) as usize,
                    )?);
                    pair[4..].copy_from_slice(
                        &ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?,
                    );

                    if level
                        >= operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                    {
                        continue;
                    }
                }

                pair[..4].copy_from_slice(&ctx.block_at::<4>(
                    (row + (UNIT_BUY + UnitBuy::TRUE_FORM_LEVEL) as i64) as usize,
                )?);
                pair[4..].copy_from_slice(
                    &ctx.block_at::<4>((row + (UNIT_BUY + UnitBuy::KEY) as i64) as usize)?,
                );

                if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                    == -1
                    && ctx.i32_at(
                        AppContext::REWARD_UNITS_OWNED
                            .wrapping_add((unit as i64 as usize).wrapping_mul(4)),
                    )? > 1
                {
                    continue;
                }

                return Ok(false);
            }
            3 => {
                if ctx.i32_at(
                    AppContext::REWARD_FORMS_OWNED
                        .wrapping_add((unit as i64 as usize).wrapping_mul(4)),
                )? >= 2
                {
                    continue;
                }

                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    Ok(true)
}
