use std::collections::BTreeMap;

use crate::Fault;

use super::{CastleRow, FixedLineupStore, MapData, SoundManager};

pub const SIZE: usize = 0x500000;

pub const ENTITY_BASE: usize = 0x838f8;
pub const ENTITY_STRIDE: usize = 0x3e8;
pub const TEAM_STRIDE: usize = 0xc738;
pub const SLOTS_PER_TEAM: i32 = 51;

pub const STAGE_ENEMY_COLUMNS: usize = 14;

const RNG_STATE: usize = 0x46f790;
const SITE: &str = "app_context";

const _: () = assert!(RNG_STATE + 4 <= SIZE);

pub struct AppContext {
    raw: Box<[u8]>,
    pub stage_enemies: Vec<[i32; STAGE_ENEMY_COLUMNS]>,
    pub enemy_castle: Vec<CastleRow>,
    pub fixed_lineup_store: FixedLineupStore,
    pub map_data: BTreeMap<i32, MapData>,
    pub map_data_ids: BTreeMap<i32, Vec<i32>>,
    pub map_stage_sets: BTreeMap<i32, Vec<i32>>,
    pub talent_definitions: BTreeMap<i32, [i32; 0x71]>,
    pub talent_levels: BTreeMap<i32, BTreeMap<i32, i32>>,
    pub outbreak_active: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub outbreak_cleared: BTreeMap<i32, BTreeMap<i32, bool>>,
    pub cleared_map_ids: Vec<i32>,
    pub stages_cleared_cache: BTreeMap<i32, [i16; 4]>,
    pub stages_cleared_neg24: Vec<Vec<i8>>,
    pub stages_cleared_neg23: Vec<Vec<i8>>,
    pub stages_cleared_neg22: Vec<Vec<i8>>,
    pub stages_cleared_neg20: Vec<i8>,
    pub stages_cleared_neg18: Vec<i8>,
    pub stages_cleared_neg17: Vec<i8>,
    pub stages_cleared_neg16: Vec<i8>,
    pub stages_cleared_neg11: Vec<i8>,
    pub stages_cleared_neg10: Vec<i32>,
    pub stages_cleared_neg9: Vec<i32>,
    pub stages_cleared_neg4: Vec<i32>,
    pub stage_record_cache: BTreeMap<i32, BTreeMap<i32, [i16; 4]>>,
    pub stage_record_neg26: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg24: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg23: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg22: Vec<Vec<Vec<i16>>>,
    pub stage_record_neg20: Vec<i16>,
    pub stage_record_neg19: Vec<i16>,
    pub stage_record_neg18: Vec<i16>,
    pub stage_record_neg17: Vec<i16>,
    pub stage_record_neg16: Vec<i16>,
    pub stage_record_neg11: Vec<i16>,
    pub stage_record_neg10: Vec<i32>,
    pub stage_record_neg9: Vec<i32>,
    pub stage_record_neg4: Vec<i32>,
    sound: Option<Box<dyn SoundManager>>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            raw: vec![0u8; SIZE].into_boxed_slice(),
            stage_enemies: Vec::new(),
            enemy_castle: Vec::new(),
            fixed_lineup_store: Default::default(),
            map_data: Default::default(),
            map_data_ids: Default::default(),
            map_stage_sets: Default::default(),
            talent_definitions: Default::default(),
            talent_levels: Default::default(),
            outbreak_active: Default::default(),
            outbreak_cleared: Default::default(),
            cleared_map_ids: Default::default(),
            stages_cleared_cache: Default::default(),
            stages_cleared_neg24: Default::default(),
            stages_cleared_neg23: Default::default(),
            stages_cleared_neg22: Default::default(),
            stages_cleared_neg20: Default::default(),
            stages_cleared_neg18: Default::default(),
            stages_cleared_neg17: Default::default(),
            stages_cleared_neg16: Default::default(),
            stages_cleared_neg11: Default::default(),
            stages_cleared_neg10: Default::default(),
            stages_cleared_neg9: Default::default(),
            stages_cleared_neg4: Default::default(),
            stage_record_cache: Default::default(),
            stage_record_neg26: Default::default(),
            stage_record_neg24: Default::default(),
            stage_record_neg23: Default::default(),
            stage_record_neg22: Default::default(),
            stage_record_neg20: Default::default(),
            stage_record_neg19: Default::default(),
            stage_record_neg18: Default::default(),
            stage_record_neg17: Default::default(),
            stage_record_neg16: Default::default(),
            stage_record_neg11: Default::default(),
            stage_record_neg10: Default::default(),
            stage_record_neg9: Default::default(),
            stage_record_neg4: Default::default(),
            sound: None,
        }
    }

    pub fn entity_field(team: i32, idx: i32, field: usize) -> usize {
        (team as usize)
            .wrapping_mul(TEAM_STRIDE)
            .wrapping_add((idx as usize).wrapping_mul(ENTITY_STRIDE))
            .wrapping_add(field)
    }

    pub fn sound(&mut self) -> Option<&mut (dyn SoundManager + 'static)> {
        self.sound.as_deref_mut()
    }

    pub fn set_sound(&mut self, sound: Box<dyn SoundManager>) {
        self.sound = Some(sound);
    }

    pub fn rng_state(&self) -> u32 {
        let mut word = [0u8; 4];
        word.copy_from_slice(&self.raw[RNG_STATE..RNG_STATE + 4]);

        u32::from_le_bytes(word)
    }

    pub fn set_rng_state(&mut self, state: u32) {
        self.raw[RNG_STATE..RNG_STATE + 4].copy_from_slice(&state.to_le_bytes());
    }

    pub fn zero(&mut self, off: usize, len: usize) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..len))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.fill(0);

        Ok(())
    }

    pub fn block_at<const N: usize>(&self, off: usize) -> Result<[u8; N], Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..N))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut block = [0u8; N];
        block.copy_from_slice(bytes);

        Ok(block)
    }

    pub fn set_block_at<const N: usize>(&mut self, off: usize, value: [u8; N]) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..N))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.copy_from_slice(&value);

        Ok(())
    }

    pub fn bytes_from(&self, off: usize) -> Result<&[u8], Fault> {
        self.raw.get(off..).ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn u8_at(&self, off: usize) -> Result<u8, Fault> {
        self.raw
            .get(off)
            .copied()
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn i8_at(&self, off: usize) -> Result<i8, Fault> {
        self.raw
            .get(off)
            .map(|byte| *byte as i8)
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
    }

    pub fn i16_at(&self, off: usize) -> Result<i16, Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..2))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut word = [0u8; 2];
        word.copy_from_slice(bytes);

        Ok(i16::from_le_bytes(word))
    }

    pub fn i32_at(&self, off: usize) -> Result<i32, Fault> {
        let bytes = self
            .raw
            .get(off..)
            .and_then(|rest| rest.get(..4))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        let mut word = [0u8; 4];
        word.copy_from_slice(bytes);

        Ok(i32::from_le_bytes(word))
    }

    pub fn set_i32_at(&mut self, off: usize, value: i32) -> Result<(), Fault> {
        let bytes = self
            .raw
            .get_mut(off..)
            .and_then(|rest| rest.get_mut(..4))
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })?;

        bytes.copy_from_slice(&value.to_le_bytes());

        Ok(())
    }
}
