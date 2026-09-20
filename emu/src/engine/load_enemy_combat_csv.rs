use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, ENEMY_STATS, ENEMY_STATS_STRIDE, EnemyStats, get_column_count,
    read_csv_cell, read_csv_row,
};

const ROWS: usize = 0x324;
const COLUMNS: i32 = 0x71;

pub fn load_enemy_combat_csv(ctx: &mut AppContext, stm: &mut AssetStream<'_>) -> Result<(), Fault> {
    let mut stats = ENEMY_STATS;

    for row in 0..ROWS {
        read_csv_row(stm);

        let row_at = row * ENEMY_STATS_STRIDE;
        ctx.zero(stats, ENEMY_STATS_STRIDE)?;
        ctx.set_i32_at(row_at + ENEMY_STATS + EnemyStats::SPAWN_ANIMATION_TYPE, -1)?;
        ctx.set_i32_at(row_at + ENEMY_STATS + EnemyStats::ATTACK_COUNT_TOTAL, -1)?;
        ctx.set_i32_at(row_at + ENEMY_STATS + EnemyStats::ATTACK_1_ABILITIES, 1)?;

        let mut column = 0;

        while column != COLUMNS {
            if (column as u64) < get_column_count(stm) {
                ctx.set_i32_at(
                    stats + (column as usize) * 4,
                    read_csv_cell(stm, column) as i32,
                )?;
            }

            column += 1;
        }

        let speed = ctx.i32_at(row_at + ENEMY_STATS + EnemyStats::SPEED)?;
        ctx.set_i32_at(row_at + ENEMY_STATS + EnemyStats::SPEED, speed << 1)?;

        let pair = ctx.block_at::<8>(row_at + ENEMY_STATS + EnemyStats::ATTACK_COOLDOWN)?;
        let mut quad = [0u8; 16];

        quad[..8].copy_from_slice(&pair);

        let quad = ops::blend_epi16(ops::slli_epi32(quad, 1), ops::slli_epi32(quad, 2), 0xc);
        let mut pair = [0u8; 8];

        pair.copy_from_slice(&quad[..8]);
        ctx.set_block_at(
            row_at + ENEMY_STATS + EnemyStats::ATTACK_COOLDOWN,
            pair,
        )?;

        let width = ctx.i32_at(row_at + ENEMY_STATS + EnemyStats::HITBOX_WIDTH)?;
        ctx.set_i32_at(row_at + ENEMY_STATS + EnemyStats::HITBOX_WIDTH, width << 2)?;

        let pair = ctx.block_at::<8>(row_at + ENEMY_STATS + EnemyStats::LD1_ANCHOR)?;
        let mut long_distance = [0u8; 16];

        long_distance[..8].copy_from_slice(&pair);

        let long_distance = ops::slli_epi32(long_distance, 2);
        let mut pair = [0u8; 8];

        pair.copy_from_slice(&long_distance[..8]);
        ctx.set_block_at(
            row_at + ENEMY_STATS + EnemyStats::LD1_ANCHOR,
            pair,
        )?;

        stats += ENEMY_STATS_STRIDE;
    }

    let mut group = 0;

    while group != ROWS * ENEMY_STATS_STRIDE {
        let first = ctx.i32_at(group + ENEMY_STATS + EnemyStats::CASH_DROP)?;
        ctx.set_i32_at(
            group + ENEMY_STATS + EnemyStats::CASH_DROP,
            first.wrapping_mul(0x64),
        )?;

        let second =
            ctx.i32_at(group + ENEMY_STATS + ENEMY_STATS_STRIDE + EnemyStats::CASH_DROP)?;
        ctx.set_i32_at(
            group + ENEMY_STATS + ENEMY_STATS_STRIDE + EnemyStats::CASH_DROP,
            second.wrapping_mul(0x64),
        )?;

        let third =
            ctx.i32_at(group + ENEMY_STATS + ENEMY_STATS_STRIDE * 2 + EnemyStats::CASH_DROP)?;
        ctx.set_i32_at(
            group + ENEMY_STATS + ENEMY_STATS_STRIDE * 2 + EnemyStats::CASH_DROP,
            third.wrapping_mul(0x64),
        )?;

        let fourth =
            ctx.i32_at(group + ENEMY_STATS + ENEMY_STATS_STRIDE * 3 + EnemyStats::CASH_DROP)?;
        ctx.set_i32_at(
            group + ENEMY_STATS + ENEMY_STATS_STRIDE * 3 + EnemyStats::CASH_DROP,
            fourth.wrapping_mul(0x64),
        )?;

        group += ENEMY_STATS_STRIDE * 4;
    }

    Ok(())
}
