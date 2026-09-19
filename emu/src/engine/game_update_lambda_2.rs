use crate::Fault;

use super::{draw_powerup_bar, AppContext};

pub fn game_update_lambda_2(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_powerup_bar(ctx)
}
