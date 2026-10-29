use crate::Fault;

use super::{
    AppContext, json_parse_object_document, json_source_from_string, open_asset_stream,
    parse_lineup_preset,
};

pub fn load_lineup_preset(ctx: &mut AppContext, name: &[u8]) -> Result<(), Fault> {
    ctx.fixed_lineup_store.units.clear();
    ctx.fixed_lineup_store.ability_levels.clear();
    ctx.set_i32_at(AppContext::LINEUP_BASE_LEVEL, -1)?;
    ctx.set_i32_at(AppContext::LINEUP_CANNON_TYPE, -1)?;
    ctx.set_i32_at(AppContext::LINEUP_CANNON_LEVEL, -1)?;
    ctx.fixed_lineup_store.treasure_flags.clear();

    if let Some(bytes) = open_asset_stream(ctx, name, 0, 0)? {
        let source = json_source_from_string(&bytes);
        let root = json_parse_object_document(Some(source))?;

        parse_lineup_preset(ctx, root.as_ref())?;
    }

    Ok(())
}
