pub fn matrix_set_translation_f(mat: &mut [f32; 6], x: f32, y: f32) {
    mat[0] = 1.0;
    mat[1] = 0.0;
    mat[2] = x;
    mat[3] = 0.0;
    mat[4] = 1.0;
    mat[5] = y;
}
