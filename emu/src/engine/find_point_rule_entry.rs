use crate::Fault;

use super::{PointRule, PointRuleTable};

pub fn find_point_rule_entry(table: &PointRuleTable, rule_id: i32) -> Result<Option<&PointRule>, Fault> {
    if table.entries.is_empty() {
        return Ok(None);
    }

    if table.entries.contains_key(&-1) {
        return table.entries.get(&-1).map(Some).ok_or(Fault::KeyNotFound { site: "find_point_rule_entry", key: -1 });
    }

    if !table.entries.contains_key(&rule_id) {
        return Ok(None);
    }

    table.entries.get(&rule_id).map(Some).ok_or(Fault::KeyNotFound { site: "find_point_rule_entry", key: rule_id as i64 })
}
