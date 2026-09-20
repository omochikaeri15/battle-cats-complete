use crate::{Fault, operation};

use super::{
    EventItemStore, find_point_rule_entry, get_kill_point_base, get_point_rule, point_band_lookup,
};

pub fn compute_event_points(store: &EventItemStore, kind: i32, args: &[i32]) -> Result<i32, Fault> {
    let table = store
        .rules
        .as_ref()
        .ok_or(Fault::null_pointer())?;
    let Some(entry) = find_point_rule_entry(table, store.rule_id)? else {
        return Ok(0);
    };
    let Some(rule) = get_point_rule(entry) else {
        return Ok(0);
    };

    if kind == 1 {
        if args.len() != 2 {
            return Ok(0);
        }

        let first = args[0];
        let second = args[1];
        let cap = store.progress_cap;

        if cap == -1 {
            let base = get_kill_point_base(rule);

            return Ok(
                operation::div_100(base.wrapping_mul(second.wrapping_add(first)) as i64) as i32,
            );
        }

        let progress = store.progress;
        let base = get_kill_point_base(rule);
        let decay = (2.0 - progress as f64 / cap as f64) * second.wrapping_add(first) as f64;

        return Ok(operation::cvttsd2si(base as f64 * decay / 100.0));
    }

    if kind == 0 && args.len() == 1 {
        let value = args[0];
        let percent = point_band_lookup(rule, value)?;

        return Ok(operation::div_100(percent.wrapping_mul(value) as i64) as i32);
    }

    Ok(0)
}
