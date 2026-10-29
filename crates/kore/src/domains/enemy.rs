pub mod animation;
pub mod filter;
pub mod files;
pub(crate) mod patterns;
pub mod scanner;
pub mod statblock;

use crate::domains::enemy::scanner::EnemyEntry;

#[derive(Default)]
pub struct EnemyDataState {
    pub enemies: Vec<EnemyEntry>,
}
