use crate::Fault;

use super::{AppContext, EventItemStore, load_point_rules_map, load_point_stage_settings};

pub fn load_point_files(ctx: &mut AppContext, store: &mut EventItemStore) -> Result<(), Fault> {
    load_point_stage_settings(ctx, store)?;

    load_point_rules_map(ctx, store)
}
