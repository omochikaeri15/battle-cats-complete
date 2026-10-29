use crate::Fault;

use super::{Mamodel, MamodelPart, transform_point};

pub fn transform_anchor(
    part: &MamodelPart,
    model: &Mamodel,
    x: i32,
    y: i32,
    out: &mut i64,
) -> Result<(), Fault> {
    let scale_unit = model.scale_unit;

    let spread_x = x
        .wrapping_sub(part.i32_at(MamodelPart::PIVOT_X).wrapping_add(part.i32_at(MamodelPart::PIVOT_X_ANIM)))
        .wrapping_mul(part.i32_at(MamodelPart::WORLD_SCALE_X)) as i64;

    let spread_y = y
        .wrapping_sub(part.i32_at(MamodelPart::PIVOT_Y).wrapping_add(part.i32_at(MamodelPart::PIVOT_Y_ANIM)))
        .wrapping_mul(part.i32_at(MamodelPart::WORLD_SCALE_Y)) as i64;

    if scale_unit == 0 {
        return Err(Fault::divide_by_zero());
    }

    let scaled_x = spread_x / scale_unit as i64;
    let scaled_y = spread_y / scale_unit as i64;

    if scaled_x != scaled_x as i32 as i64 || scaled_y != scaled_y as i32 as i64 {
        return Err(Fault::divide_overflow());
    }

    let mat = [
        part.f32_at(MamodelPart::MATRIX_0),
        part.f32_at(MamodelPart::MATRIX_1),
        part.f32_at(MamodelPart::MATRIX_2),
        part.f32_at(MamodelPart::MATRIX_3),
        part.f32_at(MamodelPart::MATRIX_4),
        part.f32_at(MamodelPart::MATRIX_5),
    ];

    transform_point(&mat, scaled_x as i32, scaled_y as i32, out);

    Ok(())
}
