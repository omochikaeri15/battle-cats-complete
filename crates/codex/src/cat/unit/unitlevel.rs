use std::collections::HashMap;
use std::fmt;

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LevelError {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LevelCurve {
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
