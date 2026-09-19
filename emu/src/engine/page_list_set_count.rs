use crate::Fault;

use super::{AppContext, PageList, get_drawable_width};

const SITE: &str = "page_list_set_count";

pub fn page_list_set_count(ctx: &mut AppContext, list: usize, count: i32) -> Result<(), Fault> {
    ctx.set_block_at::<1>(list + PageList::MULTI_PAGE, [(count >= 2) as u8])?;
    ctx.set_i32_at(list + PageList::PAGE_COUNT, count)?;
    ctx.set_i32_at(list + PageList::PAGE, 0)?;
    ctx.set_i32_at(list + PageList::PAGE_TARGET, 0)?;
    ctx.set_i32_at(list + PageList::OFFSET, 0)?;
    ctx.set_i32_at(list + PageList::OFFSET_TARGET, 0)?;

    if ctx.u8_at(list + PageList::ANIMATED)? != 0 {
        let center = (get_drawable_width(ctx)? / 2) as f32;

        ctx.ui()
            .ok_or(Fault::HostMissing { site: SITE })?
            .page_list_layout(list, count, 10, center, 60.0, 600.0);
    }

    Ok(())
}
