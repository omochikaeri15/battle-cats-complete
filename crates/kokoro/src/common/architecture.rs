#[cfg(feature = "disk")]
mod disk;

#[cfg(feature = "disk")]
pub use disk::{Scratch, Workspace, anchor, anchored, game_present, has_content, volatile, work_cleanup, work_purge};

pub const GAME: &str = "game";
pub const MODS: &str = "mods";
pub const STUDIO: &str = "studio";
pub const SANDBOX: &str = "sandbox";
pub const REPLAY: &str = "replay";
pub const WORK: &str = ".work";
