use super::{MamodelPart};

pub fn set_part_scale(part: &mut MamodelPart, x: i32, y: i32) {
    part.set_i32_at(MamodelPart::SCALE_X, x);
    part.set_i32_at(MamodelPart::SCALE_Y, y);
}
