use std::collections::BTreeMap;

use super::AppContext;

pub fn get_localizable_dictionary(ctx: &AppContext) -> &BTreeMap<Vec<u8>, Vec<u8>> {
    &ctx.localizable
}
