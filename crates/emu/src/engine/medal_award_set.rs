use super::AppContext;

pub fn medal_award_set(ctx: &mut AppContext, id: i32, awarded: u8) {
    let present = ctx.medals_awarded.contains(&id);

    if present as u8 != awarded {
        if awarded == 0 {
            if let Some(at) = ctx.medals_awarded.iter().position(|held| *held == id) {
                ctx.medals_awarded.remove(at);
            }
        } else {
            ctx.medals_awarded.push(id);
        }
    }
}
