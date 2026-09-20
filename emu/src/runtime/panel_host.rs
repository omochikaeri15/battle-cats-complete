use crate::{
    Fault,
    engine::{
        AppContext, Surface, dialog_top, draw_context, draw_surface_aligned, fill_rect,
        get_design_height2, get_drawable_width, get_text_texture, get_touch_x, get_touch_y,
        set_bgm_duck, set_draw_origin, set_draw_scale, set_tint, sound_manager,
        text_texture_cache, touch_released,
    },
};

const PANEL_WIDTH: i32 = 0x2b2;
const PANEL_PAD: i32 = 0x18;
const LINE_HEIGHT: i32 = 0x24;
const TEXT_SIZE: i32 = 0x1e;
const BUTTON_WIDTH: i32 = 0xc8;
const BUTTON_HEIGHT: i32 = 0x3c;
const BUTTON_GAP: i32 = 0x28;
const ROW_HEIGHT: i32 = 0x3c;
const ROW_GAP: i32 = 8;
const BORDER: i32 = 3;
const ALIGN_CENTER: i32 = 1;
const ALIGN_BOTH: i32 = 5;
const TWO_BUTTONS: i32 = 2;
const PRESSED: i32 = 2;
const CLOSING: i32 = 2;
const FINISHED: i32 = 5;
const DIM: [i32; 4] = [0, 0, 0, 0x80];
const PLATE: [i32; 4] = [0x2a, 0x26, 0x20, 0xf0];
const EDGE: [i32; 4] = [0xff, 0xff, 0xff, 0xff];
const BUTTON: [i32; 4] = [0xff, 0xc0, 0, 0xff];
const INK: [i32; 4] = [0, 0, 0, 0xff];
const FULL_DUCK: i32 = 0x64;
const VOLUME_STEP: i32 = 10;
const VOLUME_TOP: i32 = 100;
const SHUT: u8 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PauseOptions {
    pub music: i32,
    pub effects: i32,
    pub two_rows: bool,
    pub vibrate: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseOutcome {
    Idle,
    Changed(PauseOptions),
    Quit,
}

#[derive(Clone, Copy)]
enum PauseRow {
    Resume,
    Music,
    Effects,
    TwoRows,
    Vibrate,
    Quit,
}

const PAUSE_ROWS: [PauseRow; 6] = [
    PauseRow::Resume,
    PauseRow::Music,
    PauseRow::Effects,
    PauseRow::TwoRows,
    PauseRow::Vibrate,
    PauseRow::Quit,
];

struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn holds(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

fn tapped(ctx: &AppContext) -> Result<Option<(i32, i32)>, Fault> {
    if touch_released(ctx)? == 0 {
        return Ok(None);
    }

    Ok(Some((get_touch_x(ctx)?, get_touch_y(ctx)?)))
}

fn swallow_touch(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::TOUCH_BEGAN, [0])?;
    ctx.set_block_at::<1>(AppContext::TOUCH_RELEASED, [0])?;
    ctx.set_block_at::<1>(AppContext::TOUCH_IS_DOWN, [0])
}

fn panel(ctx: &AppContext, body: i32) -> Result<Rect, Fault> {
    let height = body + PANEL_PAD * 2;

    Ok(Rect {
        x: (get_drawable_width(ctx)? - PANEL_WIDTH) / 2,
        y: (get_design_height2(ctx) - height) / 2,
        width: PANEL_WIDTH,
        height,
    })
}

fn fill(ctx: &mut AppContext, rect: &Rect, color: [i32; 4]) -> Result<(), Fault> {
    let sink = draw_context(&mut ctx.draw)?;

    set_tint(sink, color[0], color[1], color[2], color[3]);
    fill_rect(sink, rect.x, rect.y, rect.width, rect.height);

    Ok(())
}

fn label(ctx: &mut AppContext, text: &[u8], x: i32, y: i32, align: i32, color: [i32; 4]) -> Result<(), Fault> {
    let font = ctx.default_font.clone();
    let texture = get_text_texture(text_texture_cache(ctx)?, text, &font, TEXT_SIZE, ALIGN_CENTER, 0);
    let sink = draw_context(&mut ctx.draw)?;

    set_tint(sink, color[0], color[1], color[2], color[3]);
    draw_surface_aligned(sink, Surface::Label(&texture), x, y, align);

    Ok(())
}

fn backdrop(ctx: &mut AppContext, plate: &Rect) -> Result<(), Fault> {
    set_draw_origin(ctx, 0, 0)?;
    set_draw_scale(draw_context(&mut ctx.draw)?, 1.0);

    let screen = Rect {
        x: 0,
        y: 0,
        width: get_drawable_width(ctx)?,
        height: get_design_height2(ctx),
    };
    let rim = Rect {
        x: plate.x - BORDER,
        y: plate.y - BORDER,
        width: plate.width + BORDER * 2,
        height: plate.height + BORDER * 2,
    };

    fill(ctx, &screen, DIM)?;
    fill(ctx, &rim, EDGE)?;
    fill(ctx, plate, PLATE)
}

fn finish(ctx: &mut AppContext) -> Result<(), Fault> {
    let sink = draw_context(&mut ctx.draw)?;

    set_tint(sink, 0xff, 0xff, 0xff, 0xff);

    Ok(())
}

fn dialog_lines(text: &[u8]) -> Vec<Vec<u8>> {
    String::from_utf8_lossy(text)
        .replace("<br>", "\n")
        .lines()
        .map(|line| line.as_bytes().to_vec())
        .collect()
}

fn dialog_buttons(plate: &Rect, flags: i32) -> Vec<(Rect, &'static [u8])> {
    let y = plate.y + plate.height - PANEL_PAD - BUTTON_HEIGHT;
    let centre = plate.x + plate.width / 2;

    if flags & TWO_BUTTONS == 0 {
        return vec![(
            Rect { x: centre - BUTTON_WIDTH / 2, y, width: BUTTON_WIDTH, height: BUTTON_HEIGHT },
            b"OK".as_slice(),
        )];
    }

    vec![
        (
            Rect { x: centre - BUTTON_GAP / 2 - BUTTON_WIDTH, y, width: BUTTON_WIDTH, height: BUTTON_HEIGHT },
            b"Yes".as_slice(),
        ),
        (
            Rect { x: centre + BUTTON_GAP / 2, y, width: BUTTON_WIDTH, height: BUTTON_HEIGHT },
            b"No".as_slice(),
        ),
    ]
}

fn dialog_plate(ctx: &AppContext, lines: usize) -> Result<Rect, Fault> {
    panel(ctx, lines as i32 * LINE_HEIGHT + PANEL_PAD + BUTTON_HEIGHT)
}

pub fn pump_dialogs(ctx: &mut AppContext) -> Result<bool, Fault> {
    let Some(dialog) = dialog_top(ctx) else {
        if ctx.dialogs.queued.is_empty() {
            return Ok(false);
        }

        let next = ctx.dialogs.queued.remove(0);

        ctx.dialogs.active.push(next);

        return Ok(true);
    };
    let Some(record) = ctx.dialogs.objects.get(&dialog).cloned() else {
        ctx.dialogs.active.retain(|held| *held != dialog);

        return Ok(false);
    };

    if record.state == CLOSING {
        ctx.dialogs.active.retain(|held| *held != dialog);

        if let Some(handler) = record.on_event {
            handler(ctx, dialog, FINISHED, record.button)?;
        }

        ctx.dialogs.objects.remove(&dialog);
        swallow_touch(ctx)?;

        return Ok(true);
    }

    if let Some(update) = record.on_update {
        update(ctx, dialog)?;
    }

    let plate = dialog_plate(ctx, dialog_lines(&record.text).len())?;
    let pressed = tapped(ctx)?.and_then(|(x, y)| {
        dialog_buttons(&plate, record.flags)
            .iter()
            .position(|(rect, _)| rect.holds(x, y))
    });

    swallow_touch(ctx)?;

    if let Some(button) = pressed {
        if let Some(held) = ctx.dialogs.objects.get_mut(&dialog) {
            held.button = button as i32;
        }

        if let Some(handler) = record.on_event {
            handler(ctx, dialog, PRESSED, button as i32)?;
        }
    }

    Ok(true)
}

pub fn draw_dialogs(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(dialog) = dialog_top(ctx) else {
        return Ok(());
    };
    let Some(record) = ctx.dialogs.objects.get(&dialog).cloned() else {
        return Ok(());
    };
    let lines = dialog_lines(&record.text);
    let plate = dialog_plate(ctx, lines.len())?;
    let centre = plate.x + plate.width / 2;

    backdrop(ctx, &plate)?;

    for (row, line) in lines.iter().enumerate() {
        label(ctx, line, centre, plate.y + PANEL_PAD + row as i32 * LINE_HEIGHT, ALIGN_CENTER, EDGE)?;
    }

    for (rect, caption) in dialog_buttons(&plate, record.flags) {
        fill(ctx, &rect, BUTTON)?;
        label(ctx, caption, rect.x + rect.width / 2, rect.y + rect.height / 2, ALIGN_BOTH, INK)?;
    }

    if let Some(paint) = record.on_draw {
        paint(ctx)?;
    }

    finish(ctx)
}

fn pause_plate(ctx: &AppContext) -> Result<Rect, Fault> {
    panel(ctx, PAUSE_ROWS.len() as i32 * (ROW_HEIGHT + ROW_GAP) - ROW_GAP)
}

fn pause_row(plate: &Rect, row: usize) -> Rect {
    Rect {
        x: plate.x + PANEL_PAD,
        y: plate.y + PANEL_PAD + row as i32 * (ROW_HEIGHT + ROW_GAP),
        width: plate.width - PANEL_PAD * 2,
        height: ROW_HEIGHT,
    }
}

fn close_pause(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::OPTION_WINDOW, [SHUT])?;
    ctx.set_block_at::<1>(AppContext::OPTION_MENU_IS_OPEN, [SHUT])?;
    set_bgm_duck(sound_manager(ctx)?, FULL_DUCK);

    Ok(())
}

fn stepped(level: i32, right_half: bool) -> i32 {
    let moved = if right_half { level + VOLUME_STEP } else { level - VOLUME_STEP };

    moved.clamp(0, VOLUME_TOP)
}

pub fn pump_pause(ctx: &mut AppContext, options: PauseOptions) -> Result<PauseOutcome, Fault> {
    if ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? == 0 || dialog_top(ctx).is_some() {
        return Ok(PauseOutcome::Idle);
    }

    let plate = pause_plate(ctx)?;
    let hit = tapped(ctx)?;

    swallow_touch(ctx)?;

    let Some((x, y)) = hit else {
        return Ok(PauseOutcome::Idle);
    };
    let Some((row, rect)) = (0..PAUSE_ROWS.len())
        .map(|row| (row, pause_row(&plate, row)))
        .find(|(_, rect)| rect.holds(x, y))
    else {
        return Ok(PauseOutcome::Idle);
    };
    let right_half = x >= rect.x + rect.width / 2;
    let mut changed = options;

    match PAUSE_ROWS[row] {
        PauseRow::Resume => {
            close_pause(ctx)?;

            return Ok(PauseOutcome::Idle);
        }
        PauseRow::Quit => {
            close_pause(ctx)?;

            return Ok(PauseOutcome::Quit);
        }
        PauseRow::Music => changed.music = stepped(options.music, right_half),
        PauseRow::Effects => changed.effects = stepped(options.effects, right_half),
        PauseRow::TwoRows => {
            changed.two_rows = !options.two_rows;
            ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [changed.two_rows as u8])?;
        }
        PauseRow::Vibrate => changed.vibrate = !options.vibrate,
    }

    Ok(PauseOutcome::Changed(changed))
}

pub fn draw_pause(ctx: &mut AppContext, options: PauseOptions) -> Result<(), Fault> {
    if ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? == 0 || dialog_top(ctx).is_some() {
        return Ok(());
    }

    let plate = pause_plate(ctx)?;

    backdrop(ctx, &plate)?;

    for (row, kind) in PAUSE_ROWS.iter().enumerate() {
        let rect = pause_row(&plate, row);
        let caption = match kind {
            PauseRow::Resume => "Resume".to_owned(),
            PauseRow::Music => format!("-   Music Volume {}   +", options.music),
            PauseRow::Effects => format!("-   Sound Volume {}   +", options.effects),
            PauseRow::TwoRows => format!("Two-Row Deck: {}", if options.two_rows { "On" } else { "Off" }),
            PauseRow::Vibrate => format!("Vibrate: {}", if options.vibrate { "On" } else { "Off" }),
            PauseRow::Quit => "Quit Battle".to_owned(),
        };

        fill(ctx, &rect, BUTTON)?;
        label(ctx, caption.as_bytes(), rect.x + rect.width / 2, rect.y + rect.height / 2, ALIGN_BOTH, INK)?;
    }

    finish(ctx)
}
