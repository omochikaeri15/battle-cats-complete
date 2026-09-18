pub fn atan2_deg(y: f32, x: f32) -> f32 {
    let mut degrees = ((y.atan2(x) * 180.0) as f64) / std::f64::consts::PI;

    if y < 0.0 || y.is_nan() {
        degrees += 360.0;
    }

    degrees as f32
}
