use crate::Fault;

use super::{
    AppContext, AssetStream, open_asset_stream, parse_realms_rng_table, string_format_int,
};

pub fn load_realms_rng_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.realms_rng_tables.clear();

    let name = string_format_int(ctx, b"demon_lottery_DM%03d.csv", 0)?;
    let Some(bytes) = open_asset_stream(ctx, &name, 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let table = parse_realms_rng_table(stm);

    ctx.realms_rng_tables.insert(0, table);

    Ok(())
}
