use super::{AssetStream, Cell};

pub fn get_cell<'s>(stm: &'s AssetStream<'_>, col: i32) -> Option<&'s Cell> {
    let index = col as i64 as u64;

    if stm.cells.len() as u64 > index {
        return stm.cells.get(index as usize);
    }

    None
}
