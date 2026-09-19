use crate::Fault;

use super::AppContext;

pub fn find_item_index(ctx: &AppContext, item_id: i32) -> Result<i32, Fault> {
    let mut index: usize = 0;

    loop {
        let cell = ctx.block_at::<8>(AppContext::ITEM_DEFINITION_IDS.wrapping_add(index.wrapping_mul(AppContext::ITEM_DEFINITION_STRIDE)))?;
        let id = ((cell[7] ^ cell[0]) as u32
            | ((cell[6] ^ cell[1]) as u32) << 8
            | ((cell[5] ^ cell[2]) as u32) << 0x10
            | ((cell[4] ^ cell[3]) as u32) << 0x18) as i32;

        if id == item_id {
            return Ok(index as i32);
        }

        index += 1;

        if index == 0x113 {
            return Ok(-1);
        }
    }
}
