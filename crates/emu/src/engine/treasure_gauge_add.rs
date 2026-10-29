use std::collections::BTreeMap;

use super::{AppContext, unlock_popup_is_unlocked};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct TreasureGauge {
    pub value: i32,
    pub max_value: i32,
    pub gains: BTreeMap<i32, i32>,
    pub rank: i32,
}

pub fn treasure_gauge_add(ctx: &mut AppContext, kind: i32, amount: i32) {
    if amount == 0 {
        return;
    }

    if !unlock_popup_is_unlocked(ctx, 0xa1) {
        return;
    }

    let gauge = &mut ctx.treasure_gauge;
    let value;

    if kind != 1 {
        let gain = *gauge.gains.entry(kind).or_insert(0);

        value = amount.wrapping_mul(gain).wrapping_add(gauge.value);
        gauge.value = value;
    } else {
        let rise = amount.wrapping_sub(gauge.rank);

        if amount <= gauge.rank {
            return;
        }

        let gain = *gauge.gains.entry(1).or_insert(0);

        value = rise.wrapping_mul(gain).wrapping_add(gauge.value);
        gauge.value = value;
        gauge.rank = amount;
    }

    if value > gauge.max_value {
        gauge.value = gauge.max_value;
    }
}
