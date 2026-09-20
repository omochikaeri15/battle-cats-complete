use crate::engine::{FormatArg, MetaHost};

pub struct InertMeta;

impl MetaHost for InertMeta {
    fn analytics_event(&mut self, _event: i32, _first: i32, _second: i32, _third: i32, _fourth: i32) {}
    fn mission_progress(&mut self, _kind: i32, _target: i32, _amount: i32, _first: i32, _second: i32) {}
    fn mission_refresh(&mut self) {}
    fn check_medals(&mut self, _kind: i32) {}
    fn request_save(&mut self) {}
    fn set_item_count(&mut self, _item: i32, _count: i32, _flag: u8) {}
    fn analytics_record(&mut self, _code: i32, _value: i32, _flag: i32, _params: &[(&[u8], FormatArg<'_>)]) {}
    fn analytics_named(&mut self, _code: i32, _name: &[u8], _detail: &[u8]) {}
    fn bc_log(&mut self, _name: &[u8]) {}
    fn save_battle_snapshot(&mut self) {}
    fn drop_popup_text(&mut self, _item: i32, _first: u8, _amount: i32) -> Vec<u8> {
        Vec::new()
    }
    fn bonus_popup_text(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn labyrinth_unit_count(&mut self, _rarity: i32) -> i32 {
        0
    }
    fn breadcrumb(&mut self, _id: i32) {}
    fn analytics_params(&mut self, _event: i32, _value: i32, _params: &[(&[u8], FormatArg<'_>)]) {}
    fn enigma_opened(&mut self, _map: i32, _time: f64) {}
    fn mission_progress_list(&mut self, _kind: i32, _targets: &[i32], _amount: i32) {}
    fn mission_mark(&mut self, _kind: i32, _target: i32) {}
    fn breadcrumb_with(&mut self, _id: i32, _params: &[(&[u8], &[u8])]) {}
    fn shop_offer_start(&mut self, _map: i32) {}
    fn resource_log(&mut self, _action: &[u8], _item: &[u8], _amount: i32) {}
    fn load_battle_snapshot(&mut self, _mode: i32) {}
}
