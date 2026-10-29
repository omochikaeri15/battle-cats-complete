#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MatatabiRow {
    pub gatya_id: i32,
    pub group: i32,
    pub sort: i32,
    pub require: i32,
    pub seed: u8,
    pub text: Vec<u8>,
    pub guide_text: Vec<u8>,
}

pub fn matatabi_row_init(
    row: &mut MatatabiRow,
    gatya_id: i32,
    seed: i32,
    group: i32,
    sort: i32,
    require: i32,
    guide_text: &[u8],
    text: &[u8],
) {
    row.gatya_id = gatya_id;
    row.group = group;
    row.sort = sort;
    row.require = require;
    row.seed = u8::from(seed != 0);
    row.text = text.to_vec();
    row.guide_text = guide_text.to_vec();
}
