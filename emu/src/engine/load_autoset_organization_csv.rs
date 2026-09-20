use std::collections::BTreeMap;

use crate::Fault;

use super::{AppContext, AssetStream, open_asset_stream, read_csv_cell, read_csv_row};

pub fn load_autoset_organization_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.autoset_organizations.clear();

    let Some(bytes) = open_asset_stream(ctx, b"autoset_organization.csv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut counts: BTreeMap<i32, i32> = BTreeMap::new();

    while read_csv_row(stm) {
        let key = read_csv_cell(stm, 0) as i32;
        let count = *counts.entry(key).or_insert(0);

        ctx.autoset_organizations.entry(key).or_default();

        if (count as u32) < 0xa {
            let value = read_csv_cell(stm, 1) as i32;
            let slot = *counts.entry(key).or_insert(0);

            if let Some(triple) = ctx
                .autoset_organizations
                .entry(key)
                .or_default()
                .get_mut(slot as i64 as usize)
            {
                triple[0] = value;
            }

            let value = read_csv_cell(stm, 2) as i32;
            let slot = *counts.entry(key).or_insert(0);

            if let Some(triple) = ctx
                .autoset_organizations
                .entry(key)
                .or_default()
                .get_mut(slot as i64 as usize)
            {
                triple[1] = value;
            }

            let value = read_csv_cell(stm, 3) as i32;
            let slot = *counts.entry(key).or_insert(0);

            if let Some(triple) = ctx
                .autoset_organizations
                .entry(key)
                .or_default()
                .get_mut(slot as i64 as usize)
            {
                triple[2] = value;
            }

            *counts.entry(key).or_insert(0) += 1;
        }
    }

    Ok(())
}
