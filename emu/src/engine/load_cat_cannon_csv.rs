use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, CannonGrowthStep, open_asset_stream, read_asset_stream_line,
    read_cell_stream, read_csv_cell, read_csv_row,
};

pub fn load_cat_cannon_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.cannon_parts.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"CC_AllParts_status.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut line = Cell { at: 0, len: 0 };

        read_asset_stream_line(stm, &mut line);

        while read_csv_row(stm) {
            if read_cell_stream(stm, 0).is_empty() {
                break;
            }

            let part = read_csv_cell(stm, 0) as i32;
            let value = read_csv_cell(stm, 1) as i32;

            ctx.cannon_parts.entry(part).or_default().kind = value;

            let value = read_csv_cell(stm, 2) as i32;

            ctx.cannon_parts.entry(part).or_default().makes_wave = u8::from(value != 0);

            let value = read_csv_cell(stm, 3) as i32;

            ctx.cannon_parts.entry(part).or_default().unit_id = value;

            let value = read_csv_cell(stm, 4) as i32;

            ctx.cannon_parts.entry(part).or_default().recoil = u8::from(value != 0);

            let value = read_csv_cell(stm, 5) as i32;

            ctx.cannon_parts.entry(part).or_default().ready_vfx = u8::from(value != 0);

            let value = read_csv_cell(stm, 6) as i32;

            ctx.cannon_parts.entry(part).or_default().soulstrike = u8::from(value != 0);
        }
    }

    let mut file = 0usize;

    while file != 3 {
        let name: &[u8] = match file {
            0 => b"CC_AllParts_growth.csv",
            1 => b"CC_BaseParts_growth.csv",
            _ => b"CC_DecoParts_growth.csv",
        };

        if let Some(bytes) = open_asset_stream(ctx, name, 0, 0)? {
            let stm = &mut AssetStream::new(&bytes, b'\n');
            let mut line = Cell { at: 0, len: 0 };
            let effect_base = (file as i32).wrapping_mul(100);

            read_asset_stream_line(stm, &mut line);

            while read_csv_row(stm) {
                if read_cell_stream(stm, 0).is_empty() {
                    break;
                }

                let part = read_csv_cell(stm, 0) as i32;
                let kind = read_csv_cell(stm, 1) as i32;
                let effect = kind.wrapping_add(effect_base);

                ctx.cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .push(CannonGrowthStep::default());

                let value = read_csv_cell(stm, 1) as i32;

                if let Some(step) = ctx
                    .cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .last_mut()
                {
                    step.kind = value;
                }

                let value = read_csv_cell(stm, 2) as i32;

                if let Some(step) = ctx
                    .cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .last_mut()
                {
                    step.lv2 = value;
                }

                let value = read_csv_cell(stm, 3) as i32;

                if let Some(step) = ctx
                    .cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .last_mut()
                {
                    step.value1 = value;
                }

                let value = read_csv_cell(stm, 4) as i32;

                if let Some(step) = ctx
                    .cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .last_mut()
                {
                    step.value2 = value;
                }

                let value = read_csv_cell(stm, 5) as i32;

                if let Some(step) = ctx
                    .cannon_parts
                    .entry(part)
                    .or_default()
                    .growth
                    .entry(effect)
                    .or_default()
                    .last_mut()
                {
                    step.easing = value;
                }
            }
        }

        file += 1;
    }

    ctx.castle_hp_growth.clear();

    let Some(bytes) = open_asset_stream(ctx, b"CC_Castle_growth.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut line = Cell { at: 0, len: 0 };

    read_asset_stream_line(stm, &mut line);

    while read_csv_row(stm) {
        if read_cell_stream(stm, 0).is_empty() {
            break;
        }

        ctx.castle_hp_growth.push(CannonGrowthStep::default());

        let value = read_csv_cell(stm, 0) as i32;

        if let Some(step) = ctx.castle_hp_growth.last_mut() {
            step.lv2 = value;
        }

        let value = read_csv_cell(stm, 1) as i32;

        if let Some(step) = ctx.castle_hp_growth.last_mut() {
            step.value1 = value;
        }

        let value = read_csv_cell(stm, 2) as i32;

        if let Some(step) = ctx.castle_hp_growth.last_mut() {
            step.value2 = value;
        }

        let value = read_csv_cell(stm, 3) as i32;

        if let Some(step) = ctx.castle_hp_growth.last_mut() {
            step.easing = value;
        }
    }

    Ok(())
}
