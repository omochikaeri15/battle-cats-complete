use crate::Fault;

use super::{AppContext, NyancomboRecord};

pub fn combo_sort_less(
    ctx: &AppContext,
    left: &NyancomboRecord,
    right: &NyancomboRecord,
) -> Result<bool, Fault> {
    if left.kind[0] == right.kind[0] {
        return Ok(left.power[0] < right.power[0]);
    }

    let mut entry = 0usize;

    loop {
        let tab = ctx.i32_at(AppContext::COMBO_TAB)?;
        let kinds =
            ctx.combo_store
                .tab_kinds
                .get(tab as i64 as usize)
                .ok_or(Fault::index_out_of_range(tab as i64, ctx.combo_store.tab_kinds.len() as i64))?;

        if entry as i64 >= kinds.len() as i32 as i64 {
            return Ok(false);
        }

        if kinds[entry] == left.kind[0] {
            return Ok(true);
        }

        if kinds[entry] == right.kind[0] {
            return Ok(false);
        }

        entry += 1;
    }
}
