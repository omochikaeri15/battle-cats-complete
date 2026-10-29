use super::{cos_deg, sin_deg};

pub fn matrix_rotate(mat: &mut [f32; 6], degrees: f32) {
    let cos = cos_deg(degrees);
    let sin = sin_deg(degrees);
    let a = mat[0];
    let b = mat[1];

    mat[0] = a * cos + sin * b;
    mat[1] = b * cos - a * sin;

    let c = mat[3];
    let d = mat[4];

    mat[3] = c * cos + d * sin;
    mat[4] = cos * d - c * sin;
}
