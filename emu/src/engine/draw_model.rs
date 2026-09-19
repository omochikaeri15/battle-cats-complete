use super::{DrawSink, Mamodel};

pub fn draw_model(dc: &mut dyn DrawSink, model: &Mamodel, x: i32, y: i32) {
    dc.draw_model(model, x, y);
}
