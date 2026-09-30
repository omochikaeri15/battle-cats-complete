//! Structured access to the raw game files of *The Battle Cats*.
//!
//! `nyanko` converts the engine's extracted data files into typed Rust
//! structures, and implements the game-domain logic needed to make sense of
//! them: pack decryption, animation solving, and the reconciliation of values
//! that the engine scatters across several files.
//!
//! # Stateless by design
//!
//! The crate performs no filesystem, network, or other OS-level access. Every
//! entry point that needs file data takes that data as in-memory bytes, and the
//! caller is responsible for obtaining it. This keeps the crate usable from a
//! WebAssembly target, where such access is unavailable.
//!
//! # Layout
//!
//! - [`cat`] and [`enemy`] expose the aggregate unit structures and the parsers
//!   behind them. Start at [`cat::Unit::assemble`] and [`enemy::Unit::assemble`],
//!   which perform the cross-table lookups and sentinel decisions the engine's
//!   layout requires.
//! - [`combat`] holds the shared statistic structure both factions parse into,
//!   plus the registry that recognizes which abilities an entity carries.
//! - [`chapter`] covers the level content: chapters, maps, and stages.
//! - [`files`] holds the game-wide tables no single domain owns: the tuning
//!   constants, the string dictionary, and the item catalogue.
//! - [`common`] holds the text handling, column tables, and regional conventions
//!   the other modules share.
//! - [`graphics`] parses animation rigs and resolves them into renderer-ready
//!   geometry. Requires the `graphics` feature.
//! - [`pack`] and [`bcu`] decrypt the archive formats the game ships its assets
//!   in. Require the `pack` and `bcu` features respectively.
//!
//! # Errors
//!
//! Each parser returns its own error type so that a caller handling one file
//! need not match against unrelated failures. [`Error`] converts from all of
//! them, so code spanning several parsers can propagate any of them through a
//! single signature with the `?` operator.
//!
//! # Features
//!
//! All features are off by default, so a default build depends only on `serde`
//! and `serde_json`.
//!
//! - `graphics` enables [`graphics`], pulling in `image` for texture decoding.
//! - `pack` enables [`pack`], pulling in the AES and hashing crates.
//! - `bcu` enables [`bcu`], pulling in a subset of the same cryptography crates.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

// TODO: #[cfg(feature = "apk")] pub mod apk; [file "Bouncer" and apk structure]
#[cfg(feature = "pack")] #[cfg_attr(docsrs, doc(cfg(feature = "pack")))] pub mod pack;
// TODO: #[cfg(feature = "event")] pub mod event; [event data stuff like bcdd does]
// TODO: #[cfg(feature = "unite")] pub mod unite; [decrypt "arc" nintendo files and handle formats such as btrx]
#[cfg(feature = "bcu")] #[cfg_attr(docsrs, doc(cfg(feature = "bcu")))] pub mod bcu;
#[cfg(feature = "graphics")] #[cfg_attr(docsrs, doc(cfg(feature = "graphics")))] pub mod graphics;
mod error;

pub mod cat;
pub mod combat;
pub mod enemy;
pub mod chapter;
pub mod common;
pub mod files;

pub use error::Error;

#[cfg(feature = "graphics")] pub use image;
pub use serde;
pub use serde_json;
