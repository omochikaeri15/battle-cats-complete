use super::{PointRule, PointRuleDetail};

pub fn get_point_rule(entry: &PointRule) -> Option<&PointRuleDetail> {
    entry.detail.as_ref()
}
