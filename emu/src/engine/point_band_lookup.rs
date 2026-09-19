use crate::Fault;

use super::PointRuleDetail;

pub fn point_band_lookup(rule: &PointRuleDetail, value: i32) -> Result<i32, Fault> {
    let mut previous = 0i32;

    for (&ceiling, &percent) in rule.bands.iter() {
        let floor = previous;

        previous = ceiling;

        if floor < value && ceiling >= value {
            return Ok(percent);
        }
    }

    rule.bands.values().next_back().copied().ok_or(Fault::NullPointer { site: "point_band_lookup" })
}
