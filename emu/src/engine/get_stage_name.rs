use crate::Fault;

use super::{AppContext, STAGE_DISPLAY_ORDER};

pub fn get_stage_name(ctx: &AppContext, map_type: i32, map_index: i32, stage: i32) -> Result<Vec<u8>, Fault> {
    let case = map_type.wrapping_add(0x1a) as u32;

    if case > 0x1e {
        return Ok(Vec::new());
    }

    let map = map_index as i64 as usize;
    let slot = stage as i64 as usize;

    let names = match case {
        0x00 => &ctx.stage_table_neg26.names,
        0x01 | 0x0b | 0x15 | 0x19 => return Ok(Vec::new()),
        0x02 => &ctx.stage_table_neg24.names,
        0x03 => &ctx.stage_table_neg23.names,
        0x04 => &ctx.stage_table_neg22.names,
        0x05 => &ctx.stage_name_variants[16],
        0x06 => &ctx.stage_name_variants[15],
        0x07 => {
            let rows = &ctx.stage_name_variants[14];
            let row = rows.get(map).ok_or(Fault::index_out_of_range(map as i64, rows.len() as i64))?;

            return row.get(slot).cloned().ok_or(Fault::index_out_of_range(slot as i64, row.len() as i64));
        }
        0x08 => &ctx.stage_name_variants[13],
        0x09 => &ctx.stage_name_variants[12],
        0x0a => &ctx.stage_name_variants[11],
        0x0c | 0x13 | 0x0d | 0x17 | 0x0e | 0x18 => {
            let numbered = match case {
                0x0c | 0x13 => &ctx.stage_names_numbered[2],
                0x0d | 0x17 => &ctx.stage_names_numbered[1],
                _ => &ctx.stage_names_numbered[0],
            };
            let order = *STAGE_DISPLAY_ORDER
                .get(slot)
                .ok_or(Fault::index_out_of_range(slot as i64, STAGE_DISPLAY_ORDER.len() as i64))?;

            return numbered
                .get(order as i64 as usize)
                .cloned()
                .ok_or(Fault::index_out_of_range(order as i64, numbered.len() as i64));
        }
        0x0f => &ctx.stage_name_variants[10],
        0x10 => &ctx.stage_name_variants[9],
        0x11 => &ctx.stage_name_variants[8],
        0x12 => &ctx.stage_name_variants[4],
        0x14 => &ctx.stage_name_variants[7],
        0x16 => &ctx.stage_name_variants[3],
        0x1a => &ctx.stage_name_variants[0],
        0x1b => &ctx.stage_name_variants[1],
        0x1c => &ctx.stage_name_variants[2],
        0x1d => &ctx.stage_name_variants[5],
        _ => &ctx.stage_name_variants[6],
    };
    let row = names.get(map).ok_or(Fault::index_out_of_range(map as i64, names.len() as i64))?;

    row.get(slot).cloned().ok_or(Fault::index_out_of_range(slot as i64, row.len() as i64))
}
