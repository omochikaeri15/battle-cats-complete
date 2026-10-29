pub fn matrix_multiply(mat: &mut [f32; 6], other: &[f32; 6]) {
    let old = *mat;

    mat[0] = old[0] * other[0] + old[1] * other[3];
    mat[1] = old[0] * other[1] + old[1] * other[4];
    mat[2] = old[0] * other[2] + old[1] * other[5] + old[2];
    mat[3] = old[3] * other[0] + old[4] * other[3];
    mat[4] = old[3] * other[1] + old[4] * other[4];
    mat[5] = old[3] * other[2] + old[4] * other[5] + old[5];
}
