use crate::Fault;

use super::{AppContext, draw_powerup_bar};

pub fn game_update_lambda_2(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_powerup_bar(ctx)
}
