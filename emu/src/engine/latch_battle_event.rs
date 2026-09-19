use std::collections::BTreeMap;

use super::BATTLE_EVENT_PRIORITIES;

#[derive(Default)]
pub struct BattleEventLatch {
    pub faction: i32,
    pub kind: i32,
    pub time: f64,
    pub strength: f64,
    pub gates: BTreeMap<i32, BTreeMap<i32, [f64; 3]>>,
}

pub fn latch_battle_event(latch: &mut BattleEventLatch, faction: i32, kind: i32) {
    let mut current = 0i32;
    let mut incoming = 0i32;

    if (kind.wrapping_sub(1) as u32) <= 2 {
        incoming = BATTLE_EVENT_PRIORITIES[kind.wrapping_sub(1) as u32 as usize];
    }

    let latched = latch.kind.wrapping_sub(1);

    if (latched as u32) <= 2 {
        current = BATTLE_EVENT_PRIORITIES[latched as u32 as usize];
    }

    if (incoming as u32) >= (current as u32) {
        let gate = latch.gates.entry(kind).or_default().entry(faction).or_default()[0];

        if gate != 0.0 || gate.is_nan() {
            latch.faction = faction;
            latch.kind = kind;
        }
    }
}
