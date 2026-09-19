use crate::operation;

pub fn map_index_of_map_id(map_id: i32) -> i32 {
    let offset = map_id.wrapping_sub(0xbb8) as u32;

    if offset < 3 {
        return offset as i32;
    }

    let offset = map_id.wrapping_sub(0xbbb) as u32;

    if offset < 3 {
        return offset as i32;
    }

    let offset = map_id.wrapping_sub(0xbbe) as u32;

    if offset < 3 {
        return offset as i32;
    }

    map_id.wrapping_sub(operation::div_1000(map_id).wrapping_mul(0x3e8))
}
