use crate::engine::SceneHost;

pub struct InertScene;

impl SceneHost for InertScene {
    fn battle_check_login_bonus(&mut self) {}
    fn scene_setup(&mut self, _scene: i32) {}
    fn battle_exit_cleanup(&mut self, _ex: u8) {}
    fn leadership_refund(&mut self) {}
    fn map_return_reset(&mut self) {}
    fn map_return_flags(&mut self) {}
    fn map_screen_init(&mut self, _page: i32) {}
    fn map_menu_build(&mut self, _flag: i32) {}
    fn map_background_pick(&mut self) {}
    fn scene_background_setup(&mut self) {}
    fn scene_base_init(&mut self) {}
    fn map_ui_reset(&mut self) {}
    fn map_bgm_start(&mut self, _kind: i32) {}
    fn leadership_return_begin(&mut self) {}
    fn collab_reward_ready(&mut self, _map: i32, _stage: i32) -> bool {
        false
    }
    fn collab_reward_dialog(&mut self, _map: i32, _stage: i32) {}
    fn unlock_popup_pending(&mut self) -> bool {
        false
    }
    fn loader_busy(&mut self) -> bool {
        false
    }
    fn fade_menu_dispatch(&mut self, _style: i32, _scene: i32) -> bool {
        false
    }
    fn fade_menu_prompt(&mut self) -> bool {
        false
    }
}
