use std::rc::Rc;

use crate::{Fault, ops};

use super::{DrawSink, Mamodel, MamodelPart, draw_sprite_cut, glow_set, set_alpha};

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

    if part.i32_at(MamodelPart::SHEET_ANIM).wrapping_add(part.i32_at(MamodelPart::SHEET)) == -1 {
        return Ok(());
    }

    let blend = part.i32_at(MamodelPart::GLOW);

    if blend as u32 <= 3 {
        glow_set(dc, blend);
    }

    let alpha = ops::idiv(
        (part.i32_at(MamodelPart::LIVE_OPACITY) << 8).wrapping_sub(part.i32_at(MamodelPart::LIVE_OPACITY)),
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
                (part.i32_at(MamodelPart::SHEET) as i64).wrapping_add(part.i32_at(MamodelPart::SHEET_ANIM) as i64)
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
        part.i32_at(MamodelPart::QUAD_0_X).wrapping_add(ox),
        part.i32_at(MamodelPart::QUAD_0_Y).wrapping_add(oy),
        part.i32_at(MamodelPart::QUAD_1_X).wrapping_add(ox),
        part.i32_at(MamodelPart::QUAD_1_Y).wrapping_add(oy),
        part.i32_at(MamodelPart::QUAD_2_X).wrapping_add(ox),
        part.i32_at(MamodelPart::QUAD_2_Y).wrapping_add(oy),
        ox.wrapping_add(part.i32_at(MamodelPart::QUAD_3_X)),
        oy.wrapping_add(part.i32_at(MamodelPart::QUAD_3_Y)),
        part.i32_at(MamodelPart::CUT_ANIM).wrapping_add(part.i32_at(MamodelPart::CUT)),
    );

    Ok(())
}
