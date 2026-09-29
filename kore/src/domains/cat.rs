pub mod animation;
pub mod combo;
pub mod filter;
pub mod game;
pub mod scanner;
pub mod statblock;
pub mod files;
pub(crate) mod patterns;
pub mod waiter;

use crate::domains::cat::scanner::CatEntry;

#[derive(Default)]
pub struct CatDataState {
    pub cats: Vec<CatEntry>,
}