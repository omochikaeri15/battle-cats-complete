use crate::engine::UiHost;

pub struct InertUi;

impl UiHost for InertUi {
    fn dialog_create(
        &mut self,
        _dialog: u64,
        _kind: i32,
        _text: &[u8],
        _first_button: i32,
        _second_button: i32,
        _flags: i32,
    ) {
    }
    fn message_set(&mut self, _layer: i32, _text: &[u8], _size: i32, _width: i32) {}
    fn message_clear(&mut self, _layer: i32) {}
    fn option_window_build(&mut self, _kind: i32) {}
    fn option_window_draw(&mut self) {}
    fn set_window_ratio(&mut self, _ratio: f32) {}
    fn clear_layout_latch(&mut self) {}
    fn set_layout_latch(&mut self) {}
    fn viewport_resized(&mut self) {}
    fn dialog_origin(&mut self, _dialog: u64) -> (i32, i32) {
        (0, 0)
    }
    fn dialog_draw(&mut self, _dialog: u64, _layer: i32) {}
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
