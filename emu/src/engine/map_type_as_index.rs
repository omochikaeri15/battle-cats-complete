pub fn map_type_as_index(map_type: i32) -> i32 {
    if (map_type.wrapping_add(0x1a) as u32) < 0x1f {
        map_type
    } else {
        -1
    }
}
