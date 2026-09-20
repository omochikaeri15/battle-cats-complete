use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, ItemShopRow, JsonNode, MatatabiRow, format_localized,
    get_column_count, item_shop_entry, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int,
    json_string_as_string, json_value_as_int, json_value_as_string, load_cat_drops_csv,
    load_clear_count_reward_json, load_drop_item_csv, load_gamatoto_files,
    load_gift_and_limit_tables, load_god_texts, load_localizable_tsv, load_map_stage_shortcut_csv,
    load_mission_files, load_nyancombo_files, load_opening_ending_texts, load_picture_book_files,
    load_stage_name_files, load_unlock_popup_tsv, load_zombie_lottery_csv, matatabi_row_init,
    normalize_search_text, obfuscate_value, open_asset_stream, query_localizable,
    read_cell_stream, read_csv_cell, read_csv_row, read_stream_row, read_tsv_row,
    string_format_rank_comment,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventDisplayRow {
    pub images: Vec<Vec<u8>>,
    pub fusuma_se: Vec<i32>,
    pub neko_bage_bgm: i32,
    pub fusuma_open_offset: Vec<i32>,
    pub percent_banner: i32,
    pub percent_title: i32,
    pub event_weight: i32,
    pub select_stage_img: Vec<u8>,
    pub select_stage_img_japan: Vec<u8>,
    pub select_stage_img_future: Vec<u8>,
    pub select_stage_img_space: Vec<u8>,
    pub select_stage_img_neko_dojo: Vec<u8>,
    pub select_stage_range: i32,
    pub select_stage_img_reverse: bool,
    pub select_stage_img_reverse_japan: bool,
    pub select_stage_img_reverse_future: bool,
    pub select_stage_img_reverse_space: bool,
    pub select_stage_img_reverse_neko_dojo: bool,
    pub priority: i32,
    pub battle_start_se: i32,
    pub battle_finish_win_se_type: i32,
    pub battle_finish_lose_se_type: i32,
    pub battle_finish_win_se: Vec<i32>,
    pub battle_finish_lose_se: Vec<i32>,
    pub popup_get_se: i32,
    pub title_bgm: i32,
    pub select_stage_start_se: i32,
    pub battle_bgm_play_timing: i32,
}

pub fn load_misc_data_tables(ctx: &mut AppContext) -> Result<(), Fault> {
    load_localizable_tsv(ctx)?;
    load_unlock_popup_tsv(ctx)?;
    load_drop_item_csv(ctx)?;
    load_zombie_lottery_csv(ctx)?;
    load_gift_and_limit_tables(ctx)?;
    load_gamatoto_files(ctx)?;
    load_stage_name_files(ctx)?;
    load_opening_ending_texts(ctx)?;
    load_picture_book_files(ctx)?;
    load_god_texts(ctx)?;
    load_nyancombo_files(ctx)?;
    load_mission_files(ctx)?;
    load_clear_count_reward_json(ctx)?;
    load_map_stage_shortcut_csv(ctx)?;
    load_cat_drops_csv(ctx)?;

    ctx.map_names.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Map_Name.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let key = read_csv_cell(stm, 0) as i32;
            let text = read_cell_stream(stm, 1).to_vec();

            ctx.map_names.insert(key, text);
        }
    }

    ctx.matatabi_rows.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Matatabi.tsv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_tsv_row(stm);

        while read_tsv_row(stm) {
            let gatya_id = read_csv_cell(stm, 0) as i32;
            let seed = read_csv_cell(stm, 1) as i32;
            let group = read_csv_cell(stm, 2) as i32;
            let sort = read_csv_cell(stm, 3) as i32;
            let require = read_csv_cell(stm, 4) as i32;
            let guide_text = read_cell_stream(stm, 5).to_vec();
            let text = read_cell_stream(stm, 6).to_vec();
            let mut row = MatatabiRow::default();

            matatabi_row_init(
                &mut row, gatya_id, seed, group, sort, require, &guide_text, &text,
            );
            ctx.matatabi_rows.push(row);
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"GatyaitemName.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.item_names.clear();
        ctx.item_descriptions.clear();

        let mut slot = 0i32;

        while slot != 275 {
            read_stream_row(stm, b',');
            ctx.item_names.push(read_cell_stream(stm, 0).to_vec());
            ctx.item_descriptions.push([
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"rankGiftMessage.tsv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.rank_gift_messages.clear();

        let mut slot = 0i32;

        while slot != 400 {
            read_tsv_row(stm);
            ctx.rank_gift_messages
                .push(read_cell_stream(stm, 0).to_vec());
            slot += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"eventDisplayData.json", 0, 0)? {
        let source = json_source_from_string(&bytes);
        let document = json_parse_object_document(Some(source))?;
        let Some(JsonNode::Object(root)) = document.as_ref() else {
            return Err(Fault::null_pointer());
        };
        let Some(JsonNode::Object(entries)) = root.get(b"MapSet".as_slice()) else {
            return Err(Fault::null_pointer());
        };

        for (key, entry) in entries {
            let map_id = ops::atoi(key);
            let JsonNode::Object(fields) = entry else {
                return Err(Fault::null_pointer());
            };

            if let Some(JsonNode::Array(values)) = fields.get(b"FusumaSE".as_slice()) {
                let mut slot = 0usize;

                while slot < values.len() {
                    let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;
                    let value = match element {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(element)),
                    }? as i32;

                    ctx.event_display
                        .entry(map_id)
                        .or_default()
                        .fusuma_se
                        .push(value);
                    slot += 1;
                }
            }

            for key in [
                b"FusumaOpenOffsetX".as_slice(),
                b"FusumaOpenOffsetY".as_slice(),
            ] {
                let value = fields.get(key).map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.event_display
                    .entry(map_id)
                    .or_default()
                    .fusuma_open_offset
                    .push(value);
            }

            for key in [
                b"FusumaImgPNG".as_slice(),
                b"FusumaImgIMGCUT".as_slice(),
                b"BannerImgPNG".as_slice(),
                b"BannerImgIMGCUT".as_slice(),
                b"NekoBageImg".as_slice(),
                b"TitleImgBG".as_slice(),
                b"TitleImgLogo".as_slice(),
            ] {
                let text = fields.get(key).map_or_else(Vec::new, |found| match found {
                    JsonNode::String(text) => json_string_as_string(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                    _ => json_value_as_string(found),
                });

                ctx.event_display
                    .entry(map_id)
                    .or_default()
                    .images
                    .push(text);
            }

            let value = fields
                .get(b"NekoBageBgm".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().neko_bage_bgm = value;

            let value = fields
                .get(b"PercentBanner".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().percent_banner = value;

            let value = fields
                .get(b"PercentTitle".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().percent_title = value;

            let value = fields
                .get(b"eventWeight".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().event_weight = value;

            let text = fields
                .get(b"SelectStageImg".as_slice())
                .map_or_else(Vec::new, |found| match found {
                    JsonNode::String(text) => json_string_as_string(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                    _ => json_value_as_string(found),
                });

            ctx.event_display
                .entry(map_id)
                .or_default()
                .select_stage_img = text;

            let value = fields
                .get(b"SelectStageRange".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .select_stage_range = value;

            let value = fields
                .get(b"SelectStageImgReverse".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .select_stage_img_reverse = value != 0;

            let value = fields
                .get(b"priority".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().priority = value;

            for (name, flag, region) in [
                (
                    b"SelectStageImgJapan".as_slice(),
                    b"SelectStageImgReverseJapan".as_slice(),
                    0usize,
                ),
                (
                    b"SelectStageImgFuture".as_slice(),
                    b"SelectStageImgReverseFuture".as_slice(),
                    1,
                ),
                (
                    b"SelectStageImgSpace".as_slice(),
                    b"SelectStageImgReverseSpace".as_slice(),
                    2,
                ),
                (
                    b"SelectStageImgNekoDojo".as_slice(),
                    b"SelectStageImgReverseNekoDojo".as_slice(),
                    3,
                ),
            ] {
                let text = fields.get(name).map_or_else(Vec::new, |found| match found {
                    JsonNode::String(text) => json_string_as_string(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                    _ => json_value_as_string(found),
                });
                let row = ctx.event_display.entry(map_id).or_default();

                match region {
                    0 => row.select_stage_img_japan = text,
                    1 => row.select_stage_img_future = text,
                    2 => row.select_stage_img_space = text,
                    _ => row.select_stage_img_neko_dojo = text,
                }

                let value = fields.get(flag).map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;
                let row = ctx.event_display.entry(map_id).or_default();

                match region {
                    0 => row.select_stage_img_reverse_japan = value != 0,
                    1 => row.select_stage_img_reverse_future = value != 0,
                    2 => row.select_stage_img_reverse_space = value != 0,
                    _ => row.select_stage_img_reverse_neko_dojo = value != 0,
                }
            }

            let value = fields
                .get(b"BattleStartSE".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().battle_start_se = value;

            let value = fields
                .get(b"BattleFinishWinSEtype".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .battle_finish_win_se_type = value;

            let value = fields
                .get(b"BattleFinishLoseSEtype".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .battle_finish_lose_se_type = value;

            let value = fields
                .get(b"BattleBGMPlayTiming".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .battle_bgm_play_timing = value;

            for (name, win) in [
                (b"BattleFinishWinSE".as_slice(), true),
                (b"BattleFinishLoseSE".as_slice(), false),
            ] {
                let Some(JsonNode::Array(values)) = fields.get(name) else {
                    continue;
                };
                let mut slot = 0usize;

                while slot < values.len() {
                    let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;
                    let value = match element {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(element)),
                    }? as i32;
                    let row = ctx.event_display.entry(map_id).or_default();

                    if win {
                        row.battle_finish_win_se.push(value);
                    } else {
                        row.battle_finish_lose_se.push(value);
                    }

                    slot += 1;
                }
            }

            let value = fields
                .get(b"PopUpGetSE".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().popup_get_se = value;

            let value = fields
                .get(b"TitleBgm".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display.entry(map_id).or_default().title_bgm = value;

            let value = fields
                .get(b"SelectStageStartSE".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            ctx.event_display
                .entry(map_id)
                .or_default()
                .select_stage_start_se = value;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Warning1_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.warning1_texts.clear();

        let mut slot = 0i32;

        while slot != 53 {
            read_stream_row(stm, b',');
            ctx.warning1_texts.push(read_cell_stream(stm, 0).to_vec());
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Warning2_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.warning2_rows.clear();

        let mut slot = 0i32;

        while slot != 143 {
            read_stream_row(stm, b',');
            ctx.warning2_rows.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    ctx.main_menu_rows.clear();
    ctx.main_menu_row_settings.clear();

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"MainMenu_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let mut row: Vec<Vec<u8>> = Vec::new();
            let mut column = 1i32;

            while column < get_column_count(stm) as i32 {
                row.push(read_cell_stream(stm, column).to_vec());
                column += 1;
            }

            ctx.main_menu_row_settings
                .push(read_csv_cell(stm, 0) as i32);
            ctx.main_menu_rows.push(row);
        }
    }

    ctx.main_menu_text_settings.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"MainMenuTextSetting.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let mut row: Vec<i32> = Vec::new();
            let mut column = 0i32;

            while column < get_column_count(stm) as i32 {
                row.push(read_csv_cell(stm, column) as i32);
                column += 1;
            }

            ctx.main_menu_text_settings.push(row);
        }
    }

    ctx.cat_names.clear();

    let mut unit = 0i32;

    while unit != 876 {
        let file = unit.wrapping_add(1);
        let lang = query_localizable(ctx, b"lang");
        let name =
            string_format_rank_comment(ctx, b"Unit_Explanation%d_%@.csv", file, &lang)?;
        let mut forms: [[Vec<u8>; 5]; 4] = Default::default();

        if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
            let stm = &mut AssetStream::new(&bytes, b'\n');
            let mut form = 0usize;

            while form != 4 {
                read_stream_row(stm, b',');

                let tail = read_cell_stream(stm, 4).to_vec();

                if let Some(record) = forms.get_mut(form) {
                    record[0] = read_cell_stream(stm, 0).to_vec();
                    record[1] = read_cell_stream(stm, 1).to_vec();
                    record[2] = read_cell_stream(stm, 2).to_vec();
                    record[3] = read_cell_stream(stm, 3).to_vec();
                    record[4] = normalize_search_text(&tail)?;
                }

                form += 1;
            }
        }

        ctx.cat_names.push(forms);
        unit += 1;
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"God_Explanation_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.god_explanation = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
            read_cell_stream(stm, 2).to_vec(),
            read_cell_stream(stm, 3).to_vec(),
        ];
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StageFirstMessage_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.stage_first_messages.clear();

        let mut slot = 0i32;

        while slot != 3 {
            read_stream_row(stm, b',');
            ctx.stage_first_messages.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"ChallengeMode_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.challenge_mode_texts = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
            read_cell_stream(stm, 2).to_vec(),
            read_cell_stream(stm, 3).to_vec(),
        ];
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"PageName_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.page_names.clear();

        let mut slot = 0i32;

        while slot != 14 {
            read_stream_row(stm, b',');
            ctx.page_names.push(read_cell_stream(stm, 0).to_vec());
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"FirstLose_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_stream_row(stm, b',');
        ctx.first_lose_texts = [
            read_cell_stream(stm, 0).to_vec(),
            read_cell_stream(stm, 1).to_vec(),
        ];
    }

    ctx.lose_row_settings.clear();
    ctx.lose_rows.clear();

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Lose_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_stream_row(stm, b',') {
            let mut row: Vec<Vec<u8>> = Vec::new();
            let mut column = 1i32;

            while column < get_column_count(stm) as i32 {
                row.push(read_cell_stream(stm, column).to_vec());
                column += 1;
            }

            ctx.lose_row_settings.push(read_csv_cell(stm, 0) as i32);
            ctx.lose_rows.push(row);
        }
    }

    ctx.lose_text_settings.clear();

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"LoseTextSetting.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            let mut row: Vec<i32> = Vec::new();
            let mut column = 0i32;

            while column < get_column_count(stm) as i32 {
                row.push(read_csv_cell(stm, column) as i32);
                column += 1;
            }

            ctx.lose_text_settings.push(row);
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Option_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.option_rows.clear();

        let mut slot = 0i32;

        while slot != 3 {
            read_stream_row(stm, b',');
            ctx.option_rows.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
            ]);
            slot += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"itemShopData.tsv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_tsv_row(stm);

        while read_tsv_row(stm) {
            let mut entry = ItemShopRow {
                gatya_item_id: read_csv_cell(stm, 1) as i32,
                ..ItemShopRow::default()
            };

            entry.item_value[..4]
                .copy_from_slice(&(read_csv_cell(stm, 2) as i32).to_le_bytes());
            obfuscate_value(&mut entry.item_value);
            entry.item_price[..4]
                .copy_from_slice(&(read_csv_cell(stm, 3) as i32).to_le_bytes());
            obfuscate_value(&mut entry.item_price);
            entry.draw_item = u8::from(read_csv_cell(stm, 4) as i32 == 1);
            entry.category = read_cell_stream(stm, 5).to_vec();
            entry.imgcut = read_csv_cell(stm, 6) as i32;

            let shop_id = read_csv_cell(stm, 0) as i32;
            let row = item_shop_entry(&mut ctx.item_shop_rows, shop_id);

            row.gatya_item_id = entry.gatya_item_id;
            row.item_value = entry.item_value;
            row.item_price = entry.item_price;
            row.draw_item = entry.draw_item;
            row.category = entry.category;
            row.imgcut = entry.imgcut;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"MainMenuPopUp_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.main_menu_popups.clear();

        let mut slot = 0i32;

        while slot != 15 {
            read_stream_row(stm, b',');
            ctx.main_menu_popups.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Tutorial_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.tutorial_pages.clear();

        let mut slot = 0i32;

        while slot != 31 {
            read_stream_row(stm, b',');

            let mut page: [Vec<u8>; 12] = Default::default();
            let mut column = 0i32;

            while column != 12 {
                if let Some(cell) = page.get_mut(column as usize) {
                    *cell = read_cell_stream(stm, column).to_vec();
                }

                column += 1;
            }

            ctx.tutorial_pages.push(page);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"PopUpMessage_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.popup_messages.clear();

        let mut slot = 0i32;

        while slot != 6 {
            read_stream_row(stm, b',');

            let mut message: [Vec<u8>; 10] = Default::default();
            let mut column = 0i32;

            while column != 10 {
                if let Some(cell) = message.get_mut(column as usize) {
                    *cell = read_cell_stream(stm, column).to_vec();
                }

                column += 1;
            }

            ctx.popup_messages.push(message);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"Category_Explanation_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.category_explanations.clear();

        let mut slot = 0i32;

        while slot != 20 {
            read_stream_row(stm, b',');
            ctx.category_explanations.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"StampMessage_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.stamp_messages.clear();

        let mut slot = 0i32;

        while slot != 20 {
            read_stream_row(stm, b',');
            ctx.stamp_messages.push(read_cell_stream(stm, 0).to_vec());
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"GiftMessage_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.gift_messages.clear();

        let mut slot = 0i32;

        while slot != 14 {
            read_stream_row(stm, b',');
            ctx.gift_messages.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    let lang = query_localizable(ctx, b"lang");
    let name = format_localized(ctx, b"unitevolve_%@.csv", &lang)?;

    if let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.unit_evolve_rows.clear();

        let mut slot = 0i32;

        while slot != 350 {
            read_stream_row(stm, b',');
            ctx.unit_evolve_rows.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 4).to_vec(),
                read_cell_stream(stm, 5).to_vec(),
                read_cell_stream(stm, 6).to_vec(),
            ]);
            slot += 1;
        }
    }

    if let Some(bytes) = open_asset_stream(ctx, b"matatabi_Popup.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        ctx.matatabi_popups.clear();

        let mut slot = 0i32;

        while slot != 8 {
            read_stream_row(stm, b',');
            ctx.matatabi_popups.push([
                read_cell_stream(stm, 0).to_vec(),
                read_cell_stream(stm, 1).to_vec(),
                read_cell_stream(stm, 2).to_vec(),
                read_cell_stream(stm, 3).to_vec(),
            ]);
            slot += 1;
        }
    }

    ctx.reward_stage_ids.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"rewardStage.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if read_csv_cell(stm, 0) as i32 == -1 {
                break;
            }

            ctx.reward_stage_ids.push(read_csv_cell(stm, 0) as i32);
        }
    }

    ctx.daily_login_rows.clear();
    ctx.daily_login_groups.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"DailyLoginEventData.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if get_column_count(stm) as i32 <= 1 {
                break;
            }

            ctx.daily_login_rows.push(vec![
                read_csv_cell(stm, 0) as i32,
                read_csv_cell(stm, 1) as i32,
                read_csv_cell(stm, 2) as i32,
                read_csv_cell(stm, 3) as i32,
                read_csv_cell(stm, 4) as i32,
                read_csv_cell(stm, 5) as i32,
                read_csv_cell(stm, 6) as i32,
            ]);

            if ctx.daily_login_max_id < read_csv_cell(stm, 0) as i32 {
                ctx.daily_login_max_id = read_csv_cell(stm, 0) as i32;
            }

            let mut groups: Vec<Vec<i32>> = Vec::new();
            let mut column = 7i32;

            while column < get_column_count(stm) as i32 - 1 {
                let mut entry: Vec<i32> = Vec::new();

                entry.push(read_csv_cell(stm, column) as i32);
                entry.push(read_csv_cell(stm, column.wrapping_add(1)) as i32);

                let mut item = 0i32;

                while item < read_csv_cell(stm, column) as i32 {
                    let base = column.wrapping_add(2).wrapping_add(item.wrapping_mul(3));

                    entry.push(read_csv_cell(stm, base) as i32);
                    entry.push(read_csv_cell(stm, base.wrapping_add(1)) as i32);
                    entry.push(read_csv_cell(stm, base.wrapping_add(2)) as i32);
                    item += 1;
                }

                groups.push(entry);
                column = column
                    .wrapping_add(2)
                    .wrapping_add((read_csv_cell(stm, column) as i32).wrapping_mul(3));
            }

            ctx.daily_login_groups.push(groups);
        }
    }

    ctx.beacon_rows.clear();
    ctx.beacon_groups.clear();

    let Some(bytes) = open_asset_stream(ctx, b"BeaconEventData.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');

    while read_csv_row(stm) {
        if get_column_count(stm) as i32 <= 1 {
            break;
        }

        ctx.beacon_rows.push(vec![
            read_csv_cell(stm, 0) as i32,
            read_csv_cell(stm, 1) as i32,
            read_csv_cell(stm, 2) as i32,
            read_csv_cell(stm, 3) as i32,
            read_csv_cell(stm, 4) as i32,
        ]);

        let mut groups: Vec<Vec<i32>> = Vec::new();
        let mut column = 5i32;

        while column < get_column_count(stm) as i32 - 1 {
            let mut entry: Vec<i32> = Vec::new();

            entry.push(read_csv_cell(stm, column) as i32);
            entry.push(read_csv_cell(stm, column.wrapping_add(1)) as i32);

            let mut item = 0i32;

            while item < read_csv_cell(stm, column) as i32 {
                let base = column.wrapping_add(2).wrapping_add(item.wrapping_mul(3));

                entry.push(read_csv_cell(stm, base) as i32);
                entry.push(read_csv_cell(stm, base.wrapping_add(1)) as i32);
                entry.push(read_csv_cell(stm, base.wrapping_add(2)) as i32);
                item += 1;
            }

            groups.push(entry);
            column = column
                .wrapping_add(2)
                .wrapping_add((read_csv_cell(stm, column) as i32).wrapping_mul(3));
        }

        ctx.beacon_groups.push(groups);
    }

    Ok(())
}
