use crate::Fault;

use super::{medal_awarded, AppContext};

pub fn enigma_medal_count(ctx: &AppContext) -> Result<i32, Fault> {
    let mut owned = 0i32;
    let mut index = 0usize;

    while index < ctx.enigma.medals.len() {
        let medal = *ctx.enigma.medals.get(index).ok_or(Fault::IndexOutOfRange { site: "enigma_medal_count", index: index as i64, limit: 0 })?;

        owned = owned.wrapping_add(medal_awarded(ctx, medal) as i32);
        index += 1;
    }

    Ok(owned)
}
