use std::collections::BTreeMap;

pub fn build_trait_mask(pairs: &[(i32, bool)]) -> BTreeMap<i32, bool> {
    let mut traits = BTreeMap::new();

    for (trait_bit, present) in pairs {
        traits.entry(*trait_bit).or_insert(*present);
    }

    traits
}
