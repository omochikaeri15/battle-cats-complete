use super::{AppContext, orb_inventory_count, orb_inventory_set};

pub fn orb_inventory_add(ctx: &mut AppContext, orb: i32, count: i32) {
    let total = orb_inventory_count(ctx, orb).wrapping_add(count);

    orb_inventory_set(ctx, orb, total);
}
