#![warn(unreachable_pub)]
mod vault;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod common;
pub mod domains;
pub mod systems;

pub use vault::{
    CatStore, Conflict, ContentStore, EnemyStore, Evicted, ItemStore, Listing, NameStore, Memory, Mount, Source, StageStore, Target, TreasureStore, Vault,
    Vds, Vfs, VfsError,
};
