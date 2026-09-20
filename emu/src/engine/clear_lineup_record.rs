use crate::{Fault, ops};

use super::{AppContext, get_global_map_id, get_stage_index, get_star_level};

pub fn clear_lineup_record(
    ctx: &mut AppContext,
    map: i32,
    stage: i32,
    star: i32,
) -> Result<(), Fault> {
    let (map, stage, star) = if map == -1 {
        (
            get_global_map_id(ctx, 0)?,
            get_stage_index(ctx)?,
            get_star_level(ctx)?,
        )
    } else {
        (map, stage, star)
    };
    let key = star
        .wrapping_add(stage.wrapping_mul(10))
        .wrapping_add(map.wrapping_mul(0x3e8));

    if ctx.clear_lineups.entry(key).or_default().len() > 9 {
        return Ok(());
    }

    ctx.clear_lineups.entry(key).or_default().push(Vec::new());

    for slot in 0..10usize {
        let row = ctx.bytes_from(AppContext::CLEAR_LINEUP)?;
        let value = ops::xor_row_decode(row, 10, slot).ok_or(Fault::index_out_of_range(slot as i64, 10))?;

        if value == 0xffffffff {
            break;
        }

        let unit = value.wrapping_sub(2) as i32;
        let form = ctx.i32_at(((unit as i64) * 4 + AppContext::UNIT_FORMS as i64) as usize)?;

        if let Some(lineup) = ctx.clear_lineups.entry(key).or_default().last_mut() {
            lineup.push([unit, form]);
        }
    }

    Ok(())
}
