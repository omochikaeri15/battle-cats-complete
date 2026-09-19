use crate::Fault;

use super::{
    AppContext, AssetStream, load_enemy_castle_csv, open_asset_stream, read_csv_cell, read_csv_row,
    string_format_int,
};

pub fn load_ex_map_stage_csv(ctx: &mut AppContext, map: i32) -> Result<bool, Fault> {
    let loaded = ctx.u8_at(AppContext::ALL_MAPS_OPEN)? != 0 || map < 0x53;

    if !loaded {
        return Ok(false);
    }

    let name = string_format_int(ctx, b"MapStageDataRE_%03d.csv", map)?;
    let bytes = open_asset_stream(ctx, &name, 0, 0)?.unwrap_or_default();
    let mut stm = AssetStream::new(&bytes, b'\n');

    ctx.set_i32_at(AppContext::EVENT_REWARD_ID, -1)?;
    ctx.set_i32_at(AppContext::RANKING_ID, -1)?;
    read_csv_row(&mut stm);

    let reward = read_csv_cell(&stm, 0) as i32;

    ctx.set_i32_at(
        AppContext::EVENT_REWARD_ID,
        if reward < 0x190 { reward } else { -1 },
    )?;
    read_csv_row(&mut stm);

    let count = read_csv_cell(&stm, 0) as i32;

    for row in 0..0x64usize {
        let row_at = AppContext::MAP_STAGE_ROWS + row * 0xbc;

        for col in 0..0x2eusize {
            let key = ctx.i32_at(row_at + 0xb8)?;

            ctx.set_i32_at(row_at + col * 4, !key)?;
        }
    }

    if count > 0 {
        for row in 0..count as u32 as usize {
            read_csv_row(&mut stm);

            let row_at = AppContext::MAP_STAGE_ROWS.wrapping_add(row.wrapping_mul(0xbc));

            for col in 0..0x2eusize {
                let value = read_csv_cell(&stm, col as i32) as i32;
                let key = ctx.i32_at(row_at + 0xb8)?;

                ctx.set_i32_at(row_at + col * 4, value ^ key)?;

                if ctx.i32_at(row_at + col * 4)? ^ ctx.i32_at(row_at + 0xb8)? == -1 {
                    break;
                }
            }
        }
    }

    load_enemy_castle_csv(ctx, 2)?;

    Ok(true)
}
