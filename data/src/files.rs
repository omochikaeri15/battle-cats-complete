//! Public facade for the game-wide files no single domain owns.
//!
//! A unit file belongs to a unit and a stage file to a stage, but the engine
//! also ships tables that address the whole game: the tuning constants in
//! [`Param`], the string dictionary in [`Localizable`], and the item catalogue
//! in [`GatyaItemBuy`] with its localized text in [`GatyaItemName`]. This module
//! re-exports each of those and every error type behind them, so callers never
//! reach into the internal submodule layout.
//!
//! [`img015`] and [`img022`] name the atlas positions the engine addresses its
//! icons by, which the same tables refer to.

mod gatyaitembuy;
mod gatyaitemname;
mod localizable;
mod param;

pub mod img015;
pub mod img022;

pub use crate::common::Separator;
pub use gatyaitembuy::{GatyaItemBuy, GatyaItemBuyError};
pub use gatyaitemname::{GatyaItemName, GatyaItemNameError};
pub use localizable::{Localizable, LocalizableError};
pub use param::{Param, ParamError};
