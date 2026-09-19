use crate::Fault;

use super::AppContext;

#[derive(Clone, Copy, Default)]
pub struct SheetImage {
    pub width: i32,
    pub height: i32,
    pub whole: u8,
}

pub trait AssetSource {
    fn open(&mut self, name: &[u8], packed: u8, encrypted: u8) -> Option<Vec<u8>>;
    fn load_png(&mut self, name: &[u8]) -> Option<SheetImage>;
    fn upload(&mut self, name: &[u8]);
    fn pack_entry(&mut self, name: &[u8]) -> Vec<u8>;
}

pub fn open_asset_stream(ctx: &mut AppContext, name: &[u8], packed: u8, encrypted: u8) -> Result<Option<Vec<u8>>, Fault> {
    Ok(ctx.assets().ok_or(Fault::HostMissing { site: "open_asset_stream" })?.open(name, packed, encrypted))
}
