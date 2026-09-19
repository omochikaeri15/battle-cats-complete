use crate::Fault;

use super::AppContext;

const SITE: &str = "lose_tip_targets_stage";

pub fn lose_tip_targets_stage(ctx: &AppContext, id: i32, map: i32, stage: i32) -> Result<bool, Fault> {
    if ctx.lose_text_settings.len() as i32 <= 0 {
        return Ok(false);
    }

    let key = map.wrapping_mul(100).wrapping_add(stage);
    let mut row = 0usize;

    loop {
        let first = *ctx.lose_text_settings[row].first().ok_or(Fault::NullPointer { site: SITE })?;

        if first == id {
            break;
        }

        row += 1;

        if row as i64 >= ctx.lose_text_settings.len() as i32 as i64 {
            return Ok(false);
        }
    }

    let cells = &ctx.lose_text_settings[row];
    let mode = *cells.get(0x1b).ok_or(Fault::IndexOutOfRange { site: SITE, index: 0x1b, limit: cells.len() as i64 })?;

    let target = match mode {
        1 => key,
        2 => map,
        _ => return Ok(false),
    };

    if (cells.len() as i32) < 0x1d {
        return Ok(false);
    }

    let mut column = 0x1cusize;

    loop {
        if cells[column] == -1 {
            return Ok(false);
        }

        if cells[column] == target {
            return Ok(true);
        }

        column += 1;

        if column as i64 >= cells.len() as i32 as i64 {
            return Ok(false);
        }
    }
}
