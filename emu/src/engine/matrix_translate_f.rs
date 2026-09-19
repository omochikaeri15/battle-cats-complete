pub fn matrix_translate_f(mat: &mut [f32; 6], x: f32, y: f32) {
    mat[2] += mat[0] * x + mat[1] * y;
    mat[5] += mat[3] * x + mat[4] * y;
}
