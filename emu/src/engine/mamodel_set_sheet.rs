use std::rc::Rc;

use super::{Imgcut, Mamodel};

pub fn mamodel_set_sheet(model: &mut Mamodel, sheet: Option<Rc<Imgcut>>) {
    model.sheet = sheet;
}
