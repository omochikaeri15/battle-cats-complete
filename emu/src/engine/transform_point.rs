const I32_FLOOR: f32 = -2147483648.0;
const I32_CEILING: f32 = 2147483648.0;

pub fn transform_point(mat: &[f32; 6], x: i32, y: i32, out: &mut i64) {
    let x = x as f32;
    let y = y as f32;

    let placed_x = x * mat[0] + y * mat[1] + mat[2];
    let placed_y = x * mat[3] + y * mat[4] + mat[5];

    let out_x = match (I32_FLOOR..I32_CEILING).contains(&placed_x) {
        true => placed_x as i32,
        false => i32::MIN,
    };

    let out_y = match (I32_FLOOR..I32_CEILING).contains(&placed_y) {
        true => placed_y as i32,
        false => i32::MIN,
    };

    *out = ((out_y as u32 as i64) << 32) | out_x as u32 as i64;
}
