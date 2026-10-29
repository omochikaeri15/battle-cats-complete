pub fn cos_deg(degrees: f32) -> f32 {
    ((degrees as f64) * std::f64::consts::PI / 180.0).cos() as f32
}
