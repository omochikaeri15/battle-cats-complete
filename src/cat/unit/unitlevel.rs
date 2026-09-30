use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of level growth curves.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LevelError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid unit level data."),
        }
    }
}

impl std::error::Error for LevelError {}

/// The growth trajectory of an entity's statistics.
///
/// Holds the percentage scaling increments for each ten-level bracket, and the
/// arithmetic to project a statistic to any level.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LevelCurve {
    /// The sequence of scaling factors, where each index maps to a 10-level progression bracket.
    pub increments: Vec<u16>,
}

impl LevelCurve {
    fn from_csv_line(csv_line: &str, delimiter: char) -> Self {
        let increment_values = csv_line
            .split(delimiter)
            .filter_map(|part| part.trim().parse::<u16>().ok())
            .collect();

        Self { increments: increment_values }
    }

    /// Projects a base statistic forward to the value it holds at a given level.
    ///
    /// Growth is applied one level at a time, drawing the scaling factor for
    /// each step from the ten-level bracket that step falls into. Levels beyond
    /// the last declared bracket continue to accrue at the final bracket's rate.
    /// The result is then raised by the treasure effect that governs the
    /// statistic, the cat health effect for health and the cat attack effect for
    /// damage.
    ///
    /// # Arguments
    /// * `base_value` - The unscaled statistic as declared in the unit's combat row.
    /// * `target_level` - The one-based level to project the statistic to.
    /// * `treasure` - The treasure effect's value, from 0 with none held to 150 with every Empire of Cats chapter fully Superior.
    ///
    /// # Returns
    /// An `i32` containing the projected statistic at the requested level.
    pub fn calculate_stat(&self, base_value: i32, target_level: i32, treasure: i32) -> i32 {
        let base_float = base_value as f64;
        let mut current_stat = base_float;
        let max_scaled_level = (self.increments.len() * 10) as i32;
        let level_limit = std::cmp::min(target_level, max_scaled_level);

        for level_step in 2..=level_limit {
            let curve_index = ((level_step as f64 / 10.0).ceil() as usize).saturating_sub(1);
            if let Some(&scaling_factor) = self.increments.get(curve_index) {
                current_stat += base_float * (scaling_factor as f64) / 100.0;
            }
        }

        if target_level > max_scaled_level {
            let levels_above_limit = target_level - max_scaled_level;
            if let Some(&last_scaling_factor) = self.increments.last() {
                current_stat += base_float * (last_scaling_factor as f64) * (levels_above_limit as f64) / 100.0;
            }
        }

        let leveled = current_stat.round() as i64;

        (leveled + leveled * i64::from(treasure) / 100) as i32
    }

    /// Parses the level growth table into curves keyed by unit identifier.
    ///
    /// A line's position in the file is that unit's identifier, which blank
    /// lines do not disturb.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `unitlevel.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed curves keyed by unit identifier on
    /// success, or a `LevelError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<HashMap<u32, Self>, LevelError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<HashMap<u32, LevelCurve>, LevelError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut curves = HashMap::new();

    for (line_index, csv_line) in file_content.lines().enumerate() {
        if csv_line.trim().is_empty() { continue; }
        curves.insert(line_index as u32, LevelCurve::from_csv_line(csv_line, delimiter));
    }

    if curves.is_empty() {
        return Err(LevelError::EmptyFile);
    }

    Ok(curves)
}
