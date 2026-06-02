// TODO: #[cfg(feature = "apk")] pub mod apk; [file "Bouncer" and apk structure]
#[cfg(feature = "pack")] pub mod pack;
// TODO: #[cfg(feature = "event")] pub mod event; [event data stuff: https://codeberg.org/fieryhenry/bcdd]
// TODO: #[cfg(feature = "unite")] pub mod unite; [decrypt "arc" nintendo files and handle formats such as btrx]
// TODO: #[cfg(feature = "bcu")] pub mod bcu; [decrypt bcuzip & bcuzips, convert bcu formats to game formats]
#[cfg(feature = "graphics")] pub mod graphics;
pub mod cat;
// TODO: pub mod enemy; [enemy struct, enemy data, enemy abilities]
// TODO: pub mod stage; [stage struct, stage data, CPU Skip/Treasure/Restrictions/Etc.]
pub mod common;

#[cfg(feature = "graphics")] pub use image;
pub use serde;