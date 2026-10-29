use super::{AppContext, get_localizable_dictionary, localizable_get};

pub fn query_localizable(ctx: &AppContext, key: &[u8]) -> Vec<u8> {
    localizable_get(get_localizable_dictionary(ctx), key)
}
