use crate::Fault;

use super::{AppContext, PageList};

pub fn page_list_set_page(ctx: &mut AppContext, list: usize, page: i32) -> Result<(), Fault> {
    let offset = ctx.i32_at(list + PageList::PAGE_WIDTH)?.wrapping_mul(page).wrapping_neg();

    ctx.set_i32_at(list + PageList::OFFSET, offset)?;
    ctx.set_i32_at(list + PageList::OFFSET_TARGET, offset)?;
    ctx.set_i32_at(list + PageList::PAGE, page)
}
