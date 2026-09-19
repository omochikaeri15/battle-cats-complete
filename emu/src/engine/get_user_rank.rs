use crate::{Fault, operation};

use super::{AppContext, level_cell_base, level_cell_plus};

pub fn get_user_rank(ctx: &AppContext) -> Result<i32, Fault> {
    let mut rank = 0i32;
    let mut unit = 0usize;

    while unit != 0x36c {
        let mut pair = [0u8; 8];

        pair[..4].copy_from_slice(
            &ctx.block_at::<4>(AppContext::UNITS_OWNED.wrapping_add(unit.wrapping_mul(4)))?,
        );
        pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::UNITS_OWNED_KEY)?);

        if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::IndexOutOfRange {
            site: "get_user_rank",
            index: 0,
            limit: 1,
        })? as i32
            > 0
        {
            let cell = AppContext::UNIT_LEVELS.wrapping_add(unit.wrapping_mul(8));
            let plus = level_cell_plus(ctx, cell)?;

            rank = rank
                .wrapping_add(plus)
                .wrapping_add(level_cell_base(ctx, cell)?)
                .wrapping_add(1);
        }

        unit += 1;
    }

    for tech in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
        let cell = AppContext::TECH_LEVELS.wrapping_add(tech.wrapping_mul(8));
        let plus = level_cell_plus(ctx, cell)?;

        rank = rank
            .wrapping_add(plus)
            .wrapping_add(level_cell_base(ctx, cell)?);
    }

    Ok(rank.wrapping_add(0xa))
}
