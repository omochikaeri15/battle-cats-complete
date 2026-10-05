use crate::engine::{AppContext, ENEMY_ROW_CAPACITY, ENEMY_ROWS, EX_MAPS, LABEL_SPARES, Limits, NEG5_MAPS, TALENT_GROUPS, UNIT_CAPACITY, UNITS};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostLimits {
    pub units: usize,
    pub enemy_rows: usize,
    pub talent_groups: usize,
    pub ex_maps: usize,
    pub neg5_maps: usize,
}

pub fn set_limits(ctx: &mut AppContext, host: HostLimits) -> HostLimits {
    let applied = HostLimits {
        units: host.units.clamp(UNITS, UNIT_CAPACITY),
        enemy_rows: host.enemy_rows.clamp(ENEMY_ROWS, ENEMY_ROW_CAPACITY),
        talent_groups: host.talent_groups.max(TALENT_GROUPS),
        ex_maps: host.ex_maps.max(EX_MAPS),
        neg5_maps: host.neg5_maps.max(NEG5_MAPS),
    };

    ctx.limits = Limits {
        units: applied.units as i32,
        enemy_rows: applied.enemy_rows as i32,
        talent_groups: applied.talent_groups as i32,
        ex_maps: applied.ex_maps as i32,
        neg5_maps: applied.neg5_maps as i32,
    };

    ctx.label_texts.resize(applied.units + LABEL_SPARES, None);

    applied
}
