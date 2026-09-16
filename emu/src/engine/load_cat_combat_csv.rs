use crate::fault::Fault;
use crate::operation::{blend_epi16, slli_epi32};

use super::{get_column_count, read_cell_stream, read_csv_cell, read_csv_row, AppContext, AssetStream};

const TABLE: usize = 0x9e568;
const FORM_STRIDE: usize = 0x1d8;
const FORMS: usize = 4;
const COLUMNS: i32 = 0x76;

pub fn load_cat_combat_csv(ctx: &mut AppContext, stm: &mut AssetStream<'_>, id: i32) -> Result<(), Fault> {
    let unit = (id.wrapping_add(2) as i64 as usize).wrapping_mul(0x760);
    let mut stats = TABLE + unit;

    for form in 0..FORMS {
        if !read_csv_row(stm) {
            break;
        }

        let form_at = unit + form * FORM_STRIDE;
        ctx.zero(form_at + 0x9e568, FORM_STRIDE)?;
        ctx.set_i32_at(form_at + 0x9e670, -1)?;
        ctx.set_i32_at(form_at + 0x9e64c, -1)?;
        ctx.set_i32_at(form_at + 0x9e644, -1)?;
        ctx.set_i32_at(form_at + 0x9e664, 1)?;
        ctx.set_i32_at(form_at + 0x9e720, -1)?;

        let mut column = 0;

        while column != COLUMNS {
            if (column as u64) >= get_column_count(stm) {
                break;
            }

            let cell = read_cell_stream(stm, column);
            let reprinted = format!("{}", read_csv_cell(stm, column) as i32);

            if cell != reprinted.as_bytes() {
                break;
            }

            ctx.set_i32_at(stats + (column as usize) * 4, read_csv_cell(stm, column) as i32)?;
            column += 1;
        }

        let speed = ctx.i32_at(form_at + 0x9e570)?;
        ctx.set_i32_at(form_at + 0x9e570, speed << 1)?;

        let quad = ctx.block_at::<16>(form_at + 0x9e578)?;
        ctx.set_block_at(form_at + 0x9e578, blend_epi16(slli_epi32(quad, 1), slli_epi32(quad, 2), 0xc))?;

        let cooldown = ctx.i32_at(form_at + 0x9e584)?;
        ctx.set_i32_at(form_at + 0x9e584, cooldown << 1)?;

        let width = ctx.i32_at(form_at + 0x9e58c)?;
        ctx.set_i32_at(form_at + 0x9e58c, width << 2)?;

        let long_distance = ctx.block_at::<16>(form_at + 0x9e618)?;
        ctx.set_block_at(form_at + 0x9e618, slli_epi32(long_distance, 2))?;

        stats += FORM_STRIDE;
    }

    let first = ctx.i32_at(unit + 0x9e580)?;
    ctx.set_i32_at(unit + 0x9e580, first.wrapping_mul(0x64))?;

    let second = ctx.i32_at(unit + 0x9e758)?;
    ctx.set_i32_at(unit + 0x9e758, second.wrapping_mul(0x64))?;

    let third = ctx.i32_at(unit + 0x9e930)?;
    ctx.set_i32_at(unit + 0x9e930, third.wrapping_mul(0x64))?;

    let fourth = ctx.i32_at(unit + 0x9eb08)?;
    ctx.set_i32_at(unit + 0x9eb08, fourth.wrapping_mul(0x64))?;

    Ok(())
}
