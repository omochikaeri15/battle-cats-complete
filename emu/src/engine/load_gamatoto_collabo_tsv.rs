use crate::Fault;

use super::{
    AppContext, AssetStream, open_asset_stream, read_cell_stream, read_csv_cell,
    read_csv_cell_float, read_tsv_row,
};

#[derive(Clone, PartialEq, Debug)]
pub struct GamatotoCollabo {
    pub fukidashi_png: Vec<u8>,
    pub fukidashi_imgcut: Vec<u8>,
    pub member: Vec<u8>,
    pub member_text: Vec<u8>,
    pub scale: f32,
}

impl Default for GamatotoCollabo {
    fn default() -> Self {
        Self {
            fukidashi_png: Vec::new(),
            fukidashi_imgcut: Vec::new(),
            member: Vec::new(),
            member_text: Vec::new(),
            scale: 1.0,
        }
    }
}

pub fn load_gamatoto_collabo_tsv(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.gamatoto_collabo.clear();
    ctx.gamatoto_collabo_stages.clear();
    ctx.gamatoto_collabo.insert(-1, GamatotoCollabo::default());

    let Some(bytes) = open_asset_stream(ctx, b"GamatotoCollabo.tsv", 0, 0)? else {
        return Ok(());
    };

    let stm = &mut AssetStream::new(&bytes, b'\n');

    read_tsv_row(stm);

    while read_tsv_row(stm) {
        let stage = read_csv_cell(stm, 0) as i32;

        ctx.gamatoto_collabo_stages.push(stage);

        if ctx.gamatoto_collabo.contains_key(&stage) {
            continue;
        }

        let fukidashi_png = read_cell_stream(stm, 3).to_vec();
        let fukidashi_imgcut = read_cell_stream(stm, 4).to_vec();
        let member = read_cell_stream(stm, 1).to_vec();
        let member_text = read_cell_stream(stm, 2).to_vec();
        let scale = read_csv_cell_float(stm, 5);

        ctx.gamatoto_collabo.insert(
            stage,
            GamatotoCollabo {
                fukidashi_png,
                fukidashi_imgcut,
                member,
                member_text,
                scale,
            },
        );
    }

    Ok(())
}
