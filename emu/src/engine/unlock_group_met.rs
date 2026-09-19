use crate::{Fault, operation};

use super::{AppContext, abs_i32, get_stage_record, is_map_cleared, map_type_of_map_id};

const SITE: &str = "unlock_group_met";

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct UnlockGroup {
    pub conditions: Vec<i32>,
    pub required: i32,
    pub stage: i32,
    pub flag_id: i32,
}

pub fn unlock_group_met(ctx: &mut AppContext, id: i32) -> Result<bool, Fault> {
    if !ctx.unlock_groups.contains_key(&id) {
        return Ok(true);
    }

    let flag_id = ctx.unlock_groups.entry(id).or_default().flag_id;

    if flag_id != -1 {
        if !ctx.unlock_flags.contains_key(&flag_id) {
            return Ok(false);
        }

        let flag_id = ctx.unlock_groups.entry(id).or_default().flag_id;

        return Ok(*ctx.unlock_flags.entry(flag_id).or_insert(false));
    }

    if ctx
        .unlock_groups
        .entry(id)
        .or_default()
        .conditions
        .is_empty()
    {
        return Ok(true);
    }

    let mut groups: Vec<Vec<i32>> = Vec::new();
    let mut listed = 0usize;

    while listed < ctx.unlock_groups.entry(id).or_default().conditions.len() {
        let missing = Fault::IndexOutOfRange {
            site: SITE,
            index: listed as i64,
            limit: 0,
        };

        if *ctx
            .unlock_groups
            .entry(id)
            .or_default()
            .conditions
            .get(listed)
            .ok_or(missing.clone())?
            >= 0
        {
            groups.push(Vec::new());
        }

        let condition = abs_i32(
            *ctx.unlock_groups
                .entry(id)
                .or_default()
                .conditions
                .get(listed)
                .ok_or(missing.clone())?,
        );

        groups.last_mut().ok_or(missing)?.push(condition);
        listed += 1;
    }

    let mut met = 0i32;
    let mut group = 0usize;

    while group < groups.len() {
        let mut element = 0usize;
        let mut satisfied = false;

        while element
            < groups
                .get(group)
                .ok_or(Fault::IndexOutOfRange {
                    site: SITE,
                    index: group as i64,
                    limit: groups.len() as i64,
                })?
                .len()
        {
            let members = groups.get(group).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: group as i64,
                limit: groups.len() as i64,
            })?;
            let condition = *members.get(element).ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: element as i64,
                limit: members.len() as i64,
            })?;
            let map_id = condition.wrapping_sub(
                (operation::div_100000(condition as i64) as i32).wrapping_mul(0x186a0),
            );
            let map_type = map_type_of_map_id(map_id);
            let mut map_idx = map_id.wrapping_sub(0xbb8);

            if map_idx as u32 >= 3 {
                map_idx = map_id.wrapping_sub(0xbbb);

                if map_idx as u32 >= 3 {
                    map_idx = map_id.wrapping_sub(0xbbe);

                    if map_idx as u32 >= 3 {
                        map_idx =
                            map_id.wrapping_sub(operation::div_1000(map_id).wrapping_mul(0x3e8));
                    }
                }
            }

            if (condition.wrapping_sub(0x186a0) as u32) <= 0x1869f {
                if ctx.unlock_groups.entry(id).or_default().stage == -1
                    && is_map_cleared(ctx, map_type, map_idx, 0, 0)?
                {
                    satisfied = true;

                    break;
                }

                if ctx.unlock_groups.entry(id).or_default().stage != -1 {
                    let stage = ctx.unlock_groups.entry(id).or_default().stage;

                    if get_stage_record(ctx, map_type, map_idx, stage, 0, 0)? > 0 {
                        satisfied = true;

                        break;
                    }
                }
            }

            if (condition.wrapping_sub(0x30d40) as u32) <= 0x1869f
                && ctx.condition_list_200k.contains(&map_id)
            {
                satisfied = true;

                break;
            }

            element += 1;
        }

        if satisfied {
            met = met.wrapping_add(1);
        } else if ctx.unlock_groups.entry(id).or_default().required == 0 {
            return Ok(false);
        }

        group += 1;
    }

    if ctx.unlock_groups.entry(id).or_default().required <= 0 {
        return Ok(true);
    }

    Ok(met >= ctx.unlock_groups.entry(id).or_default().required)
}
