use crate::Fault;

use super::{
    AppContext, AssetStream, OrbEffectRow, open_asset_stream, parse_orb_effect_row, read_csv_row,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbEffectStore {
    pub rows: Vec<OrbEffectRow>,
    pub counts: [i32; 4],
}

pub fn load_orb_effect_csv(ctx: &mut AppContext, store: usize) -> Result<(), Fault> {
    if let Some(effects) = ctx.orb_effects.get_mut(store) {
        effects.rows.clear();
        effects.counts = [0; 4];
    }

    let Some(bytes) = open_asset_stream(ctx, b"equipment_EffectAbility.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut row = OrbEffectRow::default();
    let mut index = 0i32;

    while read_csv_row(stm) {
        parse_orb_effect_row(&mut row, stm, index.wrapping_add(0x38e));

        if let Some(effects) = ctx.orb_effects.get_mut(store) {
            effects.rows.push(row.clone());

            if let Some(count) = effects.counts.get_mut(row.kind as usize) {
                *count = count.wrapping_add(1);
            }
        }

        index = index.wrapping_add(1);
    }

    Ok(())
}
