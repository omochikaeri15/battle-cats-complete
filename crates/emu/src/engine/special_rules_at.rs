use std::collections::BTreeMap;

use crate::Fault;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapRules {
    pub normal: BTreeMap<i32, Vec<i32>>,
    pub fever: BTreeMap<i32, Vec<i32>>,
    pub rule_name_label: Vec<u8>,
    pub rule_explanation_label: Vec<u8>,
    pub contents_type: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct ResetPopup {
    pub map_id: i32,
    pub popup_label: Vec<u8>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct SpecialRuleStore {
    pub maps: BTreeMap<i32, MapRules>,
    pub invalid_nyancombo_ids: BTreeMap<i32, Vec<i32>>,
    pub reset_popups: BTreeMap<i32, ResetPopup>,
    pub item_f7_count: i32,
    pub point_baseline: i32,
    pub fever_count: i32,
    pub fever_frame: i32,
    pub fade_phase: i32,
    pub fade_reverse: u8,
    pub gauge_fill: i32,
}

pub fn special_rules_at<'a>(
    store: &'a SpecialRuleStore,
    map_id: &i32,
) -> Result<&'a MapRules, Fault> {
    store.maps.get(map_id).ok_or(Fault::key_not_found(*map_id as i64))
}
