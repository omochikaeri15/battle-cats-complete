use super::{cos_deg, sin_deg};

pub fn matrix_set_rotation(mat: &mut [f32; 6], degrees: f32) {
    let cos = cos_deg(degrees);
    let sin = sin_deg(degrees);

    mat[0] = cos;
    mat[1] = -sin;
    mat[2] = 0.0;
    mat[3] = sin;
    mat[4] = cos;
    mat[5] = 0.0;
}
