//! Reward drop configuration for map clears.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur during the parsing of map drop tables.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropItemError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for DropItemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid drop item data."
            ),
        }
    }
}

impl std::error::Error for DropItemError {}

/// The reward drop configuration for a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropItemEntry {
    /// The identifier of the map this configuration applies to.
    pub map_id: u32,
    /// The reward scaling applied at each of the four crown difficulties.
    pub crown_multipliers: [f32; 4],
    /// The item identifiers awarded for clearing individual stages.
    pub stage_drops: [u32; 8],
    /// The percentage chance that a drop attempt yields nothing.
    pub dud_chance: u32,
    /// The item identifiers awarded from the map's material reward pool.
    pub material_drops: [u32; 16],
}

/// The parsed contents of the map drop table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DropItem {
    /// The drop configurations, keyed by map identifier.
    pub map_drops: HashMap<u32, DropItemEntry>,
}

impl DropItem {
    /// Parses the map drop table into per-map reward configurations.
    ///
    /// Rows are addressed by the map identifier in their leading column rather
    /// than by position. Trailing comment text introduced by a double slash is
    /// discarded before the columns are read.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the drop item file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `DropItem` on success, or a
    /// `DropItemError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, DropItemError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn extract_f32_array<const SIZE: usize>(parts: &[&str], start_index: usize) -> Option<[f32; SIZE]> {
    let mut result = [0.0; SIZE];
    for (offset, slot) in result.iter_mut().enumerate() {
        let string_part = parts.get(start_index + offset)?;
        *slot = string_part.trim().parse().ok()?;
    }
    Some(result)
}

fn extract_u32_array<const SIZE: usize>(parts: &[&str], start_index: usize) -> Option<[u32; SIZE]> {
    let mut result = [0; SIZE];
    for (offset, slot) in result.iter_mut().enumerate() {
        let string_part = parts.get(start_index + offset)?;
        *slot = string_part.trim().parse().ok()?;
    }
    Some(result)
}

fn extract_u32_array_optional<const SIZE: usize>(parts: &[&str], start_index: usize) -> [u32; SIZE] {
    let mut result = [0; SIZE];
    for (offset, slot) in result.iter_mut().enumerate() {
        if let Some(string_part) = parts.get(start_index + offset)
            && let Ok(parsed_value) = string_part.trim().parse::<u32>() {
                *slot = parsed_value;
            }
    }
    result
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<DropItem, DropItemError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut map_drops = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines().skip(1) {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 22 {
            continue;
        }

        let Some(map_id_string) = parts.first() else { continue; };
        let Ok(map_id) = map_id_string.trim().parse::<u32>() else { continue; };

        let Some(crown_multipliers) = extract_f32_array::<4>(&parts, 1) else { continue; };
        let Some(stage_drops) = extract_u32_array::<8>(&parts, 5) else { continue; };

        let Some(dud_chance_string) = parts.get(13) else { continue; };
        let Ok(dud_chance) = dud_chance_string.trim().parse::<u32>() else { continue; };

        let Some(base_materials) = extract_u32_array::<8>(&parts, 14) else { continue; };
        let z_materials = extract_u32_array_optional::<8>(&parts, 22);

        let mut material_drops = [0; 16];
        material_drops[..8].copy_from_slice(&base_materials);
        material_drops[8..].copy_from_slice(&z_materials);

        map_drops.insert(
            map_id,
            DropItemEntry {
                map_id,
                crown_multipliers,
                stage_drops,
                dud_chance,
                material_drops,
            },
        );
    }

    if !has_content {
        return Err(DropItemError::EmptyFile);
    }

    Ok(DropItem { map_drops })
}