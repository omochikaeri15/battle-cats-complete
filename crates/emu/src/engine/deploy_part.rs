use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    Mamodel, MamodelPart, Vector, imgcut_get_cut_count, imgcut_get_height, imgcut_get_sprite_cut,
    imgcut_get_width, matrix_rotate, matrix_set_translation, matrix_translate, transform_point,
};

pub fn deploy_part(part_index: i32, model: &mut Mamodel) -> Result<(), Fault> {
    let missing = Fault::index_out_of_range(part_index as i64, model.parts.len() as i64);
    let mut part = *model.parts.get(part_index as i64 as usize).ok_or(missing)?;

    let parent = part.i32_at(MamodelPart::PARENT_ANIM).wrapping_add(part.i32_at(MamodelPart::PARENT));
    let own_scale_x = part.i32_at(MamodelPart::SCALE_X) as i64;
    let scale_x;
    let scale_y;
    let opacity;
    let flip_x;
    let flip_y;

    if parent == -1 {
        let scale_unit = model.scale_unit;
        let across = ops::idiv(
            (own_scale_x as i32).wrapping_mul(part.i32_at(MamodelPart::SCALE_X_ANIM)),
            scale_unit,
        )
        .ok_or(Fault::divide(scale_unit as i64))?;

        scale_x = if model.mirror == 0 {
            across
        } else {
            across.wrapping_neg()
        };
        part.set_i32_at(MamodelPart::WORLD_SCALE_X, scale_x);

        scale_y = ops::idiv(
            part.i32_at(MamodelPart::SCALE_Y_ANIM).wrapping_mul(part.i32_at(MamodelPart::SCALE_Y)),
            scale_unit,
        )
        .ok_or(Fault::divide(scale_unit as i64))?;
        part.set_i32_at(MamodelPart::WORLD_SCALE_Y, scale_y);

        opacity = ops::idiv(
            part.i32_at(MamodelPart::OPACITY_ANIM).wrapping_mul(part.i32_at(MamodelPart::OPACITY)),
            model.opacity_unit,
        )
        .ok_or(Fault::divide(model.opacity_unit as i64))?;
        flip_x = part.u8_at(MamodelPart::H_FLIP);
        flip_y = part.u8_at(MamodelPart::V_FLIP);
    } else {
        let above = *model
            .parts
            .get(parent as i64 as usize)
            .ok_or(Fault::index_out_of_range(parent as i64, model.parts.len() as i64))?;
        let scale_unit = model.scale_unit as i64;

        let across = (above.i32_at(MamodelPart::WORLD_SCALE_X) as i64)
            .wrapping_mul(part.i32_at(MamodelPart::SCALE_X_ANIM) as i64)
            .wrapping_mul(own_scale_x);
        let across =
            ops::div_wide(across, scale_unit).ok_or(Fault::divide(scale_unit))?;
        scale_x =
            ops::div_wide(across, scale_unit).ok_or(Fault::divide(scale_unit))? as i32;
        part.set_i32_at(MamodelPart::WORLD_SCALE_X, scale_x);

        let down = (above.i32_at(MamodelPart::WORLD_SCALE_Y) as i64)
            .wrapping_mul(part.i32_at(MamodelPart::SCALE_Y_ANIM) as i64)
            .wrapping_mul(part.i32_at(MamodelPart::SCALE_Y) as i64);
        let down = ops::div_wide(down, scale_unit).ok_or(Fault::divide(scale_unit))?;
        scale_y =
            ops::div_wide(down, scale_unit).ok_or(Fault::divide(scale_unit))? as i32;
        part.set_i32_at(MamodelPart::WORLD_SCALE_Y, scale_y);

        let opacity_unit = model.opacity_unit as i64;
        let faded = (above.i32_at(MamodelPart::WORLD_OPACITY) as i64)
            .wrapping_mul(part.i32_at(MamodelPart::OPACITY_ANIM) as i64)
            .wrapping_mul(part.i32_at(MamodelPart::OPACITY) as i64);
        let faded =
            ops::div_wide(faded, opacity_unit).ok_or(Fault::divide(opacity_unit))?;
        opacity = ops::div_wide(faded, opacity_unit)
            .ok_or(Fault::divide(opacity_unit))? as i32;

        flip_x = (part.u8_at(MamodelPart::H_FLIP) != above.u8_at(MamodelPart::WORLD_H_FLIP)) as u8;
        flip_y = (part.u8_at(MamodelPart::V_FLIP) != above.u8_at(MamodelPart::WORLD_V_FLIP)) as u8;
    }

    part.set_i32_at(MamodelPart::WORLD_OPACITY, opacity);
    part.set_u8_at(MamodelPart::WORLD_H_FLIP, flip_x);
    part.set_u8_at(MamodelPart::WORLD_V_FLIP, flip_y);

    if part.u8_at(MamodelPart::H_FLIP) != 0 {
        part.set_i32_at(MamodelPart::WORLD_SCALE_X, scale_x.wrapping_neg());
    }

    if part.u8_at(MamodelPart::V_FLIP) != 0 {
        part.set_i32_at(MamodelPart::WORLD_SCALE_Y, scale_y.wrapping_neg());
    }

    let sheet_id = (part.i32_at(MamodelPart::SHEET) as i64).wrapping_add(part.i32_at(MamodelPart::SHEET_ANIM) as i64);

    if sheet_id as i32 != -1 {
        let sheet = match &model.sheet {
            Some(sheet) => Some(Rc::clone(sheet)),
            None => {
                let at = if model.single_sheet == 0 { sheet_id } else { 0 };
                let slot = model
                    .sheet_table
                    .get(at as usize)
                    .ok_or(Fault::index_out_of_range(at, model.sheet_table.len() as i64))?;
                let held = slot.take();

                slot.set(held.clone());
                held
            }
        };

        let Some(sheet) = sheet.as_deref() else {
            *model
                .parts
                .get_mut(part_index as i64 as usize)
                .ok_or(Fault::null_pointer())? = part;

            return Ok(());
        };

        let width;
        let height;

        if sheet.whole != 0 {
            width = imgcut_get_width(sheet);
            height = imgcut_get_height(sheet);
        } else {
            let cut = part.i32_at(MamodelPart::SPRITE_ANIM).wrapping_add(part.i32_at(MamodelPart::SPRITE));

            if cut < 0 || cut >= imgcut_get_cut_count(sheet) as i32 {
                *model
                    .parts
                    .get_mut(part_index as i64 as usize)
                    .ok_or(Fault::null_pointer())? = part;

                return Ok(());
            }

            width =
                imgcut_get_sprite_cut(sheet, part.i32_at(MamodelPart::SPRITE_ANIM).wrapping_add(part.i32_at(MamodelPart::SPRITE)))?[2];
            height =
                imgcut_get_sprite_cut(sheet, part.i32_at(MamodelPart::SPRITE_ANIM).wrapping_add(part.i32_at(MamodelPart::SPRITE)))?[3];
        }

        let live_x = part.i32_at(MamodelPart::WORLD_SCALE_X);
        let left = ops::idiv(
            part.i32_at(MamodelPart::PIVOT_X_ANIM)
                .wrapping_add(part.i32_at(MamodelPart::PIVOT_X))
                .wrapping_mul(live_x)
                .wrapping_neg(),
            model.scale_unit,
        )
        .ok_or(Fault::divide(model.scale_unit as i64))?;
        part.set_i32_at(MamodelPart::CORNER_1_X, left);
        part.set_i32_at(MamodelPart::CORNER_0_X, left);

        let right = ops::idiv(width.wrapping_mul(live_x), model.scale_unit)
            .ok_or(Fault::divide(model.scale_unit as i64))?
            .wrapping_add(left);
        part.set_i32_at(MamodelPart::CORNER_3_X, right);
        part.set_i32_at(MamodelPart::CORNER_2_X, right);

        let live_y = part.i32_at(MamodelPart::WORLD_SCALE_Y);
        let top = ops::idiv(
            part.i32_at(MamodelPart::PIVOT_Y_ANIM)
                .wrapping_add(part.i32_at(MamodelPart::PIVOT_Y))
                .wrapping_mul(live_y)
                .wrapping_neg(),
            model.scale_unit,
        )
        .ok_or(Fault::divide(model.scale_unit as i64))?;
        part.set_i32_at(MamodelPart::CORNER_3_Y, top);
        part.set_i32_at(MamodelPart::CORNER_0_Y, top);

        let bottom = ops::idiv(height.wrapping_mul(live_y), model.scale_unit)
            .ok_or(Fault::divide(model.scale_unit as i64))?
            .wrapping_add(top);
        part.set_i32_at(MamodelPart::CORNER_2_Y, bottom);
        part.set_i32_at(MamodelPart::CORNER_1_Y, bottom);
    }

    let parent = part.i32_at(MamodelPart::PARENT_ANIM).wrapping_add(part.i32_at(MamodelPart::PARENT));
    let mut mat = [
        part.f32_at(MamodelPart::MATRIX_0),
        part.f32_at(MamodelPart::MATRIX_1),
        part.f32_at(MamodelPart::MATRIX_2),
        part.f32_at(MamodelPart::MATRIX_3),
        part.f32_at(MamodelPart::MATRIX_4),
        part.f32_at(MamodelPart::MATRIX_5),
    ];

    if parent == -1 {
        matrix_set_translation(&mut mat, part.i32_at(MamodelPart::X_ANIM), part.i32_at(MamodelPart::Y_ANIM));
    } else {
        let above = *model
            .parts
            .get(parent as i64 as usize)
            .ok_or(Fault::index_out_of_range(parent as i64, model.parts.len() as i64))?;

        mat = [
            above.f32_at(MamodelPart::MATRIX_0),
            above.f32_at(MamodelPart::MATRIX_1),
            above.f32_at(MamodelPart::MATRIX_2),
            above.f32_at(MamodelPart::MATRIX_3),
            above.f32_at(MamodelPart::MATRIX_4),
            above.f32_at(MamodelPart::MATRIX_5),
        ];

        let across = ops::idiv(
            part.i32_at(MamodelPart::X_ANIM)
                .wrapping_add(part.i32_at(MamodelPart::X))
                .wrapping_mul(above.i32_at(MamodelPart::WORLD_SCALE_X)),
            model.scale_unit,
        )
        .ok_or(Fault::divide(model.scale_unit as i64))?;
        let down = ops::idiv(
            part.i32_at(MamodelPart::Y_ANIM)
                .wrapping_add(part.i32_at(MamodelPart::Y))
                .wrapping_mul(above.i32_at(MamodelPart::WORLD_SCALE_Y)),
            model.scale_unit,
        )
        .ok_or(Fault::divide(model.scale_unit as i64))?;

        matrix_translate(&mut mat, across, down);
    }

    let mut degrees = (part.i32_at(MamodelPart::ANGLE_ANIM).wrapping_add(part.i32_at(MamodelPart::ANGLE)) as f32) * 360.0
        / (model.angle_unit as f32);

    if model.mirror != 0 {
        degrees = -degrees;
    }

    if part.u8_at(MamodelPart::WORLD_H_FLIP) != part.u8_at(MamodelPart::WORLD_V_FLIP) {
        degrees = -degrees;
    }

    matrix_rotate(&mut mat, degrees);

    for (cell, value) in mat.iter().enumerate() {
        part.set_f32_at(MamodelPart::MATRIX_0 + cell * 4, *value);
    }

    for corner in [
        MamodelPart::CORNER_0_X,
        MamodelPart::CORNER_1_X,
        MamodelPart::CORNER_2_X,
        MamodelPart::CORNER_3_X,
    ] {
        let mut out = 0i64;

        transform_point(&mat, part.i32_at(corner), part.i32_at(corner + Vector::Y), &mut out);
        part.set_i32_at(corner, out as i32);
        part.set_i32_at(corner + Vector::Y, (out >> 0x20) as i32);
    }

    *model
        .parts
        .get_mut(part_index as i64 as usize)
        .ok_or(Fault::null_pointer())? = part;

    Ok(())
}
