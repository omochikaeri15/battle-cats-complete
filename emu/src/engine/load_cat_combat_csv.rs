use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, CAT_STATS, CAT_STATS_FORM_STRIDE, CAT_STATS_UNIT_STRIDE, CatStats,
    get_column_count, read_cell_stream, read_csv_cell, read_csv_row,
};

const FORMS: usize = 4;
const COLUMNS: i32 = 0x76;

pub fn load_cat_combat_csv(
    ctx: &mut AppContext,
    stm: &mut AssetStream<'_>,
    id: i32,
) -> Result<(), Fault> {
    let unit = (id.wrapping_add(2) as i64 as usize).wrapping_mul(CAT_STATS_UNIT_STRIDE);
    let mut stats = CAT_STATS + unit;

    for form in 0..FORMS {
        if !read_csv_row(stm) {
            break;
        }

        let form_at = unit + form * CAT_STATS_FORM_STRIDE;
        ctx.zero(form_at + CAT_STATS, CAT_STATS_FORM_STRIDE)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::SPAWN_ANIMATION_TYPE, -1)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::TIME_BEFORE_DEATH, -1)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::ATTACK_COUNT_TOTAL, -1)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::ATTACK_1_ABILITIES, 1)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::CONJURE_UNIT_ID, -1)?;

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

            ctx.set_i32_at(
                stats + (column as usize) * 4,
                read_csv_cell(stm, column) as i32,
            )?;
            column += 1;
        }

        let speed = ctx.i32_at(form_at + CAT_STATS + CatStats::SPEED)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::SPEED, speed << 1)?;

        let pair = ctx.block_at::<8>(form_at + CAT_STATS + CatStats::ATTACK_COOLDOWN)?;
        let mut quad = [0u8; 16];

        quad[..8].copy_from_slice(&pair);

        let quad = ops::blend_epi16(ops::slli_epi32(quad, 1), ops::slli_epi32(quad, 2), 0xc);
        let mut pair = [0u8; 8];

        pair.copy_from_slice(&quad[..8]);
        ctx.set_block_at(
            form_at + CAT_STATS + CatStats::ATTACK_COOLDOWN,
            pair,
        )?;

        let cooldown = ctx.i32_at(form_at + CAT_STATS + CatStats::COOLDOWN)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::COOLDOWN, cooldown << 1)?;

        let width = ctx.i32_at(form_at + CAT_STATS + CatStats::HITBOX_WIDTH)?;
        ctx.set_i32_at(form_at + CAT_STATS + CatStats::HITBOX_WIDTH, width << 2)?;

        let pair = ctx.block_at::<8>(form_at + CAT_STATS + CatStats::LD1_ANCHOR)?;
        let mut long_distance = [0u8; 16];

        long_distance[..8].copy_from_slice(&pair);

        let long_distance = ops::slli_epi32(long_distance, 2);
        let mut pair = [0u8; 8];

        pair.copy_from_slice(&long_distance[..8]);
        ctx.set_block_at(
            form_at + CAT_STATS + CatStats::LD1_ANCHOR,
            pair,
        )?;

        stats += CAT_STATS_FORM_STRIDE;
    }

    let first = ctx.i32_at(unit + CAT_STATS + CatStats::EOC1_COST)?;
    ctx.set_i32_at(
        unit + CAT_STATS + CatStats::EOC1_COST,
        first.wrapping_mul(0x64),
    )?;

    let second = ctx.i32_at(unit + CAT_STATS + CAT_STATS_FORM_STRIDE + CatStats::EOC1_COST)?;
    ctx.set_i32_at(
        unit + CAT_STATS + CAT_STATS_FORM_STRIDE + CatStats::EOC1_COST,
        second.wrapping_mul(0x64),
    )?;

    let third = ctx.i32_at(unit + CAT_STATS + CAT_STATS_FORM_STRIDE * 2 + CatStats::EOC1_COST)?;
    ctx.set_i32_at(
        unit + CAT_STATS + CAT_STATS_FORM_STRIDE * 2 + CatStats::EOC1_COST,
        third.wrapping_mul(0x64),
    )?;

    let fourth = ctx.i32_at(unit + CAT_STATS + CAT_STATS_FORM_STRIDE * 3 + CatStats::EOC1_COST)?;
    ctx.set_i32_at(
        unit + CAT_STATS + CAT_STATS_FORM_STRIDE * 3 + CatStats::EOC1_COST,
        fourth.wrapping_mul(0x64),
    )?;

    Ok(())
}
