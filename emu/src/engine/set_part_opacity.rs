use super::MamodelPart;

pub fn set_part_opacity(part: &mut MamodelPart, opacity: i32) {
    part.set_i32_at(0x7c, opacity);
}
