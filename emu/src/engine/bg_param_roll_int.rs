use std::collections::{btree_map::Entry, BTreeMap};

use crate::{operation, Fault};

use super::{bg_param_resolve_int, call_rng, max_i32, min_i32, AppContext, BgParamSpec};

const SITE: &str = "bg_param_roll_int";

pub fn bg_param_roll_int(ctx: &mut AppContext, spec: &BgParamSpec<i32>, groups: &mut BTreeMap<i32, i32>, reference: i32, fallback: i32) -> Result<i32, Fault> {
    if spec.enabled == 0 {
        return Ok(fallback);
    }

    if spec.has_min == 0 || spec.has_max == 0 {
        let value = if spec.values.is_empty() {
            spec.value
        } else {
            let pick = call_rng(ctx, (spec.values.len() as u64) as i32);

            *spec.values.get(pick as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: pick as i64, limit: spec.values.len() as i64 })?
        };

        return bg_param_resolve_int(ctx, reference, value, spec.base);
    }

    if spec.rand_group == -1 {
        let lower = bg_param_resolve_int(ctx, reference, spec.min, spec.min_base)?;
        let upper = bg_param_resolve_int(ctx, reference, spec.max, spec.max_base)?;
        let low = min_i32(lower, upper);
        let lower = bg_param_resolve_int(ctx, reference, spec.min, spec.min_base)?;
        let upper = bg_param_resolve_int(ctx, reference, spec.max, spec.max_base)?;
        let high = max_i32(lower, upper);

        return Ok(call_rng(ctx, high.wrapping_sub(low).wrapping_add(1)).wrapping_add(low));
    }

    if let Entry::Vacant(slot) = groups.entry(spec.rand_group) {
        let draw = call_rng(ctx, 0x2711);

        slot.insert(draw);
    }

    let low = bg_param_resolve_int(ctx, reference, spec.min, spec.min_base)?;
    let high = bg_param_resolve_int(ctx, reference, spec.max, spec.max_base)?;
    let span = high.wrapping_sub(bg_param_resolve_int(ctx, reference, spec.min, spec.min_base)?);
    let draw = *groups.get(&spec.rand_group).ok_or(Fault::KeyNotFound { site: SITE, key: spec.rand_group as i64 })?;

    Ok(operation::div_10000(span.wrapping_mul(draw)).wrapping_add(low))
}
