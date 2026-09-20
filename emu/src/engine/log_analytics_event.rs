use crate::Fault;

use super::{AppContext, FormatArg};

pub trait MetaHost {
    fn analytics_event(&mut self, event: i32, first: i32, second: i32, third: i32, fourth: i32);
    fn mission_progress(&mut self, kind: i32, target: i32, amount: i32, first: i32, second: i32);
    fn mission_refresh(&mut self);
    fn check_medals(&mut self, kind: i32);
    fn request_save(&mut self);
    fn set_item_count(&mut self, item: i32, count: i32, flag: u8);
    fn analytics_record(
        &mut self,
        code: i32,
        value: i32,
        flag: i32,
        params: &[(&[u8], FormatArg<'_>)],
    );
    fn analytics_named(&mut self, code: i32, name: &[u8], detail: &[u8]);
    fn bc_log(&mut self, name: &[u8]);
    fn save_battle_snapshot(&mut self);
    fn drop_popup_text(&mut self, item: i32, first: u8, amount: i32) -> Vec<u8>;
    fn bonus_popup_text(&mut self) -> Vec<u8>;
    fn labyrinth_unit_count(&mut self, rarity: i32) -> i32;
    fn breadcrumb(&mut self, id: i32);
    fn analytics_params(&mut self, event: i32, value: i32, params: &[(&[u8], FormatArg<'_>)]);
    fn enigma_opened(&mut self, map: i32, time: f64);
    fn mission_progress_list(&mut self, kind: i32, targets: &[i32], amount: i32);
    fn mission_mark(&mut self, kind: i32, target: i32);
    fn breadcrumb_with(&mut self, id: i32, params: &[(&[u8], &[u8])]);
    fn shop_offer_start(&mut self, map: i32);
    fn resource_log(&mut self, action: &[u8], item: &[u8], amount: i32);
    fn load_battle_snapshot(&mut self, mode: i32);
}

pub fn log_analytics_event(
    ctx: &mut AppContext,
    event: i32,
    first: i32,
    second: i32,
    third: i32,
    fourth: i32,
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .analytics_event(event, first, second, third, fourth);

    Ok(())
}
