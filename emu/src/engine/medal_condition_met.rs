use crate::Fault;

use super::{
    AppContext, Medal, abs_i32, get_medal_progress, medal_stage_condition_met, reward_owned,
};

pub fn medal_condition_met(ctx: &mut AppContext, medal: &Medal, kind: i32) -> Result<bool, Fault> {
    let mut allowed = vec![0i32];

    if (medal.kind as u32) > 3 {
        return Ok(false);
    }

    match medal.kind {
        0 => allowed.push(3),
        1 => match medal.action {
            0 => allowed.push(3),
            1 => allowed.push(1),
            2 => allowed.push(2),
            4 => allowed.push(1),
            _ => {}
        },
        2 => allowed.push(4),
        _ => allowed.push(3),
    }

    if !allowed.contains(&kind) {
        return Ok(false);
    }

    match medal.kind {
        0 => {
            let mut groups: Vec<Vec<i32>> = Vec::new();
            let mut at = 0usize;

            while at < medal.maps.len() {
                let id = *medal.maps.get(at).ok_or(Fault::index_out_of_range(at as i64, medal.maps.len() as i64))?;

                if id >= 0 {
                    groups.push(Vec::new());
                }

                let value = abs_i32(id);

                groups.last_mut().ok_or(Fault::index_out_of_range(-1, 0))?.push(value);
                at += 1;
            }

            let mut group = 0usize;

            while group < groups.len() {
                let ids = groups.get(group).ok_or(Fault::index_out_of_range(group as i64, groups.len() as i64))?.clone();

                if ids.is_empty() {
                    return Ok(false);
                }

                let mut member = 0usize;
                let mut any = false;

                while member < ids.len() {
                    if medal_stage_condition_met(ctx, medal, ids[member])? {
                        any = true;
                        break;
                    }

                    member += 1;
                }

                if !any {
                    return Ok(false);
                }

                group += 1;
            }

            Ok(true)
        }
        1 => Ok(get_medal_progress(ctx, medal.action)? >= medal.count),
        2 => reward_owned(ctx, medal.chara),
        _ => {
            let mut at = 0usize;

            while at < medal.medals.len() {
                if !ctx.medals_awarded.contains(&medal.medals[at]) {
                    return Ok(false);
                }

                at += 1;
            }

            Ok(true)
        }
    }
}
