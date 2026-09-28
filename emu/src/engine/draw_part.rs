use std::rc::Rc;

use crate::{Fault, ops};

use super::{DrawSink, Mamodel, draw_sprite_cut, glow_set, set_alpha};

pub fn draw_part(
    part_index: i32,
    model: &Mamodel,
    dc: &mut dyn DrawSink,
    ox: i32,
    oy: i32,
) -> Result<(), Fault> {
    let part = model
        .parts
        .get(part_index as i64 as usize)
        .ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;

    if part.i32_at(0x28).wrapping_add(part.i32_at(0x24)) == -1 {
        return Ok(());
    }

    let blend = part.i32_at(0x8c);

    if blend as u32 <= 3 {
        glow_set(dc, blend);
    }

    let alpha = ops::idiv(
        (part.i32_at(0x80) << 8).wrapping_sub(part.i32_at(0x80)),
        model.opacity_unit,
    )
    .ok_or(Fault::divide(model.opacity_unit as i64))?;

    if alpha == 0 {
        return Ok(());
    }

    set_alpha(dc, alpha);

    let sheet = match &model.sheet {
        Some(sheet) => Some(Rc::clone(sheet)),
        None => {
            let at = if model.single_sheet == 0 {
                (part.i32_at(0x24) as i64).wrapping_add(part.i32_at(0x28) as i64)
            } else {
                0
            };
            let slot = model
                .sheet_table
                .get(at as usize)
                .ok_or(Fault::index_out_of_range(at, model.sheet_table.len() as i64))?;
            let held = slot.take();

            slot.set(held.clone());
            held
        }
    };
    let sheet = sheet.as_deref().ok_or(Fault::null_pointer())?;

    draw_sprite_cut(
        dc,
        sheet,
        part.i32_at(0x90).wrapping_add(ox),
        part.i32_at(0x94).wrapping_add(oy),
        part.i32_at(0x98).wrapping_add(ox),
        part.i32_at(0x9c).wrapping_add(oy),
        part.i32_at(0xa0).wrapping_add(ox),
        part.i32_at(0xa4).wrapping_add(oy),
        ox.wrapping_add(part.i32_at(0xa8)),
        oy.wrapping_add(part.i32_at(0xac)),
        part.i32_at(0x30).wrapping_add(part.i32_at(0x2c)),
    );

    Ok(())
}
