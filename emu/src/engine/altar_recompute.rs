use crate::Fault;

use super::{get_stage_record, map_index_of_map_id, map_type_of_map_id, AppContext};

const SITE: &str = "altar_recompute";

#[derive(Clone, Copy, Default)]
pub struct AltarReward {
    pub amount: i32,
    pub enemy: i32,
    pub unseal: u8,
}

pub fn altar_recompute(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.altar_level_caps.clear();
    ctx.altar_unsealed.clear();

    let enemies: Vec<i32> = ctx.altar_enemy_ids.keys().copied().collect();

    for enemy in enemies {
        *ctx.altar_level_caps.entry(enemy).or_insert(0) = 0;
        *ctx.altar_unsealed.entry(enemy).or_insert(false) = false;
    }

    let rewards: Vec<(i32, AltarReward)> = ctx.altar_rewards.iter().map(|(key, reward)| (*key, *reward)).collect();

    for (key, reward) in rewards {
        let map = key / 100;
        let stage = key.wrapping_sub(map.wrapping_mul(100));
        let map_type = map_type_of_map_id(map);
        let map_index = map_index_of_map_id(map);

        if get_stage_record(ctx, map_type, map_index, stage, 0, 0)? > 0 {
            let cap = ctx.altar_level_caps.get_mut(&reward.enemy).ok_or(Fault::KeyNotFound { site: SITE, key: reward.enemy as i64 })?;

            *cap = cap.wrapping_add(reward.amount);

            let unsealed = ctx.altar_unsealed.get_mut(&reward.enemy).ok_or(Fault::KeyNotFound { site: SITE, key: reward.enemy as i64 })?;

            *unsealed |= reward.unseal != 0;
        }
    }

    Ok(())
}
