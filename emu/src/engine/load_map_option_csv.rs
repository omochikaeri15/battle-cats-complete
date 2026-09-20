use crate::Fault;

use super::{
    AppContext, AssetStream, cell_is_int, get_column_count, open_asset_stream, read_csv_cell,
    read_csv_row,
};

pub fn load_map_option_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.map_options.stars_unlocked.clear();
    ctx.map_options.back_stars_unlocked.clear();
    ctx.map_options.star_multipliers.clear();
    ctx.map_options.guerrilla_maps.clear();
    ctx.map_options.guerrilla_set.clear();
    ctx.map_options.reward_reset_kind.clear();
    ctx.map_options.show_once.clear();
    ctx.map_options.display_order.clear();
    ctx.map_options.interval.clear();
    ctx.map_options.challenge_maps.clear();
    ctx.map_options.difficulty.clear();
    ctx.map_options.hide_after_clear_maps.clear();
    ctx.map_options.double_xp_ad_maps.clear();
    ctx.map_options.cost_multipliers.clear();
    ctx.map_options.no_energy_maps.clear();
    ctx.map_options.conditioned_maps.clear();

    let Some(bytes) = open_asset_stream(ctx, b"Map_option.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_csv_row(stm);

    while read_csv_row(stm) {
        let map_id = read_csv_cell(stm, 0) as i32;

        ctx.map_options
            .stars_unlocked
            .insert(map_id, read_csv_cell(stm, 1) as i32);
        ctx.map_options
            .back_stars_unlocked
            .insert(map_id, read_csv_cell(stm, 2) as i32);

        let mut multipliers: Vec<i32> = Vec::new();
        let mut column = 3i32;

        while column != 7 {
            multipliers.push(read_csv_cell(stm, column) as i32);
            column += 1;
        }

        ctx.map_options.star_multipliers.insert(map_id, multipliers);

        let schedule = read_csv_cell(stm, 7) as i32;

        if schedule > 0 {
            let mut bit = 0i32;
            let mut mask = 1i32;

            while mask <= schedule {
                if schedule & mask != 0 {
                    ctx.map_options
                        .guerrilla_maps
                        .entry(bit)
                        .or_default()
                        .push(map_id);
                }

                mask = 2i32.wrapping_shl(bit as u32);
                bit += 1;
            }
        }

        ctx.map_options
            .guerrilla_set
            .insert(map_id, read_csv_cell(stm, 7) as i32);
        ctx.map_options
            .reward_reset_kind
            .insert(map_id, read_csv_cell(stm, 8) as i32);
        ctx.map_options
            .show_once
            .insert(map_id, read_csv_cell(stm, 9) as i32);
        ctx.map_options
            .display_order
            .insert(map_id, read_csv_cell(stm, 0xa) as i32);
        ctx.map_options
            .interval
            .insert(map_id, read_csv_cell(stm, 0xb) as i32);

        if read_csv_cell(stm, 0xc) != 0 {
            ctx.map_options.challenge_maps.push(map_id);
        }

        ctx.map_options
            .difficulty
            .insert(map_id, read_csv_cell(stm, 0xd) as i32);

        if read_csv_cell(stm, 0xe) != 0 {
            ctx.map_options.hide_after_clear_maps.push(map_id);
        }

        if read_csv_cell(stm, 0xf) != 0 {
            ctx.map_options.double_xp_ad_maps.push(map_id);
        }

        ctx.map_options
            .cost_multipliers
            .insert(map_id, read_csv_cell(stm, 0x10) as i32);

        if 0x11 < get_column_count(stm) as i32
            && cell_is_int(stm, 0x11)
            && read_csv_cell(stm, 0x11) == 0
        {
            ctx.map_options.no_energy_maps.push(map_id);
        }

        if 0x12 < get_column_count(stm) as i32
            && cell_is_int(stm, 0x12)
            && read_csv_cell(stm, 0x12) != 0
        {
            ctx.map_options.conditioned_maps.push(map_id);
        }
    }

    Ok(())
}
