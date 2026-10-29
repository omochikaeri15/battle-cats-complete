use super::Mamodel;

pub fn mamodel_get_part_count(model: &Mamodel) -> i32 {
    model.parts.len() as i32
}
