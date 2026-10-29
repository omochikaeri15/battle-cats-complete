//! The battle background table, keyed by background identifier.
//!
//! Each row names the four colors the engine fills the sky and ground with, the
//! sprite sheet the ground tile is cut from, whether a second tile hangs above
//! the sky, which image the sheet is read from, and an optional fog gradient
//! laid over the whole scene.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

/// The sentinel an image column carries when the background draws its own image.
const OWN_IMAGE: i32 = -1;

/// Represents errors that can occur during the parsing of the background table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackgroundError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for BackgroundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid background data."
            ),
        }
    }
}

impl std::error::Error for BackgroundError {}

/// One background's colors, sheet and fog, as one row of the table declares them.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackgroundEntry {
    /// The identifier a stage names the background by.
    pub id: u32,
    /// The red channel of the color at the top of the sky.
    pub sky_top_red: i32,
    /// The green channel of the color at the top of the sky.
    pub sky_top_green: i32,
    /// The blue channel of the color at the top of the sky.
    pub sky_top_blue: i32,
    /// The red channel of the color at the horizon.
    pub sky_bottom_red: i32,
    /// The green channel of the color at the horizon.
    pub sky_bottom_green: i32,
    /// The blue channel of the color at the horizon.
    pub sky_bottom_blue: i32,
    /// The red channel of the color just below the ground tile.
    pub ground_top_red: i32,
    /// The green channel of the color just below the ground tile.
    pub ground_top_green: i32,
    /// The blue channel of the color just below the ground tile.
    pub ground_top_blue: i32,
    /// The red channel of the color at the bottom of the ground.
    pub ground_bottom_red: i32,
    /// The green channel of the color at the bottom of the ground.
    pub ground_bottom_green: i32,
    /// The blue channel of the color at the bottom of the ground.
    pub ground_bottom_blue: i32,
    /// The number of the sprite cut list the tiles are read from.
    pub model_id: i32,
    /// Whether a second tile is drawn above the sky.
    pub has_upper_layer: bool,
    /// The number of the image the sheet is read from, or minus one when the background draws its own.
    pub image_id: Option<i32>,
    /// The red channel of the fog at the top of the scene, when the row declares a fog.
    pub fog_top_red: Option<i32>,
    /// The green channel of the fog at the top of the scene, when the row declares a fog.
    pub fog_top_green: Option<i32>,
    /// The blue channel of the fog at the top of the scene, when the row declares a fog.
    pub fog_top_blue: Option<i32>,
    /// The alpha channel of the fog at the top of the scene, when the row declares a fog.
    pub fog_top_alpha: Option<i32>,
    /// The red channel of the fog at the bottom of the scene, when the row declares one.
    pub fog_bottom_red: Option<i32>,
    /// The green channel of the fog at the bottom of the scene, when the row declares one.
    pub fog_bottom_green: Option<i32>,
    /// The blue channel of the fog at the bottom of the scene, when the row declares one.
    pub fog_bottom_blue: Option<i32>,
    /// The alpha channel of the fog at the bottom of the scene, when the row declares one.
    pub fog_bottom_alpha: Option<i32>,
}

/// A color the engine packs from a row, as it composes it into a vertex.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba {
    /// The red channel, from zero to two hundred and fifty-five.
    pub red: i32,
    /// The green channel, from zero to two hundred and fifty-five.
    pub green: i32,
    /// The blue channel, from zero to two hundred and fifty-five.
    pub blue: i32,
    /// The alpha channel, from zero to two hundred and fifty-five.
    pub alpha: i32,
}

impl Rgba {
    /// Builds an opaque color from three channels.
    ///
    /// # Arguments
    /// * `red` - The red channel.
    /// * `green` - The green channel.
    /// * `blue` - The blue channel.
    ///
    /// # Returns
    /// An `Rgba` with its alpha channel at two hundred and fifty-five.
    pub const fn opaque(red: i32, green: i32, blue: i32) -> Self {
        Self { red, green, blue, alpha: 255 }
    }

    /// Builds a color from four channels.
    ///
    /// # Arguments
    /// * `red` - The red channel.
    /// * `green` - The green channel.
    /// * `blue` - The blue channel.
    /// * `alpha` - The alpha channel.
    ///
    /// # Returns
    /// An `Rgba` holding the four channels as given.
    pub const fn new(red: i32, green: i32, blue: i32, alpha: i32) -> Self {
        Self { red, green, blue, alpha }
    }

    /// Returns the color with its alpha channel replaced.
    ///
    /// # Arguments
    /// * `alpha` - The alpha channel to carry instead.
    ///
    /// # Returns
    /// An `Rgba` with the same three color channels and the given alpha.
    pub const fn with_alpha(self, alpha: i32) -> Self {
        Self { alpha, ..self }
    }
}

impl BackgroundEntry {
    /// The column mapping this parser applies to one row, in the order it
    /// applies it.
    ///
    /// Published so a consumer can read the layout of the row from the parser's
    /// own table instead of restating it. The image and fog columns are read as
    /// optional, so a row that stops short of them, or whose cell names no
    /// integer, holds `None` where the engine would treat the column as absent.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        id                  : 0;
        sky_top_red         : 1;
        sky_top_green       : 2;
        sky_top_blue        : 3;
        sky_bottom_red      : 4;
        sky_bottom_green    : 5;
        sky_bottom_blue     : 6;
        ground_top_red      : 7;
        ground_top_green    : 8;
        ground_top_blue     : 9;
        ground_bottom_red   : 10;
        ground_bottom_green : 11;
        ground_bottom_blue  : 12;
        model_id            : 13;
        has_upper_layer     : 14;
        image_id            : 15, Raw, "";
        fog_top_red         : 16, Raw, "";
        fog_top_green       : 17, Raw, "";
        fog_top_blue        : 18, Raw, "";
        fog_top_alpha       : 19, Raw, "";
        fog_bottom_red      : 20, Raw, "";
        fog_bottom_green    : 21, Raw, "";
        fog_bottom_blue     : 22, Raw, "";
        fog_bottom_alpha    : 23, Raw, "";
    };

    /// Returns the number of the image the sheet is read from.
    ///
    /// # Returns
    /// A `u32` holding the image column when it names another image, and
    /// otherwise the background's own identifier.
    pub fn image(&self) -> u32 {
        match self.image_id {
            Some(image) if image != OWN_IMAGE => image.max(0) as u32,
            _ => self.id,
        }
    }

    /// Returns the color at the top of the sky.
    ///
    /// # Returns
    /// An opaque `Rgba`.
    pub fn sky_top(&self) -> Rgba {
        Rgba::opaque(self.sky_top_red, self.sky_top_green, self.sky_top_blue)
    }

    /// Returns the color at the horizon.
    ///
    /// # Returns
    /// An opaque `Rgba`.
    pub fn sky_bottom(&self) -> Rgba {
        Rgba::opaque(self.sky_bottom_red, self.sky_bottom_green, self.sky_bottom_blue)
    }

    /// Returns the color just below the ground tile.
    ///
    /// # Returns
    /// An opaque `Rgba`.
    pub fn ground_top(&self) -> Rgba {
        Rgba::opaque(self.ground_top_red, self.ground_top_green, self.ground_top_blue)
    }

    /// Returns the color at the bottom of the ground.
    ///
    /// # Returns
    /// An opaque `Rgba`.
    pub fn ground_bottom(&self) -> Rgba {
        Rgba::opaque(self.ground_bottom_red, self.ground_bottom_green, self.ground_bottom_blue)
    }

    /// Returns the fog laid over the top of the scene.
    ///
    /// The engine reads the fog only when the row reaches its alpha column with
    /// an integer, and reads the three color channels as zero where they name
    /// none.
    ///
    /// # Returns
    /// An `Option` containing the fog color, or `None` when the row declares no fog.
    pub fn fog_top(&self) -> Option<Rgba> {
        let alpha = self.fog_top_alpha?;

        Some(Rgba::new(
            self.fog_top_red.unwrap_or(0),
            self.fog_top_green.unwrap_or(0),
            self.fog_top_blue.unwrap_or(0),
            alpha,
        ))
    }

    /// Returns the fog laid over the bottom of the scene.
    ///
    /// A row that declares a fog without a second color repeats the first, so
    /// the fog is flat.
    ///
    /// # Returns
    /// An `Option` containing the fog color, or `None` when the row declares no fog.
    pub fn fog_bottom(&self) -> Option<Rgba> {
        let top = self.fog_top()?;
        let Some(alpha) = self.fog_bottom_alpha else {
            return Some(top);
        };

        Some(Rgba::new(
            self.fog_bottom_red.unwrap_or(0),
            self.fog_bottom_green.unwrap_or(0),
            self.fog_bottom_blue.unwrap_or(0),
            alpha,
        ))
    }
}

/// The parsed contents of the battle background table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Background {
    /// The rows in the order the file declares them.
    pub entries: Vec<BackgroundEntry>,
}

impl Background {
    /// Parses the background table.
    ///
    /// The first line is a header and is skipped unread. A row's leading cell
    /// must name an integer for the row to count.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the `bg.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Background` on success, or a
    /// `BackgroundError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, BackgroundError> {
        parse_inner(bytes.as_ref(), separator)
    }

    /// Finds the row declaring a background.
    ///
    /// # Arguments
    /// * `id` - The background identifier a stage names.
    ///
    /// # Returns
    /// An `Option` containing the first row carrying the identifier, or `None` when no row does.
    pub fn get(&self, id: u32) -> Option<&BackgroundEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Background, BackgroundError> {
    let content = file::scrub(bytes);
    let separator = file::resolve(separator, &content);

    let entries: Vec<BackgroundEntry> = content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let row: Vec<&str> = line.split(separator).collect();

            row.first()?.trim().parse::<u32>().ok()?;

            let mut entry = BackgroundEntry::default();

            columns::apply(&row, BackgroundEntry::COLUMNS, &mut entry);

            Some(entry)
        })
        .collect();

    if entries.is_empty() {
        return Err(BackgroundError::EmptyFile);
    }

    Ok(Background { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(BackgroundEntry::COLUMNS);
    }

    #[test]
    fn a_short_row_declares_no_fog_and_draws_its_own_image() {
        let table = Background::parse(
            "id,sky\n3,240,251,255,98,207,255,135,98,59,65,36,6,0,0\n",
            Some(Separator::Comma),
        )
        .expect("the row parses");
        let entry = table.get(3).expect("the row is listed");

        assert_eq!(entry.image(), 3);
        assert_eq!(entry.model_id, 0);
        assert!(!entry.has_upper_layer);
        assert_eq!(entry.fog_top(), None);
        assert_eq!(entry.sky_top(), Rgba::opaque(240, 251, 255));
    }

    #[test]
    fn a_fogged_row_repeats_its_top_color_when_it_names_no_bottom() {
        let table = Background::parse(
            "id\n7,1,2,3,4,5,6,7,8,9,10,11,12,2,1,-1,20,30,40,128\n",
            Some(Separator::Comma),
        )
        .expect("the row parses");
        let entry = table.get(7).expect("the row is listed");

        assert_eq!(entry.image(), 7);
        assert_eq!(entry.fog_top(), Some(Rgba::new(20, 30, 40, 128)));
        assert_eq!(entry.fog_bottom(), entry.fog_top());
    }

    #[test]
    fn an_image_column_redirects_the_sheet() {
        let table = Background::parse(
            "id\n250,1,2,3,4,5,6,7,8,9,10,11,12,5,0,40\n",
            Some(Separator::Comma),
        )
        .expect("the row parses");

        assert_eq!(table.get(250).map(BackgroundEntry::image), Some(40));
    }
}
