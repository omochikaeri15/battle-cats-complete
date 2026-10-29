use crate::Fault;

use super::{
    AppContext, AssetStream, OrbEffectRow, open_asset_stream, parse_orb_effect_row, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbEffectStore {
    pub rows: Vec<OrbEffectRow>,
    pub counts: [i32; 4],
}

const KIND_ORDER: [i32; 4] = [2, 3, 1, 0];

pub fn load_orb_effect_csv(ctx: &mut AppContext, store: usize) -> Result<(), Fault> {
    if let Some(effects) = ctx.orb_effects.get_mut(store) {
        effects.rows.clear();
        effects.counts = [0; 4];
    }

    if let Some(bytes) = open_asset_stream(ctx, b"equipment_EffectAbility.csv", 0, 0)? {
        let stm = &mut AssetStream::new(&bytes, b'\n');
        let mut row = OrbEffectRow::default();
        let mut index = 0i32;

        while read_csv_row(stm) {
            parse_orb_effect_row(&mut row, stm, index.wrapping_add(0x38e));

            if let Some(effects) = ctx.orb_effects.get_mut(store) {
                effects.rows.push(row.clone());

                let count = effects
                    .counts
                    .get_mut(row.kind as usize)
                    .ok_or(Fault::index_out_of_range(row.kind as i64, 4))?;

                *count = count.wrapping_add(1);
            }

            index = index.wrapping_add(1);
        }
    }

    if let Some(effects) = ctx.orb_effects.get_mut(store) {
        effects
            .rows
            .sort_by_key(|row| (KIND_ORDER.get(row.kind as usize).copied().unwrap_or(0), row.serial));
    }

    Ok(())
}
