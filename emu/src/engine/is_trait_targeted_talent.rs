pub fn is_trait_targeted_talent(abil: i32) -> bool {
    matches!(abil.wrapping_sub(1) as u32, 0x00..=0x08 | 0x32 | 0x3b)
}
