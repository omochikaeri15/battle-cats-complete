use super::Mamodel;

pub fn mamodel_set_mirror(model: &mut Mamodel, mirror: u8) {
    model.mirror = mirror as i32;
}
