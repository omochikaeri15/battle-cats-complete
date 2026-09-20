use crate::Fault;

use super::{
    AppContext, load_autoset_ability_csv, load_autoset_exclusion_csv, load_autoset_group_csv,
    load_autoset_organization_csv, load_autoset_rating_csv,
};

pub fn load_autoset_lineup_files(ctx: &mut AppContext) -> Result<(), Fault> {
    load_autoset_rating_csv(ctx)?;
    load_autoset_organization_csv(ctx)?;
    load_autoset_ability_csv(ctx)?;
    load_autoset_exclusion_csv(ctx)?;
    load_autoset_group_csv(ctx)
}
