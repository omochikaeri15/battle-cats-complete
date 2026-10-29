use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropItemError {
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropItemEntry {
    pub map_id: u32,
    pub crown_multipliers: [f32; 4],
    pub stage_drops: [u32; 8],
    pub dud_chance: u32,
    pub material_drops: [u32; 16],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DropItem {
    pub map_drops: HashMap<u32, DropItemEntry>,
}

impl DropItem {
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