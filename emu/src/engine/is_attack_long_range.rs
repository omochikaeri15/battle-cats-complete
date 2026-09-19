use crate::Fault;

use super::{ATTACK_LD_ANCHOR_FIELDS, ATTACK_LD_FLAG_FIELDS, AppContext, Entity};

pub fn is_attack_long_range(
    ctx: &AppContext,
    faction: i32,
    slot: i32,
    attack: i32,
) -> Result<bool, Fault> {
    let marker = 'resolved: {
        if attack > 0 {
            let flag_field =
                *ATTACK_LD_FLAG_FIELDS
                    .get(attack as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: "is_attack_long_range",
                        index: attack as i64,
                        limit: 3,
                    })?;
            let entity = AppContext::entity_field(faction, slot, 0);

            if ctx.i32_at(entity.wrapping_add((flag_field as usize).wrapping_mul(4)))? != 0 {
                let anchor_field = *ATTACK_LD_ANCHOR_FIELDS.get(attack as usize).ok_or(
                    Fault::IndexOutOfRange {
                        site: "is_attack_long_range",
                        index: attack as i64,
                        limit: 3,
                    },
                )?;
                let anchor_missing =
                    ctx.i32_at(entity.wrapping_add((anchor_field as usize).wrapping_mul(4)))? == 0;

                break 'resolved -(anchor_missing as i32) | attack;
            }
        }

        let anchor_missing = ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::ATTACK_1_LD_ANCHOR,
        ))? == 0;

        -(anchor_missing as i32)
    };

    Ok(marker != -1)
}
