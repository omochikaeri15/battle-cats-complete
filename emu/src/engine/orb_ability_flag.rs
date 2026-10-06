use super::OrbStore;

pub fn orb_ability_flag(store: &mut OrbStore, ability: i32) -> bool {
    if store.ability_flags.is_empty() {
        return false;
    }

    if !store.ability_flags.contains_key(&ability) {
        return false;
    }

    store.ability_flags.entry(ability).or_insert([0, 1])[0] != 0
}
