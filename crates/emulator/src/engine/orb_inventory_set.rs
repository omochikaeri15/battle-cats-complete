use super::AppContext;

pub fn orb_inventory_set(ctx: &mut AppContext, orb: i32, count: i32) {
    *ctx.orb_inventory.entry(orb).or_insert(0) = if count > 0 { count } else { 0 };

    let stored = ctx.orb_inventory.entry(orb).or_insert(0);

    if *stored >= 0x2710 {
        *stored = 0x270f;
    }
}
