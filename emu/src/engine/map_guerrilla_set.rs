use super::MapOption;

pub fn map_guerrilla_set(store: &MapOption, map_id: i32) -> i32 {
    store.guerrilla_set.get(&map_id).map_or(0, |bits| *bits)
}
