use super::MamodelPart;

pub fn set_part_scale(part: &mut MamodelPart, x: i32, y: i32) {
    part.set_i32_at(0x5c, x);
    part.set_i32_at(0x68, y);
}
