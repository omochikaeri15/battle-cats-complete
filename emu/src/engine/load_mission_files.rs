use std::rc::Rc;

use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, JsonNode, MissionCondition, MissionData, MissionGatyaSetting,
    MissionLimitOption, cell_is_int, get_column_count, json_parse_object_document,
    json_source_from_string, load_mission_monthly_files, load_mission_name_csv, max_i32,
    open_asset_stream, parse_mission_condition_row, parse_mission_data_row,
    parse_mission_gatya_setting, parse_mission_limit_option, parse_mission_unlock_row,
    read_cell_stream, read_csv_cell, read_csv_row, read_stream_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionConditionSetting {
    pub shortcut_type: i32,
    pub sort_type: i32,
    pub filter_condition: i32,
}

pub fn load_mission_files(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.mission_gatya_settings.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_NewGatyaSetting.json", 0, 0)? {
        let source = json_source_from_string(&bytes);
        let document = json_parse_object_document(Some(source))?;
        let Some(JsonNode::Object(root)) = document.as_ref() else {
            return Err(Fault::null_pointer());
        };
        let Some(JsonNode::Object(entries)) = root.get(b"ID".as_slice()) else {
            return Err(Fault::null_pointer());
        };

        for (key, entry) in entries {
            let id = ops::atoi(key);
            let mut record = MissionGatyaSetting::default();

            parse_mission_gatya_setting(
                &mut record,
                match entry {
                    JsonNode::Object(_) => Some(entry),
                    _ => None,
                },
            )?;
            ctx.mission_gatya_settings.insert(id, Rc::new(record));
        }
    }

    ctx.mission_limit_options.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_LimitOption.json", 0, 0)? {
        let source = json_source_from_string(&bytes);
        let document = json_parse_object_document(Some(source))?;
        let Some(JsonNode::Object(root)) = document.as_ref() else {
            return Err(Fault::null_pointer());
        };
        let Some(JsonNode::Object(entries)) = root.get(b"limitID".as_slice()) else {
            return Err(Fault::null_pointer());
        };

        for (key, entry) in entries {
            let id = ops::atoi(key);
            let mut record = MissionLimitOption::default();

            parse_mission_limit_option(
                &mut record,
                match entry {
                    JsonNode::Object(_) => Some(entry),
                    _ => None,
                },
            )?;
            ctx.mission_limit_options.insert(id, Rc::new(record));
        }
    }

    ctx.mission_unlock_conditions.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_Unlock_Condition.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);

        while read_csv_row(stm) {
            let mut row = [0i32; 10];

            parse_mission_unlock_row(&mut row, stm);

            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_unlock_conditions.insert(key, row);
        }
    }

    ctx.mission_condition_settings.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_Condition_Setting.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let setting = MissionConditionSetting {
                shortcut_type: read_csv_cell(stm, 1) as i32,
                sort_type: read_csv_cell(stm, 2) as i32,
                filter_condition: read_csv_cell(stm, 3) as i32,
            };
            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_condition_settings.insert(key, Rc::new(setting));
        }
    }

    ctx.mission_data.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_Data.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);

        while read_csv_row(stm) {
            let key = read_csv_cell(stm, 0) as i32;
            let mut row = MissionData::default();

            parse_mission_data_row(&mut row, stm);
            ctx.mission_data.insert(key, row);
        }
    }

    let mut max_type3 = 0i32;

    if let Some(bytes) = open_asset_stream(ctx, b"Mission_Condition.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut max_type1 = 0i32;
        let mut max_type2 = 0i32;
        let mut max_type0 = 0i32;

        read_csv_row(stm);

        while read_csv_row(stm) {
            let id = read_csv_cell(stm, 0) as i32;

            if !ctx.mission_data.contains_key(&id) {
                continue;
            }

            let mut condition = MissionCondition::default();

            parse_mission_condition_row(&mut condition, stm);

            let mission_type = condition.mission_type;

            if let Some(entry) = ctx.mission_data.get_mut(&id) {
                entry.condition = condition;
            }

            if mission_type as u32 <= 3 {
                match mission_type {
                    0 => {
                        let value = id.wrapping_add(0x3a98);

                        if max_type0 <= value {
                            max_type0 = value;
                        }
                    }
                    1 => {
                        if max_type1 <= id {
                            max_type1 = id;
                        }
                    }
                    2 => {
                        if max_type2 <= id {
                            max_type2 = id;
                        }
                    }
                    _ => {
                        if max_type3 <= id {
                            max_type3 = id;
                        }
                    }
                }
            }

            let shortcut = read_csv_cell(stm, 3) as i32;

            if let Some(setting) = ctx.mission_condition_settings.get(&shortcut).cloned()
                && let Some(entry) = ctx.mission_data.get_mut(&id)
            {
                entry.condition_setting = Some(setting);
            }

            let conditions_type = ctx
                .mission_data
                .get(&id)
                .map_or(0, |entry| entry.condition.conditions_type);

            if conditions_type == 0x18 {
                let values = ctx
                    .mission_data
                    .get(&id)
                    .map_or_else(Vec::new, |entry| entry.condition.values.clone());
                let mut options: Vec<Rc<MissionLimitOption>> = Vec::new();

                for value in &values {
                    let option = ctx
                        .mission_limit_options
                        .get(value)
                        .or_else(|| ctx.mission_limit_options.values().next());

                    if let Some(option) = option {
                        options.push(Rc::clone(option));
                    }
                }

                if let Some(entry) = ctx.mission_data.get_mut(&id) {
                    entry.condition.limit_options = options;
                }
            } else if conditions_type == 0x20 {
                let first = ctx
                    .mission_data
                    .get(&id)
                    .and_then(|entry| entry.condition.values.first())
                    .copied()
                    .unwrap_or(0);
                let setting = ctx
                    .mission_gatya_settings
                    .get(&first)
                    .or_else(|| ctx.mission_gatya_settings.values().next())
                    .map(Rc::clone);

                if let Some(entry) = ctx.mission_data.get_mut(&id) {
                    entry.condition.gatya_setting = setting;
                }
            }
        }

        ctx.mission_max_type2 = max_type2;
        ctx.mission_max_type1 = max_type1;
        ctx.mission_max_type0 = max_type0;
    }

    load_mission_monthly_files(ctx)?;

    let mut monthly = -1i32;

    for record in ctx.mission_monthly.values() {
        monthly = max_i32(monthly, record.id);
    }

    ctx.mission_max_type3 = max_i32(max_type3, monthly);
    ctx.mission_names.clear();
    ctx.mission_descriptions.clear();
    load_mission_name_csv(ctx)?;

    let Some(bytes) = open_asset_stream(ctx, b"Mission_Name.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_stream_row(stm, b',') {
        if (get_column_count(stm) as i32) < 2 {
            return Ok(());
        }

        if !cell_is_int(stm, 0) {
            return Ok(());
        }

        let text = read_cell_stream(stm, 1).to_vec();
        let key = read_csv_cell(stm, 0) as i32;

        ctx.mission_names.insert(key, text);

        if get_column_count(stm) as i32 >= 3 {
            let text = read_cell_stream(stm, 2).to_vec();
            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_descriptions.insert(key, text);
        } else {
            let key = read_csv_cell(stm, 0) as i32;

            ctx.mission_descriptions.insert(key, Vec::new());
        }
    }

    Ok(())
}
