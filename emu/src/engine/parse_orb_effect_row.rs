use super::{AssetStream, read_csv_cell};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct OrbEffectRow {
    pub id: i32,
    pub kind: i32,
    pub serial: i32,
    pub targets: [i32; 3],
    pub effect_id: i32,
    pub label_id: i32,
    pub amount: i32,
}

const LABEL_BY_KIND: [i32; 4] = [0x113, 0x113, 1, 0x162];

pub fn parse_orb_effect_row(row: &mut OrbEffectRow, stm: &mut AssetStream<'_>, id: i32) {
    row.targets[0] = -1;
    row.targets[1] = -1;
    row.targets[2] = -1;
    row.id = id;
    row.kind = read_csv_cell(stm, 0) as i32;

    let slot = if (row.kind as u32) < 2 {
        1
    } else if row.kind == 2 {
        2
    } else {
        0
    };

    row.targets[slot] = read_csv_cell(stm, 1) as i32;
    row.serial = read_csv_cell(stm, 2) as i32;

    let value = read_csv_cell(stm, 3) as i32;

    row.amount = if value == -1 { 0x32 } else { value };

    let value = read_csv_cell(stm, 4) as i32;

    if value != -1 {
        row.effect_id = value;
    } else if (row.kind as u32) < 2 {
        row.effect_id = 0x112;
    } else if row.kind == 2 {
        row.effect_id = 0;
        row.label_id = 1;

        return;
    } else if row.kind == 3 {
        row.effect_id = 0x160;
        row.label_id = 0x162;

        return;
    }

    if row.kind as u32 <= 3 {
        if let Some(label) = LABEL_BY_KIND.get(row.kind as u32 as usize) {
            row.label_id = *label;
        }
    }
}
