use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int, json_string_as_string,
    json_value_as_int, json_value_as_string, open_asset_stream, string_to_int,
};

pub fn load_special_rules_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.special_rules.maps.clear();
    ctx.special_rules.invalid_nyancombo_ids.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"SpecialRulesMap.json", 0, 0)? {
        let source = json_source_from_string(&bytes);
        let document = json_parse_object_document(Some(source))?;
        let Some(JsonNode::Object(root)) = document.as_ref() else {
            return Err(Fault::null_pointer());
        };

        if let Some(JsonNode::Object(entries)) = root.get(b"MapID".as_slice()) {
            for (key, entry) in entries {
                let map_id = string_to_int(key)?;
                let JsonNode::Object(map_entry) = entry else {
                    return Err(Fault::null_pointer());
                };
                let Some(JsonNode::Object(rule_types)) = map_entry.get(b"RuleType".as_slice())
                else {
                    continue;
                };

                for (rule_key, rule_entry) in rule_types {
                    let rule = string_to_int(rule_key)?;
                    let JsonNode::Object(rule_entry) = rule_entry else {
                        return Err(Fault::null_pointer());
                    };
                    let Some(JsonNode::Array(params)) =
                        rule_entry.get(b"Parameters".as_slice())
                    else {
                        continue;
                    };

                    if rule as u32 <= 0xc && 0x6cu32 >> (rule as u32) & 1 != 0 {
                        if params.len() * 8 != 0x30 || params.is_empty() {
                            continue;
                        }
                    } else if rule == 0xc {
                        let mut slot = 0usize;

                        while slot != 3 {
                            let values = ctx
                                .special_rules
                                .maps
                                .entry(map_id)
                                .or_default()
                                .normal
                                .entry(rule)
                                .or_default();
                            let element =
                                params.get(slot).ok_or(Fault::index_out_of_range(slot as i64, params.len() as i64))?;

                            let value = match element {
                                JsonNode::String(text) => json_string_as_int(text),
                                JsonNode::Array(_) | JsonNode::Object(_) => {
                                    Ok(json_container_as_int())
                                }
                                _ => Ok(json_value_as_int(element)),
                            }? as i32;

                            values.push(value);
                            slot += 1;
                        }

                        let element = params.get(3).ok_or(Fault::index_out_of_range(3, params.len() as i64))?;
                        let fever = match element {
                            JsonNode::String(text) => json_string_as_int(text),
                            JsonNode::Array(_) | JsonNode::Object(_) => {
                                Ok(json_container_as_int())
                            }
                            _ => Ok(json_value_as_int(element)),
                        }? as i32;

                        if params.len() * 8 < 0x21 {
                            continue;
                        }

                        let mut slot = 4usize;

                        loop {
                            let values = ctx
                                .special_rules
                                .maps
                                .entry(map_id)
                                .or_default()
                                .fever
                                .entry(fever)
                                .or_default();
                            let element =
                                params.get(slot).ok_or(Fault::index_out_of_range(slot as i64, params.len() as i64))?;

                            let value = match element {
                                JsonNode::String(text) => json_string_as_int(text),
                                JsonNode::Array(_) | JsonNode::Object(_) => {
                                    Ok(json_container_as_int())
                                }
                                _ => Ok(json_value_as_int(element)),
                            }? as i32;

                            values.push(value);
                            slot += 1;

                            if params.len() <= slot {
                                break;
                            }
                        }

                        continue;
                    } else if params.is_empty() {
                        continue;
                    }

                    let mut slot = 0usize;

                    while slot < params.len() {
                        let values = ctx
                            .special_rules
                            .maps
                            .entry(map_id)
                            .or_default()
                            .normal
                            .entry(rule)
                            .or_default();
                        let element = params.get(slot).ok_or(Fault::index_out_of_range(slot as i64, params.len() as i64))?;

                        let value = match element {
                            JsonNode::String(text) => json_string_as_int(text),
                            JsonNode::Array(_) | JsonNode::Object(_) => {
                                Ok(json_container_as_int())
                            }
                            _ => Ok(json_value_as_int(element)),
                        }? as i32;

                        values.push(value);
                        slot += 1;
                    }
                }

                let text = map_entry
                    .get(b"RuleNameLabel".as_slice())
                    .map_or_else(Vec::new, |found| match found {
                        JsonNode::String(text) => json_string_as_string(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                        _ => json_value_as_string(found),
                    });

                ctx.special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .rule_name_label = text;

                let text = map_entry
                    .get(b"RuleExplanationLabel".as_slice())
                    .map_or_else(Vec::new, |found| match found {
                        JsonNode::String(text) => json_string_as_string(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                        _ => json_value_as_string(found),
                    });

                ctx.special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .rule_explanation_label = text;

                let contents = map_entry
                    .get(b"ContentsType".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })? as i32;

                ctx.special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .contents_type = contents;

                if ctx
                    .special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .contents_type
                    <= 0
                {
                    continue;
                }

                let Some(JsonNode::Object(reset)) = map_entry.get(b"Reset".as_slice()) else {
                    continue;
                };

                let flag = reset
                    .get(b"ResetFlag".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })? as i32;

                if flag != 1 {
                    continue;
                }

                let contents = ctx
                    .special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .contents_type;

                ctx.special_rules
                    .reset_popups
                    .entry(contents)
                    .or_default()
                    .map_id = map_id;

                let text = reset
                    .get(b"PopupLabel".as_slice())
                    .map_or_else(Vec::new, |found| match found {
                        JsonNode::String(text) => json_string_as_string(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                        _ => json_value_as_string(found),
                    });
                let contents = ctx
                    .special_rules
                    .maps
                    .entry(map_id)
                    .or_default()
                    .contents_type;

                ctx.special_rules
                    .reset_popups
                    .entry(contents)
                    .or_default()
                    .popup_label = text;
            }
        }
    }

    let Some(bytes) = open_asset_stream(ctx, b"SpecialRulesMapOption.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(rule_types)) = root.get(b"RuleType".as_slice()) else {
        return Ok(());
    };

    for (rule_key, rule_entry) in rule_types {
        let rule = string_to_int(rule_key)?;
        let JsonNode::Object(rule_entry) = rule_entry else {
            return Err(Fault::null_pointer());
        };
        let Some(JsonNode::Array(ids)) = rule_entry.get(b"InvalidNyancomboID".as_slice()) else {
            return Err(Fault::null_pointer());
        };

        if ids.is_empty() {
            continue;
        }

        let mut slot = 0usize;

        loop {
            let values = ctx
                .special_rules
                .invalid_nyancombo_ids
                .entry(rule)
                .or_default();
            let element = ids.get(slot).ok_or(Fault::index_out_of_range(slot as i64, ids.len() as i64))?;

            let value = match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32;

            values.push(value);
            slot += 1;

            if ids.len() <= slot {
                break;
            }
        }
    }

    Ok(())
}
