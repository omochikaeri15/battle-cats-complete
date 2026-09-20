use std::collections::BTreeMap;

use crate::Fault;

use super::AppContext;

pub type DialogEventHandler = fn(&mut AppContext, u64, i32, i32) -> Result<(), Fault>;
pub type DialogDrawHandler = fn(&mut AppContext) -> Result<(), Fault>;
pub type DialogUpdateHandler = fn(&mut AppContext, u64) -> Result<(), Fault>;

#[derive(Clone, Default)]
pub struct Dialog {
    pub kind: i32,
    pub text: Vec<u8>,
    pub first_button: i32,
    pub second_button: i32,
    pub flags: i32,
    pub state: i32,
    pub button: i32,
    pub back_button: i32,
    pub on_event: Option<DialogEventHandler>,
    pub on_draw: Option<DialogDrawHandler>,
    pub on_update: Option<DialogUpdateHandler>,
}

#[derive(Clone, Default)]
pub struct DialogManager {
    pub next_id: u64,
    pub objects: BTreeMap<u64, Dialog>,
    pub active: Vec<u64>,
    pub queued: Vec<u64>,
}

pub trait UiHost {
    fn dialog_create(
        &mut self,
        dialog: u64,
        kind: i32,
        text: &[u8],
        first_button: i32,
        second_button: i32,
        flags: i32,
    );
    fn message_set(&mut self, layer: i32, text: &[u8], size: i32, width: i32);
    fn message_clear(&mut self, layer: i32);
    fn option_window_build(&mut self, kind: i32);
    fn option_window_draw(&mut self);
    fn set_window_ratio(&mut self, ratio: f32);
    fn clear_layout_latch(&mut self);
    fn set_layout_latch(&mut self);
    fn viewport_resized(&mut self);
    fn dialog_origin(&mut self, dialog: u64) -> (i32, i32);
    fn dialog_draw(&mut self, dialog: u64, layer: i32);
    fn page_list_layout(
        &mut self,
        list: usize,
        count: i32,
        rows: i32,
        center: f32,
        spacing: f32,
        width: f32,
    );
}

pub fn dialog_new(
    ctx: &mut AppContext,
    kind: i32,
    text: &[u8],
    first_button: i32,
    second_button: i32,
    flags: i32,
    on_event: Option<DialogEventHandler>,
) -> Result<u64, Fault> {
    ctx.dialogs.next_id = ctx.dialogs.next_id.wrapping_add(1);

    let dialog = ctx.dialogs.next_id;

    ctx.dialogs.objects.insert(
        dialog,
        Dialog {
            kind,
            text: text.to_vec(),
            first_button,
            second_button,
            flags,
            state: 0,
            button: -1,
            back_button: -3,
            on_event,
            on_draw: None,
            on_update: None,
        },
    );

    ctx.ui()
        .ok_or(Fault::HostMissing { site: "dialog_new" })?
        .dialog_create(dialog, kind, text, first_button, second_button, flags);

    Ok(dialog)
}
