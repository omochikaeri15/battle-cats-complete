use super::OrbStore;

pub fn orb_ability_repeat(store: &mut OrbStore, abil: i32) -> bool {
    if store.ability_flags.is_empty() {
        return true;
    }

    store.ability_flags.entry(abil).or_insert([0, 1])[1] != 0
}
