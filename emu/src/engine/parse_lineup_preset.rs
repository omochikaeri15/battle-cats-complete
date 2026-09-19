use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, FixedLineupUnit, JsonNode, get_unit_form_count, get_unit_guide_order,
    get_unit_max_level, get_unit_max_plus_level, is_unit_available, json_container_as_bool,
    json_container_as_int, json_string_as_bool, json_string_as_int, json_value_as_bool,
    json_value_as_int, string_to_int,
};

const SITE: &str = "parse_lineup_preset";

pub fn parse_lineup_preset(ctx: &mut AppContext, root: Option<&JsonNode>) -> Result<bool, Fault> {
    let Some(root) = root else {
        return Ok(false);
    };
    let JsonNode::Object(root) = root else {
        return Err(Fault::NullPointer { site: SITE });
    };
    let slot = match root.get(b"slot".as_slice()) {
        Some(JsonNode::Object(slot)) => Some(slot),
        _ => None,
    };
    let mut unit_index: BTreeMap<i32, i32> = BTreeMap::new();

    if let Some(slot) = slot
        && let Some(JsonNode::Object(_)) = slot.get(b"data".as_slice())
    {
        let Some(JsonNode::Object(data)) = slot.get(b"data".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        if let Some((_, entry)) = data.iter().next() {
            let JsonNode::Object(entry) = entry else {
                return Err(Fault::NullPointer { site: SITE });
            };
            let mut index = 0usize;

            loop {
                let Some(JsonNode::Array(chara)) = entry.get(b"chara".as_slice()) else {
                    return Err(Fault::NullPointer { site: SITE });
                };

                if index >= chara.len() {
                    break;
                }

                ctx.fixed_lineup_store
                    .units
                    .push(FixedLineupUnit::default());

                let node = &chara[index];
                let unit_id = match node {
                    JsonNode::String(text) => json_string_as_int(text)?,
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                    _ => json_value_as_int(node),
                } as i32;
                let unit = ctx
                    .fixed_lineup_store
                    .units
                    .last_mut()
                    .ok_or(Fault::NullPointer { site: SITE })?;

                unit.unit_id = unit_id;
                unit_index.insert(unit_id, index as i32);
                index += 1;
            }

            match entry.get(b"cannon".as_slice()) {
                Some(node) => {
                    let cannon = match node {
                        JsonNode::String(text) => json_string_as_int(text)?,
                        JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                        _ => json_value_as_int(node),
                    } as i32;

                    ctx.set_i32_at(AppContext::LINEUP_CANNON_TYPE, cannon)?;
                }
                None => ctx.set_i32_at(AppContext::LINEUP_CANNON_TYPE, -1)?,
            }
        }
    }

    if let Some(JsonNode::Object(chara)) = root.get(b"chara".as_slice())
        && let Some(JsonNode::Object(_)) = chara.get(b"data".as_slice())
    {
        let units: Vec<i32> = unit_index.keys().copied().collect();

        for unit_id in units {
            let Some(JsonNode::Object(data)) = chara.get(b"data".as_slice()) else {
                return Err(Fault::NullPointer { site: SITE });
            };
            let Some(JsonNode::Object(entry)) = data.get(unit_id.to_string().as_bytes()) else {
                return Ok(false);
            };

            if unit_id as u32 > 0x36b
                || !is_unit_available(ctx, unit_id)?
                || get_unit_guide_order(ctx, unit_id)? == -1
            {
                return Ok(false);
            }

            if let Some(node) = entry.get(b"evolution".as_slice()) {
                let evolution = match node {
                    JsonNode::String(text) => json_string_as_int(text)?,
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                    _ => json_value_as_int(node),
                } as i32;

                if evolution <= 0 || evolution > get_unit_form_count(ctx, unit_id)? {
                    return Ok(false);
                }

                let index = *unit_index.entry(unit_id).or_insert(0);

                ctx.fixed_lineup_store
                    .units
                    .get_mut(index as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: index as i64,
                        limit: 0,
                    })?
                    .form = evolution.wrapping_sub(1);
            }

            if let Some(node) = entry.get(b"level".as_slice()) {
                let level = match node {
                    JsonNode::String(text) => json_string_as_int(text)?,
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                    _ => json_value_as_int(node),
                } as i32;

                if level <= 0 || level > get_unit_max_level(ctx, unit_id)? {
                    return Ok(false);
                }

                let index = *unit_index.entry(unit_id).or_insert(0);

                ctx.fixed_lineup_store
                    .units
                    .get_mut(index as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: index as i64,
                        limit: 0,
                    })?
                    .level = level.wrapping_sub(1);
            }

            if let Some(node) = entry.get(b"plus".as_slice()) {
                let plus = match node {
                    JsonNode::String(text) => json_string_as_int(text)?,
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                    _ => json_value_as_int(node),
                } as i32;

                if plus < 0 || plus > get_unit_max_plus_level(ctx, unit_id)? {
                    return Ok(false);
                }

                let index = *unit_index.entry(unit_id).or_insert(0);

                ctx.fixed_lineup_store
                    .units
                    .get_mut(index as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: index as i64,
                        limit: 0,
                    })?
                    .plus_level = plus;
            }
        }
    }

    if let Some(JsonNode::Object(ability)) = root.get(b"ability".as_slice()) {
        let Some(JsonNode::Object(data)) = ability.get(b"data".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        for (key, value) in data {
            let kind = string_to_int(key)?;

            if kind as u32 > 9 {
                return Ok(false);
            }

            let JsonNode::Object(value) = value else {
                return Err(Fault::NullPointer { site: SITE });
            };
            let level = value
                .get(b"level".as_slice())
                .map_or(Ok(0), |node| match node {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(node)),
                })? as i32;
            let plus = value
                .get(b"plus".as_slice())
                .map_or(Ok(0), |node| match node {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(node)),
                })? as i32;

            if level <= 0 || plus < 0 {
                return Ok(false);
            }

            let row = (kind as usize)
                .wrapping_mul(0x14)
                .wrapping_add(AppContext::TECH_MAX_LEVELS);

            if level > ctx.i32_at(row)? || plus > ctx.i32_at(row + 4)? {
                return Ok(false);
            }

            ctx.fixed_lineup_store
                .ability_levels
                .insert(kind, level.wrapping_add(plus).wrapping_sub(1));
        }
    }

    if let Some(JsonNode::Object(cannon)) = root.get(b"cannon".as_slice()) {
        let Some(JsonNode::Object(data)) = cannon.get(b"data".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        for (key, value) in data {
            let part = string_to_int(key)?;
            let JsonNode::Object(value) = value else {
                return Err(Fault::NullPointer { site: SITE });
            };
            let level = (value
                .get(b"level".as_slice())
                .map_or(Ok(0), |node| match node {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(node)),
                })? as i32)
                .wrapping_sub(1);

            if part == 0 {
                ctx.set_i32_at(AppContext::LINEUP_BASE_LEVEL, level)?;
                ctx.set_i32_at(AppContext::LINEUP_CANNON_LEVEL, level)?;
            } else {
                let cannon_type = ctx.i32_at(AppContext::LINEUP_CANNON_TYPE)?;

                if cannon_type != -1 && part == cannon_type {
                    ctx.set_i32_at(AppContext::LINEUP_CANNON_LEVEL, level)?;
                }
            }
        }
    }

    let Some(JsonNode::Object(treasure)) = root.get(b"treasure".as_slice()) else {
        return Ok(true);
    };

    if let Some(JsonNode::Object(_)) = treasure.get(b"defaultData".as_slice()) {
        let Some(JsonNode::Object(default)) = treasure.get(b"defaultData".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };
        let none = default
            .get(b"none".as_slice())
            .is_some_and(|node| match node {
                JsonNode::String(text) => json_string_as_bool(text),
                JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_bool(),
                _ => json_value_as_bool(node),
            });

        if none {
            for chapter in 0..10 {
                ctx.fixed_lineup_store.treasure_flags.insert(chapter, false);
            }
        }
    }

    let Some(JsonNode::Object(data)) = treasure.get(b"data".as_slice()) else {
        return Ok(true);
    };

    for (key, value) in data {
        let chapter = string_to_int(key)?;

        if chapter as u32 > 9 || chapter == 3 {
            return Ok(false);
        }

        let JsonNode::Object(value) = value else {
            return Err(Fault::NullPointer { site: SITE });
        };
        let Some(JsonNode::Array(count)) = value.get(b"count".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };
        let full = count.len() >= 3
            && match &count[2] {
                JsonNode::String(text) => json_string_as_int(text)?,
                JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                node => json_value_as_int(node),
            } == 0x30;

        ctx.fixed_lineup_store.treasure_flags.insert(chapter, full);
    }

    Ok(true)
}
