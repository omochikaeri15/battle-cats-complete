use nyanko::common::Param;

use super::game::localizable::Localizable;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub param: &'a Param,
    pub localizable: &'a Localizable,
}