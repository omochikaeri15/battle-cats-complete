use std::collections::BTreeMap;

#[derive(Clone, Default)]
pub struct RewardDef {
    pub id: i32,
    pub threshold: i32,
    pub kind: i32,
    pub target: i32,
    pub amount: i32,
    pub message: Vec<u8>,
}

pub fn reward_def_lookup(
    defs: &BTreeMap<i32, Vec<RewardDef>>,
    group: i32,
    id: i32,
) -> Option<&RewardDef> {
    defs.get(&group)?;

    let mut index = 0usize;

    loop {
        let list = defs.get(&group)?;

        if list.len() <= index {
            return None;
        }

        if list.get(index)?.id == id {
            return list.get(index);
        }

        index += 1;
    }
}
