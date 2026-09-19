use crate::Fault;

use super::{
    AppContext, get_built_deck_stage_key, get_cannon_part_id, get_foundation_part_id, get_map_type,
    get_style_part_id, is_ex_option_target,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct LineupRecord {
    pub units: [u16; 10],
    pub forms: [u8; 10],
    pub cannon: u8,
    pub style: u8,
    pub foundation: u8,
}

pub fn record_stage_lineup(ctx: &mut AppContext) -> Result<(), Fault> {
    let kind = get_map_type(ctx, 0)?.wrapping_add(0x19) as u32;

    if kind <= 0x13 {
        if 0x87c11u32 >> kind & 1 != 0 {
            return Ok(());
        }

        if kind == 0x11 && !is_ex_option_target(ctx)? {
            return Ok(());
        }
    }

    let cannon = get_cannon_part_id(ctx)? as u8;
    let style = get_style_part_id(ctx)? as u8;
    let foundation = get_foundation_part_id(ctx)? as u8;
    let key = ctx.block_at::<2>(AppContext::BATTLE_DECK + 0x28)?;
    let mut lineup = LineupRecord {
        cannon,
        style,
        foundation,
        ..Default::default()
    };

    for slot in 0..10usize {
        let cell = ctx.block_at::<2>(AppContext::BATTLE_DECK + slot * 4)?;
        let unit = lineup.units.get_mut(slot).ok_or(Fault::IndexOutOfRange {
            site: "record_stage_lineup",
            index: slot as i64,
            limit: 10,
        })?;

        *unit = u16::from_le_bytes([cell[0] ^ key[0], cell[1] ^ key[1]]);

        let form = ctx.u8_at(AppContext::BUTTON_UNIT_FORMS + slot * 4)?;

        if let Some(entry) = lineup.forms.get_mut(slot) {
            *entry = form;
        }
    }

    let stage_key = get_built_deck_stage_key(ctx)?;

    for stages in ctx.lineup_stages.values_mut() {
        if let Some(at) = stages.iter().position(|value| *value == stage_key) {
            stages.remove(at);
        }
    }

    let existing = ctx
        .lineup_records
        .iter()
        .find(|(_, record)| **record == lineup)
        .map(|(id, _)| *id);

    if let Some(id) = existing {
        ctx.lineup_stages.entry(id).or_default().push(stage_key);

        return Ok(());
    }

    let count = ctx.lineup_stages.len() as i32;
    let mut id = count;
    let mut probe = 0i32;

    while probe < count {
        if ctx
            .lineup_stages
            .entry(probe as i16)
            .or_default()
            .is_empty()
        {
            id = probe;

            break;
        }

        probe += 1;
    }

    *ctx.lineup_records.entry(id as i16).or_default() = lineup;
    ctx.lineup_stages
        .entry(id as i16)
        .or_default()
        .push(stage_key);

    Ok(())
}
