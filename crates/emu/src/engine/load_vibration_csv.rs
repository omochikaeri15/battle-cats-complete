use std::collections::BTreeMap;

use crate::{Fault, ops};

use super::{
    AppContext, AssetStream, cell_is_int, get_setting_double, open_asset_stream, read_csv_cell,
    read_csv_cell_double, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct VibrationStep {
    pub strength: f64,
    pub seconds: f64,
    pub extra: f64,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct VibrationStore {
    pub steps: BTreeMap<i32, BTreeMap<i32, VibrationStep>>,
    pub power: f64,
    pub time: f64,
    pub tail: f64,
}

pub fn load_vibration_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.vibration.steps.clear();

    if let Some(bytes) = open_asset_stream(ctx, b"Vibration.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');

        while read_csv_row(stm) {
            if !cell_is_int(stm, 0) {
                break;
            }

            let id = read_csv_cell(stm, 0) as i32;
            let kind = read_csv_cell(stm, 1) as i32;

            if kind != -1 && kind != 1 {
                continue;
            }

            let value = read_csv_cell_double(stm, 2);
            let group = ops::div_2(id).wrapping_add(1);
            let slot = id.wrapping_sub(ops::div_2(id).wrapping_mul(2));

            ctx.vibration
                .steps
                .entry(group)
                .or_default()
                .entry(slot)
                .or_default()
                .strength = value;

            let value = read_csv_cell_double(stm, 3) / 1000.0;

            ctx.vibration
                .steps
                .entry(group)
                .or_default()
                .entry(slot)
                .or_default()
                .seconds = value;

            let value = read_csv_cell_double(stm, 4);

            ctx.vibration
                .steps
                .entry(group)
                .or_default()
                .entry(slot)
                .or_default()
                .extra = value;
        }
    }

    ctx.vibration.power =
        get_setting_double(&ctx.settings, b"option_vibration_power_android", 0.0)?;
    ctx.vibration.time =
        get_setting_double(&ctx.settings, b"option_vibration_time_android", 0.0)? / 1000.0;
    ctx.vibration.tail = -1.0;

    Ok(())
}
