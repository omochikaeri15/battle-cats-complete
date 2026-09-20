use crate::Fault;

use super::{AppContext, AssetStream, GamatotoBonus, open_asset_stream, parse_gamatoto_bonus};

pub fn load_gamatoto_bonus_csv(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"Gamatoto_Bonus.csv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut parsed = GamatotoBonus::default();

    parse_gamatoto_bonus(&mut parsed, stm);
    ctx.gamatoto_bonus = parsed;

    Ok(())
}
