use super::AppContext;

pub fn hidden_drop_key(ctx: &mut AppContext, id: i32) -> Vec<u8> {
    ctx.hidden_drop_keys.entry(id).or_default().clone()
}
