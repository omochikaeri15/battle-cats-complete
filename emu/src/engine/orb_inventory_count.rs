use super::AppContext;

pub fn orb_inventory_count(ctx: &AppContext, orb: i32) -> i32 {
    ctx.orb_inventory.get(&orb).copied().unwrap_or(0)
}
