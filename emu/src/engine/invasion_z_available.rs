use crate::Fault;

use super::{is_map_cleared, AppContext};

pub fn invasion_z_available(ctx: &mut AppContext, chapter: i32) -> Result<bool, Fault> {
    if chapter != 9 || ctx.u8_at(AppContext::MAP_NEG15_CLEARED)? == 0 || ctx.u8_at(AppContext::MAP_NEG25_CLEARED)? != 0 {
        return Ok(false);
    }

    if !is_map_cleared(ctx, -0x16, 0, 0, 0)? {
        return Ok(false);
    }

    let mut next = 0;

    loop {
        let chapter = next;

        if chapter != 3 {
            let moon = ctx.outbreak_cleared.entry(chapter).or_default().entry(0x2f).or_default();

            if !*moon {
                return Ok(false);
            }
        }

        next = chapter + 1;

        if next == 0xa {
            return Ok(chapter as u32 >= 9);
        }
    }
}
