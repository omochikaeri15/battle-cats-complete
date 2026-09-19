use super::MamodelPart;

pub fn set_part_angle(part: &mut MamodelPart, angle: i32) {
    part.set_i32_at(0x74, angle);
}
