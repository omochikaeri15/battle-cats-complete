use std::rc::Rc;

use super::{Mamodel, SheetTable};

pub fn mamodel_set_sheet_table(model: &mut Mamodel, table: &SheetTable) {
    model.sheet = None;
    model.sheet_table = Rc::clone(table);
}
