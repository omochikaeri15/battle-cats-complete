use crate::operation;

pub fn transform_point_f(mat: &[f32; 6], x: f32, y: f32, out: &mut [i32; 2]) {
    out[0] = operation::cvttss2si(x * mat[0] + y * mat[1] + mat[2]);
    out[1] = operation::cvttss2si(x * mat[3] + y * mat[4] + mat[5]);
}
