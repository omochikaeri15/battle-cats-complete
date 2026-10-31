use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use tracing::error;

use codex::graphics::animate::{FrameData, resolve_frame};
use codex::graphics::rig::Animation;
use kokoro::systems::animation::{Loop, cycle, loop_frame};

use crate::platform::{Atlas, Bytes, Captured, Fault, Painted, Platform, Scene, Surface};
use crate::systems::export::{Phase, png, settle};

use super::{Active, Held, Motions, NO_TINT, Region, ViewerState, offset_of, selected_index};

const FRAME_MS: u32 = 33;
const MAX_FRAMES: i32 = 600;
const TRANSPARENT: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub(super) async fn run(platform: Platform, mut state: ViewerState, surface: Signal<Option<Rc<dyn Surface>>>, motions: Motions, held: Held, slots: Vec<Option<usize>>, phase: Signal<Phase>) {
    let outcome = export(&platform, state, surface, &motions, &held, &slots).await;

    if let Err(fault) = &outcome {
        error!("The animation export failed: {fault}");
    }

    state.region.set(None);
    settle(platform, phase, outcome.is_ok()).await;
}

#[derive(Clone, Copy)]
struct Camera {
    size: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
    scale: f32,
    region: Option<Region>,
}

impl Camera {
    fn snapshot(state: ViewerState, scale: f32) -> Self {
        Self { size: *state.size.peek(), pan: *state.pan.peek(), zoom: *state.zoom.peek(), scale, region: *state.region.peek() }
    }
}

fn prefix(set_name: &str, slug: &str) -> String {
    let mut parts = set_name.split('_');
    let mut clean = parts.next().unwrap_or("").to_owned();

    if let Some(form) = parts.next()
        && !clean.is_empty()
        && clean.chars().all(|c| c.is_ascii_digit())
    {
        let number = match form.chars().next() {
            Some('f') => 1,
            Some('c') => 2,
            Some('s') => 3,
            Some('u') => 4,
            _ => 0,
        };

        if number > 0 {
            clean = format!("{clean}-{number}");
        }
    }

    format!("{clean}.{slug}")
}

fn cropped(captured: Captured, camera: &Camera) -> Captured {
    let Some(region) = camera.region else {
        return captured;
    };
    let (width, height) = camera.size;
    let scale = camera.scale;
    let zoom = camera.zoom * scale;
    let pan = camera.pan;
    let to_pixels = |x: f32, y: f32| ((width * scale / 2.0 + (x + pan.0) * zoom), (height * scale / 2.0 + (y + pan.1) * zoom));
    let (min_x, min_y) = to_pixels(region.x, region.y);
    let (max_x, max_y) = to_pixels(region.x + region.w, region.y + region.h);
    let x0 = (min_x.floor().max(0.0) as u32).min(captured.width);
    let y0 = (min_y.floor().max(0.0) as u32).min(captured.height);
    let x1 = (max_x.ceil().max(0.0) as u32).clamp(x0 + 1, captured.width.max(x0 + 1));
    let y1 = (max_y.ceil().max(0.0) as u32).clamp(y0 + 1, captured.height.max(y0 + 1));
    let (w, h) = ((x1 - x0).min(captured.width - x0), (y1 - y0).min(captured.height - y0));
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);

    for y in y0..y0 + h {
        let start = ((y * captured.width + x0) * 4) as usize;

        rgba.extend_from_slice(&captured.rgba[start..start + (w * 4) as usize]);
    }

    Captured { width: w, height: h, rgba }
}

async fn export(platform: &Platform, state: ViewerState, surface: Signal<Option<Rc<dyn Surface>>>, motions: &Motions, held: &Held, slots: &[Option<usize>]) -> Result<(), Fault> {
    let surface = surface.peek().clone().ok_or_else(|| Fault("the viewer has no canvas yet".to_owned()))?;
    let rig = held.0.rig.as_ref().ok_or_else(|| Fault("no rig is loaded".to_owned()))?;
    let anim = held.0.anim.as_deref();
    let label = state.selected_label.peek().clone();
    let index = selected_index(&motions.set, slots, label.as_deref());
    let motion = index.and_then(|index| motions.set.motions.get(index));
    let slug = motion.map_or_else(|| "model".to_owned(), |motion| motion.slug());
    let active = state.active.peek().clone();
    let offset = offset_of(Some(rig), *state.placement.peek());
    let atlas_image = rig.sheet.image_data.as_ref().ok_or_else(|| Fault("the rig has no sprite sheet".to_owned()))?;
    let camera = Camera::snapshot(state, platform.host().pixel_ratio());
    let (pan, zoom) = (camera.pan, camera.zoom);
    let base = prefix(&motions.set.name, &slug);
    let shoot = |frame: i32| {
        let mut frames = resolve_frame(rig, anim, frame, offset);

        flatten_glow(&mut frames);

        let parts: Vec<Painted> = frames.into_iter().map(|frame| Painted { frame, tint: NO_TINT }).collect();
        let atlas = Atlas { key: Arc::as_ptr(atlas_image) as usize, width: atlas_image.width(), height: atlas_image.height(), pixels: atlas_image.as_raw() };

        surface.capture(&Scene { atlas: Some(atlas), parts: &parts, shapes: &[], pan, zoom, clear: TRANSPARENT })
    };
    let playing = *state.playing.peek();

    if !playing {
        let shown = *state.frame.peek();
        let frame = active.as_ref().map_or(shown, |active| active.playback_frame(shown)).floor() as i32;
        let captured = cropped(shoot(frame).await?, &camera);
        let image = straight(captured)?;

        platform.host().download(&format!("{base}.{frame}f.png"), "image/png", png(&image)?);

        return Ok(());
    }

    let start = state.loop_start.peek().unwrap_or(0.0).max(0.0) as i32;
    let chosen_end = *state.loop_end.peek();
    let declared = anim.map(|anim| anim.declared_frames() - 1);
    let bound = chosen_end.map(|end| end as i32).or_else(|| active.as_ref().and_then(Active::loop_bound)).or(declared).unwrap_or(start);
    let end = bound.max(start).min(start + MAX_FRAMES - 1);
    let mut shots = Vec::with_capacity((end - start + 1) as usize);

    for frame in start..=end {
        let at = exported_frame(active.as_ref(), anim, frame as f32).floor() as i32;

        shots.push(straight(cropped(shoot(at).await?, &camera))?);
        platform.host().frame().await;
    }

    let mut encoded = Vec::with_capacity(shots.len());

    for shot in &shots {
        encoded.push(lossless(shot)?);
        platform.host().frame().await;
    }

    let bytes = animated_webp(&encoded, shots.first().map_or((1, 1), RgbaImage::dimensions), FRAME_MS)?;

    platform.host().download(&format!("{base}.{start}f~{end}f.webp"), "image/webp", bytes);

    Ok(())
}

fn exported_frame(active: Option<&Active>, anim: Option<&Animation>, raw: f32) -> f32 {
    let (Some(active), Some(anim)) = (active, anim) else {
        return raw;
    };
    let natively_loops = match active.looping {
        Loop::Exact => true,
        Loop::Auto => cycle(anim).is_some(),
        Loop::Frames | Loop::Continuous => false,
    };

    if natively_loops { raw } else { loop_frame(anim, raw) }
}

fn flatten_glow(frames: &mut [FrameData]) {
    for frame in frames {
        frame.glow = match frame.glow {
            1 | 3 => 1,
            2 => 0,
            other => other,
        };
    }
}

fn straight(captured: Captured) -> Result<RgbaImage, Fault> {
    let mut rgba = captured.rgba;

    for texel in rgba.as_chunks_mut::<4>().0 {
        let alpha = u32::from(texel[3]);

        if alpha > 0 && alpha < 255 {
            for channel in &mut texel[..3] {
                *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }

    RgbaImage::from_raw(captured.width, captured.height, rgba).ok_or_else(|| Fault("the captured frame had the wrong size".to_owned()))
}

fn lossless(image: &RgbaImage) -> Result<Vec<u8>, Fault> {
    let mut bytes = Vec::new();

    WebPEncoder::new_lossless(&mut Cursor::new(&mut bytes))
        .write_image(image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8)
        .map_err(|err| Fault(format!("a frame could not be encoded: {err}")))?;

    Ok(bytes)
}

fn chunk<'a>(file: &'a [u8], fourcc: &[u8; 4]) -> Option<&'a [u8]> {
    let mut at = 12;

    while at + 8 <= file.len() {
        let size = u32::from_le_bytes([file[at + 4], file[at + 5], file[at + 6], file[at + 7]]) as usize;
        let total = 8 + size + (size & 1);

        if &file[at..at + 4] == fourcc {
            return file.get(at..at + total);
        }

        at += total;
    }

    None
}

fn u24(value: u32) -> [u8; 3] {
    let bytes = value.to_le_bytes();

    [bytes[0], bytes[1], bytes[2]]
}

fn animated_webp(frames: &[Vec<u8>], (width, height): (u32, u32), duration_ms: u32) -> Result<Bytes, Fault> {
    if frames.is_empty() {
        return Err(Fault("there are no frames to export".to_owned()));
    }

    let mut body = Vec::new();

    body.extend_from_slice(b"VP8X");
    body.extend_from_slice(&10u32.to_le_bytes());
    body.push(0x12);
    body.extend_from_slice(&[0, 0, 0]);
    body.extend_from_slice(&u24(width - 1));
    body.extend_from_slice(&u24(height - 1));
    body.extend_from_slice(b"ANIM");
    body.extend_from_slice(&6u32.to_le_bytes());
    body.extend_from_slice(&[0, 0, 0, 0, 0, 0]);

    for encoded in frames {
        let bitstream = chunk(encoded, b"VP8L").ok_or_else(|| Fault("the encoder produced no lossless bitstream".to_owned()))?;
        let size = 16 + bitstream.len();

        body.extend_from_slice(b"ANMF");
        body.extend_from_slice(&(size as u32).to_le_bytes());
        body.extend_from_slice(&u24(0));
        body.extend_from_slice(&u24(0));
        body.extend_from_slice(&u24(width - 1));
        body.extend_from_slice(&u24(height - 1));
        body.extend_from_slice(&u24(duration_ms));
        body.push(0b0000_0011);
        body.extend_from_slice(bitstream);

        if size % 2 == 1 {
            body.push(0);
        }
    }

    let mut file = Vec::with_capacity(body.len() + 12);

    file.extend_from_slice(b"RIFF");
    file.extend_from_slice(&((body.len() + 4) as u32).to_le_bytes());
    file.extend_from_slice(b"WEBP");
    file.extend_from_slice(&body);

    Ok(Bytes::from(file))
}
