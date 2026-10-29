use crate::Fault;

use super::{AppContext, load_fixed_lineup_csv, load_stage_hints_csv};

pub fn load_lineup_csvs(ctx: &mut AppContext) -> Result<(), Fault> {
    load_fixed_lineup_csv(ctx)?;
    load_stage_hints_csv(ctx)
}
