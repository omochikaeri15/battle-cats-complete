use crate::fault::Fault;

use super::CastleRow;

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
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AppContext {
    pub fn new() -> Self {
        Self { raw: vec![0u8; SIZE].into_boxed_slice(), stage_enemies: Vec::new(), enemy_castle: Vec::new() }
    }

    pub fn entity_field(team: i32, idx: i32, field: usize) -> usize {
        (team as usize)
            .wrapping_mul(TEAM_STRIDE)
            .wrapping_add((idx as usize).wrapping_mul(ENTITY_STRIDE))
            .wrapping_add(field)
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

    pub fn u8_at(&self, off: usize) -> Result<u8, Fault> {
        self.raw
            .get(off)
            .copied()
            .ok_or(Fault::IndexOutOfRange { site: SITE, index: off as i64, limit: SIZE as i64 })
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
