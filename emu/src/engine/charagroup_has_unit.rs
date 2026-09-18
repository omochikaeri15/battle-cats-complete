use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CharaGroup {
    pub units: Vec<i32>,
    pub kind: i32,
}

pub fn charagroup_has_unit(store: &BTreeMap<i32, CharaGroup>, group_id: i32, unit_id: i32) -> bool {
    let Some(group) = store.get(&group_id) else {
        return false;
    };

    if group.kind != 3 {
        return false;
    }

    group.units.contains(&unit_id)
}
