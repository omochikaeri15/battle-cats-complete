use super::AppContext;

#[derive(Clone, Copy, Default)]
pub struct WebPopupEntry {
    pub kind: i32,
    pub map: i32,
    pub stage: i32,
}

pub fn web_popup_stage_match(ctx: &mut AppContext, map: i32, stage: i32) -> bool {
    if ctx.web_popup_shown.iter().any(|shown| shown[0] == map && shown[1] == stage) {
        return false;
    }

    for entry in &ctx.web_popup_entries {
        if entry.stage == stage && entry.map == map && entry.kind == 3 {
            ctx.web_popup_shown.push([map, stage]);

            return true;
        }
    }

    false
}
