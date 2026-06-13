use nyanko::common::Param;
use crate::global::game::localizable::Localizable;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub param: &'a Param,
    pub localizable: &'a Localizable,
}