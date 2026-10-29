pub fn matrix_set_translation(mat: &mut [f32; 6], x: i32, y: i32) {
    let across = x as f32;

    mat[0] = 1.0;
    mat[1] = 0.0;
    mat[2] = across;
    mat[3] = 0.0;
    mat[4] = 1.0;
    mat[5] = y as f32;
}
