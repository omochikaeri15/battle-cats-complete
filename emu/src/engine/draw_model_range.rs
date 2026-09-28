use crate::Fault;

use super::{DrawSink, Mamodel, draw_part};

pub fn draw_model_range(
    dc: &mut dyn DrawSink,
    model: &Mamodel,
    ox: i32,
    oy: i32,
    first: i32,
    count: i32,
) -> Result<(), Fault> {
    let held_glow = dc.glow();
    let held_color = dc.color();

    if count > 0 {
        let mut turn = first as i64;

        loop {
            let index = *model
                .draw_order
                .get(turn as usize)
                .ok_or(Fault::index_out_of_range(turn, model.draw_order.len() as i64))?;

            draw_part(index, model, dc, ox, oy)?;
            turn += 1;

            if turn >= count.wrapping_add(first) as i64 {
                break;
            }
        }
    }

    dc.glow_set(held_glow);
    dc.set_color(held_color[0], held_color[1], held_color[2], held_color[3]);

    Ok(())
}
