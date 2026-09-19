use super::OrbStore;

pub fn orb_ability_flag(store: &mut OrbStore, abil: i32) -> bool {
    if store.ability_flags.is_empty() {
        return false;
    }

    if !store.ability_flags.contains_key(&abil) {
        return false;
    }

    store.ability_flags.entry(abil).or_insert([0, 1])[0] != 0
}
