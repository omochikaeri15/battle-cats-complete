use crate::{Fault, ops};

use super::{DrawSink, Mamodel, draw_sprite_cut};

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
) -> Result<(), Fault> {
    let held_glow = dc.glow();
    let held_color = dc.color();

    if !model.parts.is_empty() {
        let shift_y = second.wrapping_sub(pivot_y);
        let shift_x = first.wrapping_sub(pivot_x);
        let mut turn = 0usize;

        loop {
            let index = *model
                .draw_order
                .get(turn)
                .ok_or(Fault::index_out_of_range(turn as i64, model.draw_order.len() as i64))?;
            let part = model
                .parts
                .get(index as i64 as usize)
                .ok_or(Fault::index_out_of_range(index as i64, model.parts.len() as i64))?;

            'part: {
                if part.i32_at(0x28).wrapping_add(part.i32_at(0x24)) == -1 {
                    break 'part;
                }

                let blend = part.i32_at(0x8c);

                if blend as u32 <= 3 {
                    dc.glow_set(blend);
                }

                let faded = ops::idiv(
                    (part.i32_at(0x80) << 8).wrapping_sub(part.i32_at(0x80)),
                    model.opacity_unit,
                )
                .ok_or(Fault::divide(model.opacity_unit as i64))?
                .wrapping_mul(alpha);

                if (faded.wrapping_add(0xfe) as u32) < 0x1fd {
                    break 'part;
                }

                dc.set_alpha(ops::div_255(faded));

                let at = if model.single_sheet == 0 {
                    (part.i32_at(0x24) as i64).wrapping_add(part.i32_at(0x28) as i64)
                } else {
                    0
                };
                let cut = part.i32_at(0x30).wrapping_add(part.i32_at(0x2c));
                let x0 = ops::cvttss2si(
                    part.i32_at(0x90).wrapping_add(shift_x) as f32 * scale + pivot_x as f32 + x as f32,
                );
                let y0 = ops::cvttss2si(
                    part.i32_at(0x94).wrapping_add(shift_y) as f32 * scale + pivot_y as f32 + y as f32,
                );
                let x1 = ops::cvttss2si(
                    part.i32_at(0x98).wrapping_add(shift_x) as f32 * scale + pivot_x as f32 + x as f32,
                );
                let y1 = ops::cvttss2si(
                    part.i32_at(0x9c).wrapping_add(shift_y) as f32 * scale + pivot_y as f32 + y as f32,
                );
                let x2 = ops::cvttss2si(
                    part.i32_at(0xa0).wrapping_add(shift_x) as f32 * scale + pivot_x as f32 + x as f32,
                );
                let y2 = ops::cvttss2si(
                    part.i32_at(0xa4).wrapping_add(shift_y) as f32 * scale + pivot_y as f32 + y as f32,
                );
                let x3 = ops::cvttss2si(
                    part.i32_at(0xa8).wrapping_add(shift_x) as f32 * scale + pivot_x as f32 + x as f32,
                );
                let y3 = ops::cvttss2si(
                    part.i32_at(0xac).wrapping_add(shift_y) as f32 * scale + pivot_y as f32 + y as f32,
                );
                let slot = model
                    .sheet_table
                    .get(at as usize)
                    .ok_or(Fault::index_out_of_range(at, model.sheet_table.len() as i64))?;
                let held = slot.take();

                slot.set(held.clone());

                let sheet = held.as_deref().ok_or(Fault::null_pointer())?;

                draw_sprite_cut(dc, sheet, x0, y0, x1, y1, x2, y2, x3, y3, cut);
            }

            turn += 1;

            if model.parts.len() <= turn {
                break;
            }
        }
    }

    dc.glow_set(held_glow);
    dc.set_color(held_color[0], held_color[1], held_color[2], held_color[3]);

    Ok(())
}
