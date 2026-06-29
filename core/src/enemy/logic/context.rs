use nyanko::enemy::unit::Battle;

use crate::enemy::registry::Magnification;
use crate::global::context::GlobalContext;

#[derive(Clone, Copy)]
pub struct EnemyRenderContext<'a> {
    pub global: GlobalContext<'a>,
    pub stats: &'a Battle,
    pub magnification: Magnification,
}