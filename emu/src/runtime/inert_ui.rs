use crate::engine::UiHost;

pub struct InertUi;

impl UiHost for InertUi {
    fn message_set(&mut self, _layer: i32, _text: &[u8], _size: i32, _width: i32) {}
    fn message_clear(&mut self, _layer: i32) {}
    fn option_window_build(&mut self, _kind: i32) {}
    fn option_window_draw(&mut self) {}
    fn title_option_window_set_touchable(&mut self, _touchable: u8) {}
    fn set_window_ratio(&mut self, _ratio: f32) {}
    fn clear_layout_latch(&mut self) {}
    fn set_layout_latch(&mut self) {}
    fn viewport_resized(&mut self) {}
    fn page_list_layout(
        &mut self,
        _list: usize,
        _count: i32,
        _rows: i32,
        _center: f32,
        _spacing: f32,
        _width: f32,
    ) {
    }
}
