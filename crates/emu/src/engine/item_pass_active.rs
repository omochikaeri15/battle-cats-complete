use crate::Fault;

use super::{AppContext, obf_value_read};

pub fn item_pass_active(ctx: &AppContext, item: i32) -> Result<bool, Fault> {
    if item == 0 {
        return Ok(true);
    }

    if obf_value_read(&ctx.club_pass_state).wrapping_sub(1) > 1 {
        return Ok(false);
    }

    for row in ctx.officers_club_rows.values() {
        let items = row.items.clone();
        let lower = row.lower_limit;
        let upper = row.upper_limit;
        let rank = ctx.club_user_rank;
        let mut found = 3i32;

        if (lower == 0 || rank >= lower) && (upper == 0 || rank <= upper) && ctx.club_owned.contains_key(&row.club_item_id) {
            found = 0;

            for entry in &items {
                if entry[0] == 4 && entry[1] == item {
                    found = 1;
                    break;
                }
            }
        }

        if found != 3 && found != 0 {
            return Ok(true);
        }
    }

    Ok(false)
}
