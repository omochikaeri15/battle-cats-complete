use crate::Fault;

use super::AppContext;

pub trait SceneHost {
    fn battle_check_login_bonus(&mut self);
    fn scene_setup(&mut self, scene: i32);
    fn battle_exit_cleanup(&mut self, ex: u8);
    fn leadership_refund(&mut self);
    fn map_return_reset(&mut self);
    fn map_return_flags(&mut self);
    fn map_screen_init(&mut self, page: i32);
    fn map_menu_build(&mut self, flag: i32);
    fn map_background_pick(&mut self);
    fn scene_background_setup(&mut self);
    fn scene_base_init(&mut self);
    fn map_ui_reset(&mut self);
    fn map_bgm_start(&mut self, kind: i32);
    fn leadership_return_begin(&mut self);
    fn collab_reward_ready(&mut self, map: i32, stage: i32) -> bool;
    fn collab_reward_dialog(&mut self, map: i32, stage: i32);
    fn unlock_popup_pending(&mut self) -> bool;
    fn loader_busy(&mut self) -> bool;
    fn fade_menu_dispatch(&mut self, style: i32, scene: i32) -> bool;
    fn fade_menu_prompt(&mut self) -> bool;
}

pub fn battle_check_login_bonus(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.scene_host().ok_or(Fault::HostMissing { site: "battle_check_login_bonus" })?.battle_check_login_bonus();

    Ok(())
}
