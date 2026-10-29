use crate::Fault;

use super::{
    AppContext, JsonNode, bg_effect_def_new, bg_param_set_value, json_container_as_int,
    json_container_as_string, json_parse_object_document, json_source_from_string,
    json_string_as_int, json_string_as_string, json_value_as_int, json_value_as_string,
    maanim_load, mamodel_load, open_asset_stream, parse_bg_equally_spaced, parse_bg_param_float,
    parse_bg_param_int, string_format_int, texture_cache_load,
};

pub fn load_bg_effect_json(ctx: &mut AppContext, background: i32) -> Result<(), Fault> {
    let name = string_format_int(ctx, b"bg%03d.json", background)?;
    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(());
    };
    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };

    if root.contains_key(b"id".as_slice()) {
        let id = root
            .get(b"id".as_slice())
            .map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

        return load_bg_effect_json(ctx, id);
    }

    let Some(JsonNode::Array(data)) = root.get(b"data".as_slice()) else {
        return Err(Fault::null_pointer());
    };

    let mut index = 0usize;

    while index < data.len() {
        ctx.bg_effects.defs.push(bg_effect_def_new());

        let JsonNode::Object(entry) = &data[index] else {
            return Err(Fault::null_pointer());
        };
        let def = ctx
            .bg_effects
            .defs
            .last_mut()
            .ok_or(Fault::null_pointer())?;

        parse_bg_param_int(
            &mut def.count,
            match entry.get(b"count".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.model,
            match entry.get(b"model".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.x,
            match entry.get(b"x".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_x,
            match entry.get(b"startX".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.y,
            match entry.get(b"y".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_y,
            match entry.get(b"startY".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.z,
            match entry.get(b"z".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.scale,
            match entry.get(b"scale".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_scale,
            match entry.get(b"startScale".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.scale_x,
            match entry.get(b"scaleX".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_scale_x,
            match entry.get(b"startScaleX".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.scale_y,
            match entry.get(b"scaleY".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_scale_y,
            match entry.get(b"startScaleY".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.angle,
            match entry.get(b"angle".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.angular_v,
            match entry.get(b"angularV".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.alpha,
            match entry.get(b"alpha".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.v,
            match entry.get(b"v".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_v,
            match entry.get(b"startV".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.move_angle,
            match entry.get(b"moveAngle".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.vx,
            match entry.get(b"vx".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_vx,
            match entry.get(b"startVx".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.vy,
            match entry.get(b"vy".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.start_vy,
            match entry.get(b"startVy".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.destroy_left,
            match entry.get(b"destroyLeft".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.destroy_right,
            match entry.get(b"destroyRight".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.destroy_top,
            match entry.get(b"destroyTop".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_float(
            &mut def.destroy_bottom,
            match entry.get(b"destroyBottom".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.frame,
            match entry.get(b"frame".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.start_frame,
            match entry.get(b"startFrame".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.wait,
            match entry.get(b"wait".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        parse_bg_param_int(
            &mut def.life_time,
            match entry.get(b"lifeTime".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;

        if let Some(file) = match entry.get(b"file".as_slice()) {
            Some(JsonNode::Object(found)) => Some(found),
            _ => None,
        } {
            let key = !(index as i32);
            bg_param_set_value(&mut def.model, key);
            ctx.bg_effects.image_names.insert(
                key,
                match file.get(b"image".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                },
            );
            ctx.bg_effects.model_names.insert(
                key,
                match file.get(b"model".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                },
            );
            ctx.bg_effects.model_anims.insert(
                key,
                match file.get(b"anime".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                },
            );

            let image =
                ctx.bg_effects
                    .image_names
                    .get(&key)
                    .cloned()
                    .ok_or(Fault::key_not_found(key as i64))?;

            if !ctx.bg_effect_sheets.contains_key(&image) {
                let cut = match file.get(b"imgcut".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                };
                let sheet = texture_cache_load(ctx, &image, &cut, 0x2601)?;

                ctx.bg_effect_sheets.insert(image, sheet);
            }

            let model =
                ctx.bg_effects
                    .model_names
                    .get(&key)
                    .cloned()
                    .ok_or(Fault::key_not_found(key as i64))?;

            if !ctx.bg_models.contains_key(&model) {
                let mut loaded = ctx.bg_models.remove(&model).unwrap_or_default();

                mamodel_load(ctx, &mut loaded, &model)?;
                ctx.bg_models.insert(model, loaded);
            }

            let anim = ctx
                .bg_effects
                .model_anims
                .get(&key)
                .cloned()
                .ok_or(Fault::key_not_found(key as i64))?;

            if !ctx.bg_anim_cache.contains_key(&anim) {
                let mut loaded = ctx.bg_anim_cache.remove(&anim).unwrap_or_default();

                maanim_load(ctx, &mut loaded, &anim)?;
                ctx.bg_anim_cache.insert(anim, loaded);
            }
        }

        let def = ctx
            .bg_effects
            .defs
            .last_mut()
            .ok_or(Fault::null_pointer())?;

        parse_bg_equally_spaced(
            &mut def.equally_spaced,
            match entry.get(b"equallySpaced".as_slice()) {
                Some(JsonNode::Object(found)) => Some(found),
                _ => None,
            },
        )?;
        index += 1;
    }

    Ok(())
}
