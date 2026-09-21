use crate::Fault;

use super::{
    AppContext, AssetStream, CharaGroup, StageRestriction, get_map_max_crown, get_stage_count,
    map_type_of_map_id, open_asset_stream, parse_charagroup_row, read_csv_cell, read_csv_row,
};

pub fn load_cat_group_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    if let Some(bytes) = open_asset_stream(ctx, b"Charagroup.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        read_csv_row(stm);

        while read_csv_row(stm) {
            let mut group = CharaGroup::default();

            parse_charagroup_row(&mut group, stm)?;

            let group_id = read_csv_cell(stm, 0) as i32;

            ctx.chara_groups.insert(group_id, group);
        }
    }

    let Some(bytes) = open_asset_stream(ctx, b"Stage_option.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let map_id = read_csv_cell(stm, 0) as i32;
        let crown = read_csv_cell(stm, 1) as i32;
        let stage = read_csv_cell(stm, 2) as i32;
        let mut crowns: Vec<i32> = Vec::new();

        if crown != -1 {
            crowns.push(crown);
        } else {
            let mut level = 0i32;

            while level <= get_map_max_crown(&ctx.map_options, map_id) {
                crowns.push(level);
                level += 1;
            }
        }

        let mut stages: Vec<i32> = Vec::new();

        if stage != -1 {
            stages.push(stage);
        } else {
            let map_index = map_id.wrapping_rem(1000);
            let map_type = map_type_of_map_id(map_id);
            let mut index = 0i32;

            while index < get_stage_count(ctx, map_type, map_index)? {
                stages.push(index);
                index += 1;
            }
        }

        for level in &crowns {
            for index in &stages {
                let rarity_mask = read_csv_cell(stm, 3) as i32;
                let deploy_limit = read_csv_cell(stm, 4) as i32;
                let rows = read_csv_cell(stm, 5) as i32;
                let min_cost = read_csv_cell(stm, 6) as i32;
                let max_cost = read_csv_cell(stm, 7) as i32;
                let group_id = read_csv_cell(stm, 8) as i32;
                let key = index
                    .wrapping_add(map_id.wrapping_mul(1000))
                    .wrapping_add(level.wrapping_mul(100));

                ctx.stage_restrictions.insert(
                    key,
                    StageRestriction {
                        stage: *index,
                        rarity_mask,
                        deploy_limit,
                        rows,
                        min_cost,
                        max_cost,
                        group_id,
                    },
                );
            }
        }
    }

    Ok(())
}
