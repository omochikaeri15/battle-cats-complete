use crate::Fault;

use super::{Mamodel, MamodelPart, transform_point};

const SITE: &str = "transform_anchor";

pub fn transform_anchor(
    part: &MamodelPart,
    model: &Mamodel,
    x: i32,
    y: i32,
    out: &mut i64,
) -> Result<(), Fault> {
    let scale_unit = model.scale_unit;

    let spread_x = x
        .wrapping_sub(part.i32_at(0x4c).wrapping_add(part.i32_at(0x54)))
        .wrapping_mul(part.i32_at(0x60)) as i64;

    let spread_y = y
        .wrapping_sub(part.i32_at(0x50).wrapping_add(part.i32_at(0x58)))
        .wrapping_mul(part.i32_at(0x6c)) as i64;

    if scale_unit == 0 {
        return Err(Fault::DivideByZero { site: SITE });
    }

    let scaled_x = spread_x / scale_unit as i64;
    let scaled_y = spread_y / scale_unit as i64;

    if scaled_x != scaled_x as i32 as i64 || scaled_y != scaled_y as i32 as i64 {
        return Err(Fault::DivideOverflow { site: SITE });
    }

    let mat = [
        part.f32_at(0x04),
        part.f32_at(0x08),
        part.f32_at(0x0c),
        part.f32_at(0x10),
        part.f32_at(0x14),
        part.f32_at(0x18),
    ];

    transform_point(&mat, scaled_x as i32, scaled_y as i32, out);

    Ok(())
}
