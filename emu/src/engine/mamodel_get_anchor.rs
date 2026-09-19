use crate::Fault;

use super::{Mamodel, MamodelAnchor};

pub fn mamodel_get_anchor(model: &Mamodel, idx: i32) -> Result<&MamodelAnchor, Fault> {
    model
        .anchors
        .get(idx as i64 as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "mamodel_get_anchor",
            index: idx as i64,
            limit: model.anchors.len() as i64,
        })
}
