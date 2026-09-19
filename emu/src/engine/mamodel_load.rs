use std::{cell, rc::Rc};

use crate::Fault;

use super::{
    AppContext, AssetStream, Cell, Imgcut, open_asset_stream, read_asset_stream_line,
    read_csv_cell, read_csv_row,
};

pub const PART_STRIDE: usize = 0xb0;

pub type SheetTable = Rc<[cell::Cell<Option<Rc<Imgcut>>>]>;

const DEFAULT_SCALE_UNIT: i32 = 0x64;
const DEFAULT_ANGLE_UNIT: i32 = 0x168;
const DEFAULT_OPACITY_UNIT: i32 = 0xff;

#[derive(Clone, Copy)]
pub struct MamodelPart {
    raw: [u8; PART_STRIDE],
}

impl Default for MamodelPart {
    fn default() -> Self {
        Self {
            raw: [0; PART_STRIDE],
        }
    }
}

impl MamodelPart {
    pub fn from_raw(raw: [u8; PART_STRIDE]) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &[u8; PART_STRIDE] {
        &self.raw
    }

    pub fn i32_at(&self, off: usize) -> i32 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&self.raw[off..off + 4]);

        i32::from_le_bytes(word)
    }

    pub fn set_i32_at(&mut self, off: usize, value: i32) {
        self.raw[off..off + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn u8_at(&self, off: usize) -> u8 {
        self.raw[off]
    }

    pub fn set_u8_at(&mut self, off: usize, value: u8) {
        self.raw[off] = value;
    }

    pub fn set_f32_at(&mut self, off: usize, value: f32) {
        self.raw[off..off + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn f32_at(&self, off: usize) -> f32 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&self.raw[off..off + 4]);

        f32::from_le_bytes(word)
    }
}

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
    pub sheet: Option<Rc<Imgcut>>,
    pub sheet_table: SheetTable,
    pub single_sheet: u8,
    pub parts: Vec<MamodelPart>,
    pub draw_order: Vec<i32>,
    pub draw_z: Vec<i32>,
    pub scale_unit: i32,
    pub angle_unit: i32,
    pub opacity_unit: i32,
    pub mirror: i32,
    pub anchors: Vec<MamodelAnchor>,
    pub anchor_count: i32,
    pub path: Vec<u8>,
}

pub fn mamodel_load(ctx: &mut AppContext, model: &mut Mamodel, path: &[u8]) -> Result<bool, Fault> {
    model.parts.clear();
    model.draw_order.clear();
    model.draw_z.clear();
    model.anchors.clear();
    model.path.clear();
    model.mirror = 0;
    model.path.extend_from_slice(path);

    let Some(bytes) = open_asset_stream(ctx, path, 1, 0)? else {
        return Ok(false);
    };

    let mut stream = AssetStream::new(&bytes, b'\n');
    let stm = &mut stream;

    let mut discarded = Cell { at: 0, len: 0 };
    read_asset_stream_line(stm, &mut discarded);

    read_csv_row(stm);
    let version = read_csv_cell(stm, 0) as i32;

    read_csv_row(stm);
    let part_count = read_csv_cell(stm, 0) as i32 as i64 as usize;

    model.parts.resize(part_count, MamodelPart::default());
    model.draw_order.resize(part_count, 0);
    model.draw_z.resize(part_count, 0);

    for row in 0..model.parts.len() {
        read_csv_row(stm);

        let part = &mut model.parts[row];

        part.set_i32_at(0x1c, read_csv_cell(stm, 0) as i32);
        part.set_i32_at(0x24, read_csv_cell(stm, 1) as i32);
        part.set_i32_at(0x2c, read_csv_cell(stm, 2) as i32);
        part.set_i32_at(0x34, read_csv_cell(stm, 3) as i32);
        part.set_i32_at(0x3c, read_csv_cell(stm, 4) as i32);
        part.set_i32_at(0x40, read_csv_cell(stm, 5) as i32);
        part.set_i32_at(0x4c, read_csv_cell(stm, 6) as i32);
        part.set_i32_at(0x50, read_csv_cell(stm, 7) as i32);
        part.set_i32_at(0x5c, read_csv_cell(stm, 8) as i32);
        part.set_i32_at(0x68, read_csv_cell(stm, 9) as i32);
        part.set_i32_at(0x74, read_csv_cell(stm, 0xa) as i32);
        part.set_i32_at(0x7c, read_csv_cell(stm, 0xb) as i32);

        let mut glow = 0;

        if version >= 2 {
            glow = read_csv_cell(stm, 0xc) as i32;
        }

        part.set_i32_at(0x8c, glow);
    }

    if version <= 0 {
        model.scale_unit = DEFAULT_SCALE_UNIT;
        model.angle_unit = DEFAULT_ANGLE_UNIT;
        model.opacity_unit = DEFAULT_OPACITY_UNIT;
        model.anchor_count = 0;

        return Ok(true);
    }

    read_csv_row(stm);
    model.scale_unit = read_csv_cell(stm, 0) as i32;
    model.angle_unit = read_csv_cell(stm, 1) as i32;
    model.opacity_unit = read_csv_cell(stm, 2) as i32;

    if (version as u32) < 3 {
        model.anchor_count = 0;

        return Ok(true);
    }

    read_csv_row(stm);
    let anchor_count = read_csv_cell(stm, 0) as i32;
    model.anchor_count = anchor_count;
    model
        .anchors
        .resize(anchor_count as i64 as usize, MamodelAnchor::default());

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

    Ok(true)
}
