const TRAIT_COLORS: [[i32; 3]; 13] = [
    [210, 72, 40],
    [62, 205, 44],
    [28, 31, 22],
    [117, 119, 118],
    [240, 221, 107],
    [114, 236, 213],
    [198, 105, 238],
    [129, 154, 100],
    [255, 255, 255],
    [0, 0, 0],
    [0, 0, 0],
    [38, 161, 234],
    [255, 255, 255],
];

pub fn orb_trait_color(trait_index: i32) -> [i32; 3] {
    TRAIT_COLORS
        .get(trait_index as i64 as usize)
        .copied()
        .unwrap_or([0, 0, 0])
}
