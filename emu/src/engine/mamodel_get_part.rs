use super::Mamodel;

pub fn mamodel_get_part(model: &Mamodel, index: i32) -> Option<usize> {
    if index < 0 || model.parts.len() <= index as usize {
        return None;
    }

    Some(index as usize)
}
