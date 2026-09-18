pub fn matrix_translate(mat: &mut [f32; 6], x: i32, y: i32) {
    let across = x as f32;
    let down = y as f32;

    mat[2] += mat[0] * across + mat[1] * down;
    mat[5] += mat[4] * down + mat[3] * across;
}
