use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, talent_targets_trait};

const TRAIT_BITS: [(i32, i32); 10] = [
    (0, 0x1),
    (3, 0x8),
    (1, 0x2),
    (2, 0x4),
    (4, 0x10),
    (5, 0x20),
    (6, 0x40),
    (7, 0x80),
    (8, 0x800),
    (9, 0x100),
];

pub fn get_talent_trait_set(
    ctx: &mut AppContext,
    unit_id: i32,
    form: i32,
) -> Result<BTreeMap<i32, bool>, Fault> {
    let mut traits: BTreeMap<i32, bool> = BTreeMap::new();

    if (unit_id.wrapping_add(2) as u32) > 0x36d {
        return Ok(traits);
    }

    if (form as u32) >= 4 {
        return Ok(traits);
    }

    for (key, bit) in TRAIT_BITS {
        if talent_targets_trait(ctx, 0, unit_id, form, bit)? {
            traits.insert(key, true);
        }
    }

    if !traits.values().any(|shown| *shown) {
        traits.insert(0x10, true);
    }

    Ok(traits)
}
