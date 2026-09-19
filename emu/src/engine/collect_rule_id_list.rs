use super::SpecialRuleStore;

pub fn collect_rule_id_list(store: &mut SpecialRuleStore, map_id: i32) -> Vec<i32> {
    let mut ids = Vec::new();
    let Some(rules) = store.maps.get(&map_id) else {
        return ids;
    };
    let groups: Vec<i32> = rules
        .normal
        .keys()
        .chain(rules.fever.keys())
        .copied()
        .collect();

    for group in groups {
        if !store.rule_units.contains_key(&group) {
            continue;
        }

        let mut index = 0usize;

        while index < store.rule_units.entry(group).or_default().len() {
            let unit = store.rule_units.entry(group).or_default()[index];

            ids.push(unit);
            index += 1;
        }
    }

    ids
}
