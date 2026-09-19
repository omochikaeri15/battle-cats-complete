use crate::Fault;

use super::{compute_event_points, find_point_rule_entry, get_point_rule_type, EventItemStore};

pub fn award_event_points(store: &mut EventItemStore, kind: i32, args: &[i32]) -> Result<i32, Fault> {
    const SITE: &str = "award_event_points";

    let table = store.rules.as_ref().ok_or(Fault::NullPointer { site: SITE })?;
    let entry = find_point_rule_entry(table, store.rule_id)?.ok_or(Fault::NullPointer { site: SITE })?;
    let mut points = 0i32;

    if get_point_rule_type(entry) == 0 {
        points = compute_event_points(store, kind, args)?;
    }

    let total = points.wrapping_add(store.total);

    store.total = if total < 0x3b9ac9ff { total } else { 0x3b9ac9ff };

    Ok(total)
}
