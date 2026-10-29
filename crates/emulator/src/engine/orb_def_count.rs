use super::OrbStore;

pub fn orb_def_count(store: &OrbStore) -> i64 {
    store.orbs.len() as i64
}
