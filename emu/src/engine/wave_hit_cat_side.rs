use crate::Fault;

use super::{AppContext, Entity, WaveRecord, WaveSprite, cat_attack_dispatch, get_wave_block};

pub fn wave_hit_cat_side(
    ctx: &mut AppContext,
    wave_index: i32,
    target: i32,
    attack: i32,
) -> Result<(), Fault> {
    let record = (wave_index as i64).wrapping_mul(AppContext::WAVE_RECORD_STRIDE as i64) as usize;
    let owner = ctx.i32_at(
        AppContext::WAVE_RECORDS
            .wrapping_add(record)
            .wrapping_add(WaveRecord::OWNER_SLOT),
    )?;
    let mini = ctx.u8_at(
        AppContext::WAVE_RECORDS
            .wrapping_add(record)
            .wrapping_add(WaveRecord::MINI),
    )?;
    let proc_flags = ctx.bytes_from(
        AppContext::WAVE_RECORDS
            .wrapping_add(record)
            .wrapping_add(WaveRecord::PROC_FLAGS),
    )?;
    let proc_flags: [u8; 12] = proc_flags
        .get(..12)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(Fault::IndexOutOfRange {
            site: "wave_hit_cat_side",
            index: record as i64,
            limit: 12,
        })?;
    let metal_killer_pct = ctx.i32_at(
        AppContext::WAVE_RECORDS
            .wrapping_add(record)
            .wrapping_add(WaveRecord::METAL_KILLER_PCT),
    )?;

    if cat_attack_dispatch(
        ctx,
        1,
        owner,
        target,
        attack,
        mini as i32,
        proc_flags[0],
        proc_flags[8],
        proc_flags[1],
        proc_flags[2],
        proc_flags[3],
        proc_flags[4],
        proc_flags[5],
        proc_flags[6],
        proc_flags[7],
        proc_flags[10],
        metal_killer_pct,
        100,
    )? && get_wave_block(ctx, 1, target)?
    {
        ctx.set_i32_at(
            AppContext::WAVE_RECORDS
                .wrapping_add(record)
                .wrapping_add(WaveRecord::KIND),
            0,
        )?;
        ctx.set_i32_at(
            AppContext::WAVE_RECORDS
                .wrapping_add(record)
                .wrapping_add(WaveRecord::FRAME),
            0,
        )?;
        ctx.set_i32_at(
            AppContext::WAVE_RECORDS
                .wrapping_add(record)
                .wrapping_add(WaveRecord::LEVEL),
            0,
        )?;

        let mut sprite = 0usize;

        while sprite != 6 {
            ctx.set_i32_at(
                AppContext::WAVE_SPRITES
                    .wrapping_add(record)
                    .wrapping_add(sprite.wrapping_mul(WaveSprite::STRIDE))
                    .wrapping_add(WaveSprite::TIMER),
                0,
            )?;
            sprite += 1;
        }

        ctx.set_i32_at(
            AppContext::entity_field(1, target, Entity::WAVE_BLOCK_VFX_FRAME),
            0,
        )?;
        ctx.set_i32_at(
            AppContext::entity_field(1, target, Entity::WAVE_BLOCK_VFX_ACTIVE),
            1,
        )?;
    }

    Ok(())
}
