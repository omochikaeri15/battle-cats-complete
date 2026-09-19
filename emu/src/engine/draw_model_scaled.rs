use super::{DrawSink, Mamodel};

#[allow(clippy::too_many_arguments)]
pub fn draw_model_scaled(
    dc: &mut dyn DrawSink,
    model: &Mamodel,
    x: i32,
    y: i32,
    pivot_x: i32,
    pivot_y: i32,
    scale: f32,
    alpha: i32,
    first: i32,
    second: i32,
) {
    dc.draw_model_scaled(model, x, y, pivot_x, pivot_y, scale, alpha, first, second);
}
