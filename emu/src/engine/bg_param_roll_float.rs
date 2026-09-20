use std::collections::{BTreeMap, btree_map::Entry};

use crate::{Fault, ops};

use super::{AppContext, BgParamSpec, bg_param_resolve_float, call_rng, max_f32, min_f32};

pub fn bg_param_roll_float(
    ctx: &mut AppContext,
    spec: &BgParamSpec<f32>,
    groups: &mut BTreeMap<i32, i32>,
    reference: i32,
    fallback: f32,
) -> Result<f32, Fault> {
    if spec.enabled == 0 {
        return Ok(fallback);
    }

    if spec.has_min == 0 || spec.has_max == 0 {
        let value = if spec.values.is_empty() {
            spec.value
        } else {
            let pick = call_rng(ctx, (spec.values.len() as u64) as i32);

            *spec
                .values
                .get(pick as i64 as usize)
                .ok_or(Fault::index_out_of_range(pick as i64, spec.values.len() as i64))?
        };

        return bg_param_resolve_float(ctx, reference, value, spec.base);
    }

    if spec.rand_group == -1 {
        let lower = bg_param_resolve_float(ctx, reference, spec.min, spec.min_base)?;
        let upper = bg_param_resolve_float(ctx, reference, spec.max, spec.max_base)?;
        let low = min_f32(lower, upper);
        let lower = bg_param_resolve_float(ctx, reference, spec.min, spec.min_base)?;
        let upper = bg_param_resolve_float(ctx, reference, spec.max, spec.max_base)?;
        let high = max_f32(lower, upper);
        let draw = call_rng(ctx, ops::cvttss2si((high - low) * 10000.0 + 1.0));

        return Ok((low * 10000.0 + draw as f32) / 10000.0);
    }

    if let Entry::Vacant(slot) = groups.entry(spec.rand_group) {
        let draw = call_rng(ctx, 0x2711);

        slot.insert(draw);
    }

    let low = bg_param_resolve_float(ctx, reference, spec.min, spec.min_base)?;
    let high = bg_param_resolve_float(ctx, reference, spec.max, spec.max_base)?;
    let span = high - bg_param_resolve_float(ctx, reference, spec.min, spec.min_base)?;
    let draw = *groups.get(&spec.rand_group).ok_or(Fault::key_not_found(spec.rand_group as i64))?;

    Ok(low + span * draw as f32 / 10000.0)
}
