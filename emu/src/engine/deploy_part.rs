use std::rc::Rc;

use crate::{Fault, operation};

use super::{
    Mamodel, imgcut_get_cut_count, imgcut_get_height, imgcut_get_sprite_cut, imgcut_get_width,
    matrix_rotate, matrix_set_translation, matrix_translate, transform_point,
};

const SITE: &str = "deploy_part";

pub fn deploy_part(part_index: i32, model: &mut Mamodel) -> Result<(), Fault> {
    let missing = Fault::IndexOutOfRange {
        site: SITE,
        index: part_index as i64,
        limit: model.parts.len() as i64,
    };
    let mut part = *model.parts.get(part_index as i64 as usize).ok_or(missing)?;

    let parent = part.i32_at(0x20).wrapping_add(part.i32_at(0x1c));
    let own_scale_x = part.i32_at(0x5c) as i64;
    let scale_x;
    let scale_y;
    let opacity;
    let flip_x;
    let flip_y;

    if parent == -1 {
        let scale_unit = model.scale_unit;
        let across = operation::idiv(
            (own_scale_x as i32).wrapping_mul(part.i32_at(0x64)),
            scale_unit,
        )
        .ok_or(Fault::divide(SITE, scale_unit as i64))?;

        scale_x = if model.mirror == 0 {
            across
        } else {
            across.wrapping_neg()
        };
        part.set_i32_at(0x60, scale_x);

        scale_y = operation::idiv(
            part.i32_at(0x70).wrapping_mul(part.i32_at(0x68)),
            scale_unit,
        )
        .ok_or(Fault::divide(SITE, scale_unit as i64))?;
        part.set_i32_at(0x6c, scale_y);

        opacity = operation::idiv(
            part.i32_at(0x84).wrapping_mul(part.i32_at(0x7c)),
            model.opacity_unit,
        )
        .ok_or(Fault::divide(SITE, model.opacity_unit as i64))?;
        flip_x = part.u8_at(0x88);
        flip_y = part.u8_at(0x8a);
    } else {
        let above = *model
            .parts
            .get(parent as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: parent as i64,
                limit: model.parts.len() as i64,
            })?;
        let scale_unit = model.scale_unit as i64;

        let across = (above.i32_at(0x60) as i64)
            .wrapping_mul(part.i32_at(0x64) as i64)
            .wrapping_mul(own_scale_x);
        let across =
            operation::div_wide(across, scale_unit).ok_or(Fault::divide(SITE, scale_unit))?;
        scale_x =
            operation::div_wide(across, scale_unit).ok_or(Fault::divide(SITE, scale_unit))? as i32;
        part.set_i32_at(0x60, scale_x);

        let down = (above.i32_at(0x6c) as i64)
            .wrapping_mul(part.i32_at(0x70) as i64)
            .wrapping_mul(part.i32_at(0x68) as i64);
        let down = operation::div_wide(down, scale_unit).ok_or(Fault::divide(SITE, scale_unit))?;
        scale_y =
            operation::div_wide(down, scale_unit).ok_or(Fault::divide(SITE, scale_unit))? as i32;
        part.set_i32_at(0x6c, scale_y);

        let opacity_unit = model.opacity_unit as i64;
        let faded = (above.i32_at(0x80) as i64)
            .wrapping_mul(part.i32_at(0x84) as i64)
            .wrapping_mul(part.i32_at(0x7c) as i64);
        let faded =
            operation::div_wide(faded, opacity_unit).ok_or(Fault::divide(SITE, opacity_unit))?;
        opacity = operation::div_wide(faded, opacity_unit)
            .ok_or(Fault::divide(SITE, opacity_unit))? as i32;

        flip_x = (part.u8_at(0x88) != above.u8_at(0x89)) as u8;
        flip_y = (part.u8_at(0x8a) != above.u8_at(0x8b)) as u8;
    }

    part.set_i32_at(0x80, opacity);
    part.set_u8_at(0x89, flip_x);
    part.set_u8_at(0x8b, flip_y);

    if part.u8_at(0x88) != 0 {
        part.set_i32_at(0x60, scale_x.wrapping_neg());
    }

    if part.u8_at(0x8a) != 0 {
        part.set_i32_at(0x6c, scale_y.wrapping_neg());
    }

    let sheet_id = (part.i32_at(0x24) as i64).wrapping_add(part.i32_at(0x28) as i64);

    if sheet_id as i32 != -1 {
        let sheet = match &model.sheet {
            Some(sheet) => Some(Rc::clone(sheet)),
            None => {
                let at = if model.single_sheet == 0 { sheet_id } else { 0 };
                let slot = model
                    .sheet_table
                    .get(at as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: at,
                        limit: model.sheet_table.len() as i64,
                    })?;
                let held = slot.take();

                slot.set(held.clone());
                held
            }
        };

        let Some(sheet) = sheet.as_deref() else {
            *model
                .parts
                .get_mut(part_index as i64 as usize)
                .ok_or(Fault::NullPointer { site: SITE })? = part;

            return Ok(());
        };

        let width;
        let height;

        if sheet.whole != 0 {
            width = imgcut_get_width(sheet);
            height = imgcut_get_height(sheet);
        } else {
            let cut = part.i32_at(0x30).wrapping_add(part.i32_at(0x2c));

            if cut < 0 || cut >= imgcut_get_cut_count(sheet) as i32 {
                *model
                    .parts
                    .get_mut(part_index as i64 as usize)
                    .ok_or(Fault::NullPointer { site: SITE })? = part;

                return Ok(());
            }

            width =
                imgcut_get_sprite_cut(sheet, part.i32_at(0x30).wrapping_add(part.i32_at(0x2c)))?[2];
            height =
                imgcut_get_sprite_cut(sheet, part.i32_at(0x30).wrapping_add(part.i32_at(0x2c)))?[3];
        }

        let live_x = part.i32_at(0x60);
        let left = operation::idiv(
            part.i32_at(0x54)
                .wrapping_add(part.i32_at(0x4c))
                .wrapping_mul(live_x)
                .wrapping_neg(),
            model.scale_unit,
        )
        .ok_or(Fault::divide(SITE, model.scale_unit as i64))?;
        part.set_i32_at(0x98, left);
        part.set_i32_at(0x90, left);

        let right = operation::idiv(width.wrapping_mul(live_x), model.scale_unit)
            .ok_or(Fault::divide(SITE, model.scale_unit as i64))?
            .wrapping_add(left);
        part.set_i32_at(0xa8, right);
        part.set_i32_at(0xa0, right);

        let live_y = part.i32_at(0x6c);
        let top = operation::idiv(
            part.i32_at(0x58)
                .wrapping_add(part.i32_at(0x50))
                .wrapping_mul(live_y)
                .wrapping_neg(),
            model.scale_unit,
        )
        .ok_or(Fault::divide(SITE, model.scale_unit as i64))?;
        part.set_i32_at(0xac, top);
        part.set_i32_at(0x94, top);

        let bottom = operation::idiv(height.wrapping_mul(live_y), model.scale_unit)
            .ok_or(Fault::divide(SITE, model.scale_unit as i64))?
            .wrapping_add(top);
        part.set_i32_at(0xa4, bottom);
        part.set_i32_at(0x9c, bottom);
    }

    let parent = part.i32_at(0x20).wrapping_add(part.i32_at(0x1c));
    let mut mat = [
        part.f32_at(0x04),
        part.f32_at(0x08),
        part.f32_at(0x0c),
        part.f32_at(0x10),
        part.f32_at(0x14),
        part.f32_at(0x18),
    ];

    if parent == -1 {
        matrix_set_translation(&mut mat, part.i32_at(0x44), part.i32_at(0x48));
    } else {
        let above = *model
            .parts
            .get(parent as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: parent as i64,
                limit: model.parts.len() as i64,
            })?;

        mat = [
            above.f32_at(0x04),
            above.f32_at(0x08),
            above.f32_at(0x0c),
            above.f32_at(0x10),
            above.f32_at(0x14),
            above.f32_at(0x18),
        ];

        let across = operation::idiv(
            part.i32_at(0x44)
                .wrapping_add(part.i32_at(0x3c))
                .wrapping_mul(above.i32_at(0x60)),
            model.scale_unit,
        )
        .ok_or(Fault::divide(SITE, model.scale_unit as i64))?;
        let down = operation::idiv(
            part.i32_at(0x48)
                .wrapping_add(part.i32_at(0x40))
                .wrapping_mul(above.i32_at(0x6c)),
            model.scale_unit,
        )
        .ok_or(Fault::divide(SITE, model.scale_unit as i64))?;

        matrix_translate(&mut mat, across, down);
    }

    let mut degrees = (part.i32_at(0x78).wrapping_add(part.i32_at(0x74)) as f32) * 360.0
        / (model.angle_unit as f32);

    if model.mirror != 0 {
        degrees = -degrees;
    }

    if part.u8_at(0x89) != part.u8_at(0x8b) {
        degrees = -degrees;
    }

    matrix_rotate(&mut mat, degrees);

    for (cell, value) in mat.iter().enumerate() {
        part.set_f32_at(0x04 + cell * 4, *value);
    }

    for corner in [0x90usize, 0x98, 0xa0, 0xa8] {
        let mut out = 0i64;

        transform_point(&mat, part.i32_at(corner), part.i32_at(corner + 4), &mut out);
        part.set_i32_at(corner, out as i32);
        part.set_i32_at(corner + 4, (out >> 0x20) as i32);
    }

    *model
        .parts
        .get_mut(part_index as i64 as usize)
        .ok_or(Fault::NullPointer { site: SITE })? = part;

    Ok(())
}
