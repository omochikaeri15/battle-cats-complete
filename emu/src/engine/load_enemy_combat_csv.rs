use crate::fault::Fault;
use crate::operation::{blend_epi16, slli_epi32};

use super::{get_column_count, read_csv_cell, read_csv_row, AppContext, AssetStream};

const TABLE: usize = 0x233108;
const STRIDE: usize = 0x1c4;
const ROWS: usize = 0x324;
const COLUMNS: i32 = 0x71;

pub fn load_enemy_combat_csv(ctx: &mut AppContext, stm: &mut AssetStream<'_>) -> Result<(), Fault> {
    let mut stats = TABLE;

    for row in 0..ROWS {
        read_csv_row(stm);

        let row_at = row * STRIDE;
        ctx.zero(stats, STRIDE)?;
        ctx.set_i32_at(row_at + 0x2331dc, -1)?;
        ctx.set_i32_at(row_at + 0x2331d0, -1)?;
        ctx.set_i32_at(row_at + 0x2331f4, 1)?;

        let mut column = 0;

        while column != COLUMNS {
            if (column as u64) < get_column_count(stm) {
                ctx.set_i32_at(stats + (column as usize) * 4, read_csv_cell(stm, column) as i32)?;
            }

            column += 1;
        }

        let speed = ctx.i32_at(row_at + 0x233110)?;
        ctx.set_i32_at(row_at + 0x233110, speed << 1)?;

        let quad = ctx.block_at::<16>(row_at + 0x233118)?;
        ctx.set_block_at(row_at + 0x233118, blend_epi16(slli_epi32(quad, 1), slli_epi32(quad, 2), 0xc))?;

        let width = ctx.i32_at(row_at + 0x233128)?;
        ctx.set_i32_at(row_at + 0x233128, width << 2)?;

        let long_distance = ctx.block_at::<16>(row_at + 0x233194)?;
        ctx.set_block_at(row_at + 0x233194, slli_epi32(long_distance, 2))?;

        stats += STRIDE;
    }

    let mut group = 0;

    while group != 0x58b90 {
        let first = ctx.i32_at(group + 0x233120)?;
        ctx.set_i32_at(group + 0x233120, first.wrapping_mul(0x64))?;

        let second = ctx.i32_at(group + 0x2332e4)?;
        ctx.set_i32_at(group + 0x2332e4, second.wrapping_mul(0x64))?;

        let third = ctx.i32_at(group + 0x2334a8)?;
        ctx.set_i32_at(group + 0x2334a8, third.wrapping_mul(0x64))?;

        let fourth = ctx.i32_at(group + 0x23366c)?;
        ctx.set_i32_at(group + 0x23366c, fourth.wrapping_mul(0x64))?;

        group += 0x710;
    }

    Ok(())
}
