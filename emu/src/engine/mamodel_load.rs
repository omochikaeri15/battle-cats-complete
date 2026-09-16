use super::{read_asset_stream_line, read_csv_cell, read_csv_row, AssetStream, Cell};

pub const PART_STRIDE: usize = 0xb0;

const DEFAULT_SCALE_UNIT: i32 = 0x64;
const DEFAULT_ANGLE_UNIT: i32 = 0x168;
const DEFAULT_OPACITY_UNIT: i32 = 0xff;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct MamodelAnchor {
    pub part: i32,
    pub part_idx: i32,
    pub unused_1: i32,
    pub x: i32,
    pub y: i32,
    pub unused_4: i32,
    pub unused_5: i32,
}

#[derive(Default)]
pub struct Mamodel {
    pub parts: Vec<[u8; PART_STRIDE]>,
    pub part_scratch: Vec<i32>,
    pub part_order: Vec<i32>,
    pub scale_unit: i32,
    pub angle_unit: i32,
    pub opacity_unit: i32,
    pub mirror: i32,
    pub anchors: Vec<MamodelAnchor>,
    pub anchor_count: i32,
    pub path: String,
}

pub fn mamodel_load(model: &mut Mamodel, path: &str, stm: Option<&mut AssetStream<'_>>) -> bool {
    model.parts.clear();
    model.part_scratch.clear();
    model.part_order.clear();
    model.anchors.clear();
    model.path.clear();
    model.mirror = 0;
    model.path.push_str(path);

    let Some(stm) = stm else {
        return false;
    };

    let mut discarded = Cell { at: 0, len: 0 };
    read_asset_stream_line(stm, &mut discarded);

    read_csv_row(stm);
    let version = read_csv_cell(stm, 0) as i32;

    read_csv_row(stm);
    let part_count = read_csv_cell(stm, 0) as i32 as i64 as usize;

    model.parts.resize(part_count, [0u8; PART_STRIDE]);
    model.part_scratch.resize(part_count, 0);
    model.part_order.resize(part_count, 0);

    for row in 0..model.parts.len() {
        read_csv_row(stm);

        let part = &mut model.parts[row];

        part[0x1c..0x20].copy_from_slice(&(read_csv_cell(stm, 0) as i32).to_le_bytes());
        part[0x24..0x28].copy_from_slice(&(read_csv_cell(stm, 1) as i32).to_le_bytes());
        part[0x2c..0x30].copy_from_slice(&(read_csv_cell(stm, 2) as i32).to_le_bytes());
        part[0x34..0x38].copy_from_slice(&(read_csv_cell(stm, 3) as i32).to_le_bytes());
        part[0x3c..0x40].copy_from_slice(&(read_csv_cell(stm, 4) as i32).to_le_bytes());
        part[0x40..0x44].copy_from_slice(&(read_csv_cell(stm, 5) as i32).to_le_bytes());
        part[0x4c..0x50].copy_from_slice(&(read_csv_cell(stm, 6) as i32).to_le_bytes());
        part[0x50..0x54].copy_from_slice(&(read_csv_cell(stm, 7) as i32).to_le_bytes());
        part[0x5c..0x60].copy_from_slice(&(read_csv_cell(stm, 8) as i32).to_le_bytes());
        part[0x68..0x6c].copy_from_slice(&(read_csv_cell(stm, 9) as i32).to_le_bytes());
        part[0x74..0x78].copy_from_slice(&(read_csv_cell(stm, 0xa) as i32).to_le_bytes());
        part[0x7c..0x80].copy_from_slice(&(read_csv_cell(stm, 0xb) as i32).to_le_bytes());

        let mut glow = 0;

        if version >= 2 {
            glow = read_csv_cell(stm, 0xc) as i32;
        }

        part[0x8c..0x90].copy_from_slice(&glow.to_le_bytes());
    }

    if version <= 0 {
        model.scale_unit = DEFAULT_SCALE_UNIT;
        model.angle_unit = DEFAULT_ANGLE_UNIT;
        model.opacity_unit = DEFAULT_OPACITY_UNIT;
        model.anchor_count = 0;

        return true;
    }

    read_csv_row(stm);
    model.scale_unit = read_csv_cell(stm, 0) as i32;
    model.angle_unit = read_csv_cell(stm, 1) as i32;
    model.opacity_unit = read_csv_cell(stm, 2) as i32;

    if (version as u32) < 3 {
        model.anchor_count = 0;

        return true;
    }

    read_csv_row(stm);
    let anchor_count = read_csv_cell(stm, 0) as i32;
    model.anchor_count = anchor_count;
    model.anchors.resize(anchor_count as i64 as usize, MamodelAnchor::default());

    for row in 0..model.anchors.len() {
        read_csv_row(stm);

        let part_idx = read_csv_cell(stm, 0) as i32;
        let anchor = &mut model.anchors[row];

        anchor.part_idx = part_idx;
        anchor.part = part_idx;
        anchor.unused_1 = read_csv_cell(stm, 1) as i32;
        anchor.x = read_csv_cell(stm, 2) as i32;
        anchor.y = read_csv_cell(stm, 3) as i32;
        anchor.unused_4 = read_csv_cell(stm, 4) as i32;
        anchor.unused_5 = read_csv_cell(stm, 5) as i32;
    }

    true
}
