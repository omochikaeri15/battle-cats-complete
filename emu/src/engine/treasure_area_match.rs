pub fn treasure_area_match(chapter: i32, area: i32) -> bool {
    if chapter < 0 || chapter == 3 {
        return false;
    }

    if (area.wrapping_sub(1) as u32) <= 2 {
        return (chapter as u32) < area as u32;
    }

    (area.wrapping_sub(4) as u32) < 6 && chapter as u32 <= area as u32
}
