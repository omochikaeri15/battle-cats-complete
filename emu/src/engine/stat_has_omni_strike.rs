use crate::Fault;

use super::{read_flag, AppContext};

pub fn stat_has_omni_strike(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    let enemy_row = (unit_id.wrapping_add(2) as i64) * 0x1c4;
    let cat_row = (unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8;

    let ld1_anchor = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        cat_row + 0x9e618
    } else {
        enemy_row + 0x233194
    };

    if ctx.i32_at(ld1_anchor as usize)? != 0 {
        let ld1_span = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
            cat_row + 0x9e61c
        } else {
            enemy_row + 0x233198
        };

        if ctx.i32_at(ld1_span as usize)? < 0 {
            return Ok(true);
        }
    }

    let ld2_anchor = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        cat_row + 0x9e6f8
    } else {
        enemy_row + 0x233288
    };

    if ctx.i32_at(ld2_anchor as usize)? & 0x3fffffff != 0 {
        let ld2_span = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
            cat_row + 0x9e6fc
        } else {
            enemy_row + 0x23328c
        };

        if ctx.i32_at(ld2_span as usize)? & 0x20000000 != 0 {
            return Ok(true);
        }
    }

    let ld3_anchor = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        cat_row + 0x9e704
    } else {
        enemy_row + 0x233294
    };

    if ctx.i32_at(ld3_anchor as usize)? & 0x3fffffff != 0 {
        let ld3_span = if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
            cat_row + 0x9e708
        } else {
            enemy_row + 0x233298
        };

        if ctx.i32_at(ld3_span as usize)? & 0x20000000 != 0 {
            return Ok(true);
        }
    }

    Ok(false)
}
