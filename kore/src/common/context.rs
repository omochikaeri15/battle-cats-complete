use nyanko::files::Localizable;
use nyanko::files::Param;

use crate::systems::treasure::Bonus;
use crate::Vault;

#[derive(Clone, Copy)]
pub struct GlobalContext<'a> {
    pub param: &'a Param,
    pub localizable: &'a Localizable,
    pub vault: &'a Vault,
    pub treasure: &'a Bonus,
}