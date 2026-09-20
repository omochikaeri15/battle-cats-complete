use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MapOption {
    pub stars_unlocked: BTreeMap<i32, i32>,
    pub back_stars_unlocked: BTreeMap<i32, i32>,
    pub star_multipliers: BTreeMap<i32, Vec<i32>>,
    pub guerrilla_maps: BTreeMap<i32, Vec<i32>>,
    pub guerrilla_set: BTreeMap<i32, i32>,
    pub reward_reset_kind: BTreeMap<i32, i32>,
    pub show_once: BTreeMap<i32, i32>,
    pub display_order: BTreeMap<i32, i32>,
    pub interval: BTreeMap<i32, i32>,
    pub challenge_maps: Vec<i32>,
    pub difficulty: BTreeMap<i32, i32>,
    pub hide_after_clear_maps: Vec<i32>,
    pub double_xp_ad_maps: Vec<i32>,
    pub cost_multipliers: BTreeMap<i32, i32>,
    pub no_energy_maps: Vec<i32>,
    pub conditioned_maps: Vec<i32>,
}

pub fn get_map_max_star(store: &MapOption, map_id: i32) -> i32 {
    store
        .stars_unlocked
        .get(&map_id)
        .map_or(0, |unlocked| unlocked.wrapping_sub(1))
}
