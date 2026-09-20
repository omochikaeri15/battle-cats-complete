use crate::Fault;

use super::{
    AppContext, load_orb_attribute_csv, load_orb_database_json, load_orb_grade_csv,
    load_orb_options_csv, load_orb_slots_csv, load_orb_text_tsv, load_orb_traits_tsv,
};

pub fn load_talent_orb_files(ctx: &mut AppContext) -> Result<(), Fault> {
    load_orb_database_json(ctx)?;
    load_orb_grade_csv(ctx)?;
    load_orb_attribute_csv(ctx)?;
    load_orb_traits_tsv(ctx)?;
    load_orb_text_tsv(ctx)?;
    load_orb_slots_csv(ctx)?;
    load_orb_options_csv(ctx)
}
