use crate::Fault;

use super::{DrawSink, Mamodel, draw_model_range};

pub fn draw_model(dc: &mut dyn DrawSink, model: &Mamodel, x: i32, y: i32) -> Result<(), Fault> {
    draw_model_range(dc, model, x, y, 0, model.parts.len() as i32)
}
