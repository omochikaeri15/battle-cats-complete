//! The scripted effects a background's JSON file spawns over the scene.
//!
//! A file declares a list of effect definitions, each naming a model, how many
//! instances to spawn, and a parameter for every property an instance carries.
//! A parameter is a fixed value, a choice between listed values, or a range
//! the engine rolls between, and each may be measured against a base such as
//! the world's edges or the screen's. The engine rolls every instance once on
//! spawn and again each time one expires.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::Rng;

/// The draw of a shared random group, scaled by this to span a range.
const GROUP_SCALE: i32 = 10000;

/// The exclusive bound a shared random group is drawn from.
const GROUP_BOUND: i32 = 0x2711;

/// The frames one second occupies.
const FRAMES_PER_SECOND: i32 = 30;

/// The width of the image a background is authored against.
const IMAGE_WIDTH: i32 = 960;

/// The vertical reach of the world above the top of the screen, in design pixels.
const WORLD_TOP_LIFT: i32 = 0x1e0;

/// The height the world's bottom edge is measured from.
const WORLD_BOTTOM_LINE: i32 = 0x208;

/// The design height the world's bottom edge subtracts, scaled by one hundred.
const WORLD_BOTTOM_SPAN: i32 = 0xcb20;

/// The lift of the world's top edge, scaled by one hundred.
const WORLD_TOP_SPAN: i32 = 0xc350;

/// Represents errors that can occur while parsing a background effect file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EffectsError {
    /// The bytes did not hold a JSON object.
    NotAnObject,
    /// The object carried neither a redirecting identifier nor a data list.
    MissingData,
    /// A parameter's text named no number.
    NotANumber(String),
    /// A parameter's values list was not a list.
    NotAList,
}

impl fmt::Display for EffectsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnObject => write!(formatter, "The background effect file did not hold a JSON object."),
            Self::MissingData => write!(formatter, "The background effect file declared no data list."),
            Self::NotANumber(text) => write!(formatter, "The background effect text {text:?} named no number."),
            Self::NotAList => write!(formatter, "A background effect parameter's values were not a list."),
        }
    }
}

impl std::error::Error for EffectsError {}

/// The quantity a parameter's value is measured against.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Base {
    /// The value is taken as it is written.
    #[default]
    None,
    /// The value is a count of background image widths.
    BgImage,
    /// The value is measured from the left edge of the world.
    WorldLeft,
    /// The value is measured from the right edge of the world.
    WorldRight,
    /// The value is measured from the top edge of the world.
    WorldTop,
    /// The value is measured from the bottom edge of the world.
    WorldBottom,
    /// The value is measured from the left edge of the screen.
    ScreenLeft,
    /// The value is measured from the right edge of the screen.
    ScreenRight,
    /// The value is measured from the top edge of the screen.
    ScreenTop,
    /// The value is measured from the bottom edge of the screen.
    ScreenBottom,
    /// The value is a screen coordinate.
    Screen,
    /// The value is a depth placed in front of the units.
    FrontChara,
    /// The value is a depth placed behind the units.
    BackChara,
    /// The value is a per second rate converted to a per frame one.
    SecondToFrame,
    /// The value is a percentage converted to a ratio.
    PercentToFloat,
    /// The value is a percentage converted to an eight bit alpha.
    PercentToAlpha,
    /// The value is a frame count added to the length of the instance's animation.
    AnimeLength,
    /// The value is a frame count added to the length of the instance's animation.
    AnimeInterval,
}

impl Base {
    /// Reads the base a parameter names.
    ///
    /// # Arguments
    /// * `name` - The text of the parameter's base.
    ///
    /// # Returns
    /// A `Base`, which is `Base::None` for a name the engine does not recognize.
    pub fn named(name: &str) -> Self {
        match name {
            "bgImage" => Self::BgImage,
            "worldLeft" => Self::WorldLeft,
            "worldRight" => Self::WorldRight,
            "worldTop" => Self::WorldTop,
            "worldBottom" => Self::WorldBottom,
            "screenLeft" => Self::ScreenLeft,
            "screenRight" => Self::ScreenRight,
            "screenTop" => Self::ScreenTop,
            "screenBottom" => Self::ScreenBottom,
            "screen" => Self::Screen,
            "frontChara" => Self::FrontChara,
            "backChara" => Self::BackChara,
            "secondToFrame" => Self::SecondToFrame,
            "percentToFloat" => Self::PercentToFloat,
            "percentToAlpha" => Self::PercentToAlpha,
            "animeLength" => Self::AnimeLength,
            "animeInterval" => Self::AnimeInterval,
            _ => Self::None,
        }
    }
}

/// One end of a parameter's range.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Bound<T> {
    /// The end's value before its base is applied.
    pub value: T,
    /// The quantity the value is measured against.
    pub base: Base,
}

/// The random group of a parameter that draws on its own.
const OWN_DRAW: i32 = -1;

/// One property of an effect, as the file declares it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param<T> {
    /// Whether the file declares the parameter at all.
    pub enabled: bool,
    /// The fixed value, when the file declares one.
    pub value: T,
    /// The values the engine picks one of at random, when the file lists them.
    pub values: Vec<T>,
    /// The quantity the fixed or listed value is measured against.
    pub base: Base,
    /// The lower end of the range, when the file declares one.
    pub min: Option<Bound<T>>,
    /// The upper end of the range, when the file declares one.
    pub max: Option<Bound<T>>,
    /// The random group a ranged parameter shares its draw with, or minus one for a draw of its own.
    pub rand_group: i32,
}

impl<T: Default> Default for Param<T> {
    fn default() -> Self {
        Self {
            enabled: false,
            value: T::default(),
            values: Vec::new(),
            base: Base::None,
            min: None,
            max: None,
            rand_group: OWN_DRAW,
        }
    }
}

impl<T: Default> Param<T> {
    fn fixed(value: T) -> Self {
        Self { enabled: true, value, ..Self::default() }
    }
}

/// The positions an instance's model is repeated at along the x axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spaced {
    /// The first multiple of the spacing the model is drawn at.
    pub pos1: i32,
    /// The last multiple of the spacing the model is drawn at.
    pub pos2: i32,
    /// The spacing before its base is applied.
    pub value: i32,
    /// The quantity the spacing is measured against.
    pub base: Base,
}

impl Default for Spaced {
    fn default() -> Self {
        Self { pos1: 0, pos2: 0, value: 1, base: Base::BgImage }
    }
}

/// The files an effect draws its model from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectFile {
    /// The name of the sprite sheet image.
    pub image: String,
    /// The name of the sprite cut list.
    pub imgcut: String,
    /// The name of the model.
    pub model: String,
    /// The name of the animation.
    pub anime: String,
}

/// One effect definition, declaring how its instances are spawned and move.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EffectDef {
    /// The files the model is drawn from, when the definition names its own rather than one of the background's.
    pub file: Option<EffectFile>,
    /// The number of instances spawned.
    pub count: Param<i32>,
    /// The model drawn, which a definition naming its own files has overwritten with a key below zero.
    pub model: Param<i32>,
    /// The depth layer, where one is in front of the units, two behind them and zero over everything.
    pub z: Param<i32>,
    /// The animation frame an instance restarts at.
    pub frame: Param<i32>,
    /// The animation frame an instance first spawns at.
    pub start_frame: Param<i32>,
    /// The frames an instance waits before it moves and draws.
    pub wait: Param<i32>,
    /// The frames an instance lives before it respawns, or zero to live until destroyed.
    pub life_time: Param<i32>,
    /// The x position an instance restarts at.
    pub x: Param<f32>,
    /// The x position an instance first spawns at.
    pub start_x: Param<f32>,
    /// The y position an instance restarts at.
    pub y: Param<f32>,
    /// The y position an instance first spawns at.
    pub start_y: Param<f32>,
    /// The uniform scale an instance restarts at.
    pub scale: Param<f32>,
    /// The uniform scale an instance first spawns at.
    pub start_scale: Param<f32>,
    /// The horizontal scale an instance restarts at.
    pub scale_x: Param<f32>,
    /// The horizontal scale an instance first spawns at.
    pub start_scale_x: Param<f32>,
    /// The vertical scale an instance restarts at.
    pub scale_y: Param<f32>,
    /// The vertical scale an instance first spawns at.
    pub start_scale_y: Param<f32>,
    /// The rotation in degrees.
    pub angle: Param<f32>,
    /// The rotation added each frame, in degrees.
    pub angular_v: Param<f32>,
    /// The opacity, from zero to two hundred and fifty-five.
    pub alpha: Param<f32>,
    /// The speed along the heading an instance restarts at.
    pub v: Param<f32>,
    /// The speed along the heading an instance first spawns at.
    pub start_v: Param<f32>,
    /// The heading in degrees.
    pub move_angle: Param<f32>,
    /// The horizontal speed an instance restarts at.
    pub vx: Param<f32>,
    /// The horizontal speed an instance first spawns at.
    pub start_vx: Param<f32>,
    /// The vertical speed an instance restarts at.
    pub vy: Param<f32>,
    /// The vertical speed an instance first spawns at.
    pub start_vy: Param<f32>,
    /// The x position below which an instance is destroyed.
    pub destroy_left: Param<f32>,
    /// The x position above which an instance is destroyed.
    pub destroy_right: Param<f32>,
    /// The y position below which an instance is destroyed.
    pub destroy_top: Param<f32>,
    /// The y position above which an instance is destroyed.
    pub destroy_bottom: Param<f32>,
    /// The positions the model is repeated at.
    pub equally_spaced: Spaced,
}

/// The parsed contents of a background effect file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectsFile {
    /// The file defers to another background's effect file.
    Redirect(i32),
    /// The file declares its own effects.
    Defs(Vec<EffectDef>),
}

impl EffectsFile {
    /// Parses a background effect file.
    ///
    /// A definition that names its own files has its model parameter rewritten
    /// to a key below zero, the bitwise complement of its index, as the engine
    /// does so a drawn instance can be traced back to the files it came from.
    ///
    /// # Arguments
    /// * `bytes` - The raw bytes of the background's `bg*.json` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `EffectsFile` on success, or an
    /// `EffectsError` if the file was not an effect file or a parameter could
    /// not be read.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, EffectsError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<EffectsFile, EffectsError> {
    let document: Value = serde_json::from_slice(bytes).map_err(|_| EffectsError::NotAnObject)?;
    let root = document.as_object().ok_or(EffectsError::NotAnObject)?;

    if let Some(id) = root.get("id") {
        return Ok(EffectsFile::Redirect(as_int(id)? as i32));
    }

    let data = root.get("data").and_then(Value::as_array).ok_or(EffectsError::MissingData)?;
    let mut defs = Vec::with_capacity(data.len());

    for (index, entry) in data.iter().enumerate() {
        let entry = entry.as_object().ok_or(EffectsError::MissingData)?;
        let object = |key: &str| entry.get(key).and_then(Value::as_object);

        let mut def = EffectDef {
            file: None,
            count: int_param(object("count"))?,
            model: int_param(object("model"))?,
            z: int_param(object("z"))?,
            frame: int_param(object("frame"))?,
            start_frame: int_param(object("startFrame"))?,
            wait: int_param(object("wait"))?,
            life_time: int_param(object("lifeTime"))?,
            x: float_param(object("x"))?,
            start_x: float_param(object("startX"))?,
            y: float_param(object("y"))?,
            start_y: float_param(object("startY"))?,
            scale: float_param(object("scale"))?,
            start_scale: float_param(object("startScale"))?,
            scale_x: float_param(object("scaleX"))?,
            start_scale_x: float_param(object("startScaleX"))?,
            scale_y: float_param(object("scaleY"))?,
            start_scale_y: float_param(object("startScaleY"))?,
            angle: float_param(object("angle"))?,
            angular_v: float_param(object("angularV"))?,
            alpha: float_param(object("alpha"))?,
            v: float_param(object("v"))?,
            start_v: float_param(object("startV"))?,
            move_angle: float_param(object("moveAngle"))?,
            vx: float_param(object("vx"))?,
            start_vx: float_param(object("startVx"))?,
            vy: float_param(object("vy"))?,
            start_vy: float_param(object("startVy"))?,
            destroy_left: float_param(object("destroyLeft"))?,
            destroy_right: float_param(object("destroyRight"))?,
            destroy_top: float_param(object("destroyTop"))?,
            destroy_bottom: float_param(object("destroyBottom"))?,
            equally_spaced: spaced(object("equallySpaced"))?,
        };

        if let Some(file) = object("file") {
            let text = |key: &str| file.get(key).map(as_text).unwrap_or_default();

            def.model = Param::fixed(!(index as i32));
            def.file = Some(EffectFile {
                image: text("image"),
                imgcut: text("imgcut"),
                model: text("model"),
                anime: text("anime"),
            });
        }

        defs.push(def);
    }

    Ok(EffectsFile::Defs(defs))
}

fn as_int(value: &Value) -> Result<i64, EffectsError> {
    match value {
        Value::String(text) => leading_int(text),
        Value::Bool(flag) => Ok(*flag as i64),
        Value::Number(number) => Ok(number
            .as_i64()
            .or_else(|| number.as_u64().map(|held| held as i64))
            .or_else(|| number.as_f64().map(|held| held as i64))
            .unwrap_or(0)),
        _ => Ok(0),
    }
}

fn as_float(value: &Value) -> Result<f64, EffectsError> {
    match value {
        Value::String(text) => leading_float(text),
        Value::Bool(flag) => Ok(*flag as i8 as f64),
        Value::Number(number) => Ok(number.as_f64().unwrap_or(0.0)),
        _ => Ok(0.0),
    }
}

fn as_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    }
}

fn leading_int(text: &str) -> Result<i64, EffectsError> {
    let trimmed = text.trim_start();
    let digits = trimmed
        .char_indices()
        .take_while(|(at, c)| c.is_ascii_digit() || (*at == 0 && (*c == '-' || *c == '+')))
        .count();

    trimmed[..digits].parse::<i64>().map_err(|_| EffectsError::NotANumber(text.to_owned()))
}

fn leading_float(text: &str) -> Result<f64, EffectsError> {
    let trimmed = text.trim_start();
    let mut end = 0;

    for (at, c) in trimmed.char_indices() {
        let accepted = c.is_ascii_digit()
            || c == '.'
            || (at == 0 && (c == '-' || c == '+'))
            || ((c == 'e' || c == 'E') && at > 0)
            || ((c == '-' || c == '+') && trimmed[..at].ends_with(['e', 'E']));

        if !accepted {
            break;
        }

        end = at + c.len_utf8();
    }

    while end > 0 && trimmed[..end].parse::<f64>().is_err() {
        end -= 1;
    }

    if end == 0 {
        return Err(EffectsError::NotANumber(text.to_owned()));
    }

    trimmed[..end].parse::<f64>().map_err(|_| EffectsError::NotANumber(text.to_owned()))
}

fn base_of(node: &serde_json::Map<String, Value>) -> Base {
    Base::named(&node.get("base").map(as_text).unwrap_or_default())
}

fn param<T: Default + Copy>(
    node: Option<&serde_json::Map<String, Value>>,
    read: impl Fn(&Value) -> Result<T, EffectsError>,
) -> Result<Param<T>, EffectsError> {
    let Some(node) = node else {
        return Ok(Param::default());
    };

    let mut spec = Param { enabled: true, ..Param::default() };

    if let Some(found) = node.get("value") {
        spec.value = read(found)?;
    } else if let Some(listed) = node.get("values") {
        let values = listed.as_array().ok_or(EffectsError::NotAList)?;

        for found in values {
            spec.values.push(read(found)?);
        }
    } else {
        let bound = |key: &str| -> Result<Option<Bound<T>>, EffectsError> {
            let Some(found) = node.get(key).and_then(Value::as_object) else {
                return Ok(None);
            };
            let value = found.get("value").map(&read).transpose()?.unwrap_or_default();

            Ok(Some(Bound { value, base: base_of(found) }))
        };

        spec.min = bound("min")?;
        spec.max = bound("max")?;

        if let Some(found) = node.get("randGroup") {
            spec.rand_group = as_int(found)? as i32;
        }
    }

    spec.base = base_of(node);

    Ok(spec)
}

fn int_param(node: Option<&serde_json::Map<String, Value>>) -> Result<Param<i32>, EffectsError> {
    param(node, |value| as_int(value).map(|held| held as i32))
}

fn float_param(node: Option<&serde_json::Map<String, Value>>) -> Result<Param<f32>, EffectsError> {
    param(node, |value| as_float(value).map(|held| held as f32))
}

fn spaced(node: Option<&serde_json::Map<String, Value>>) -> Result<Spaced, EffectsError> {
    let Some(node) = node else {
        return Ok(Spaced::default());
    };

    let int = |key: &str| node.get(key).map(as_int).transpose().map(|held| held.unwrap_or(0) as i32);

    Ok(Spaced { pos1: int("pos1")?, pos2: int("pos2")?, value: int("value")?, base: base_of(node) })
}

/// The world an effect's parameters are measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Frame {
    pub(super) origin_x: i32,
    pub(super) origin_y: i32,
    pub(super) stage_length: i32,
    pub(super) letterbox: i32,
    pub(super) min_zoom: i32,
    pub(super) width: i32,
    pub(super) height: i32,
}

impl Frame {
    fn world_top_lift(&self) -> i32 {
        let lift = (self.letterbox / 2).wrapping_mul(100).wrapping_add(WORLD_TOP_SPAN);

        lift.checked_div(self.min_zoom).unwrap_or(0)
    }

    fn world_bottom_span(&self) -> i32 {
        let span = self.height.wrapping_mul(100).wrapping_sub(WORLD_BOTTOM_SPAN);

        span.checked_div(self.min_zoom).unwrap_or(0)
    }

    pub(super) fn resolve_int(&self, value: i32, base: Base, anim_length: impl Fn() -> i32) -> i32 {
        match base {
            Base::BgImage => value.wrapping_mul(IMAGE_WIDTH),
            Base::WorldLeft => (value as f64 + self.origin_x as f64 / -10.0) as i32,
            Base::WorldRight => (value as f32 + self.origin_x.wrapping_add(self.stage_length) as f32 / 10.0) as i32,
            Base::WorldTop => value
                .wrapping_sub(self.world_top_lift())
                .wrapping_add(self.origin_y)
                .wrapping_add(WORLD_TOP_LIFT),
            Base::WorldBottom => value
                .wrapping_sub(self.letterbox)
                .wrapping_add(self.world_bottom_span())
                .wrapping_add(WORLD_BOTTOM_LINE),
            Base::ScreenRight => self.width.wrapping_add(value),
            Base::ScreenBottom => self.height.wrapping_add(value),
            Base::FrontChara => value.wrapping_add(1),
            Base::BackChara => value.wrapping_add(2),
            Base::SecondToFrame => value / FRAMES_PER_SECOND,
            Base::PercentToFloat => value / 100,
            Base::PercentToAlpha => (value.wrapping_shl(8).wrapping_sub(value)) / 100,
            Base::AnimeLength | Base::AnimeInterval => value.wrapping_add(anim_length()),
            Base::None | Base::ScreenLeft | Base::ScreenTop | Base::Screen => value,
        }
    }

    pub(super) fn resolve_float(&self, value: f32, base: Base, anim_length: impl Fn() -> i32) -> f32 {
        match base {
            Base::BgImage => value * IMAGE_WIDTH as f32,
            Base::WorldLeft => (value as f64 + self.origin_x as f64 / -10.0) as f32,
            Base::WorldRight => value + self.origin_x.wrapping_add(self.stage_length) as f32 / 10.0,
            Base::WorldTop => {
                value + self.origin_y.wrapping_sub(self.world_top_lift()).wrapping_add(WORLD_TOP_LIFT) as f32
            }
            Base::WorldBottom => {
                value + self.world_bottom_span().wrapping_sub(self.letterbox).wrapping_add(WORLD_BOTTOM_LINE) as f32
            }
            Base::ScreenRight => value + self.width as f32,
            Base::ScreenBottom => value + self.height as f32,
            Base::FrontChara => value + 1.0,
            Base::BackChara => value + 2.0,
            Base::SecondToFrame => value / FRAMES_PER_SECOND as f32,
            Base::PercentToFloat => value / 100.0,
            Base::PercentToAlpha => value * 255.0 / 100.0,
            Base::AnimeLength | Base::AnimeInterval => value + anim_length() as f32,
            Base::None | Base::ScreenLeft | Base::ScreenTop | Base::Screen => value,
        }
    }
}

fn truncated(value: f32) -> i32 {
    if (-2147483648.0..2147483648.0).contains(&value) { value as i32 } else { i32::MIN }
}

pub(super) fn roll_int(
    rng: &mut Rng,
    frame: &Frame,
    spec: &Param<i32>,
    groups: &mut BTreeMap<i32, i32>,
    fallback: i32,
    anim_length: impl Fn() -> i32,
) -> i32 {
    if !spec.enabled {
        return fallback;
    }

    let (Some(min), Some(max)) = (spec.min, spec.max) else {
        let value = if spec.values.is_empty() {
            spec.value
        } else {
            let pick = rng.roll(spec.values.len() as i32);

            spec.values.get(pick as usize).copied().unwrap_or_default()
        };

        return frame.resolve_int(value, spec.base, anim_length);
    };

    let lower = frame.resolve_int(min.value, min.base, &anim_length);
    let upper = frame.resolve_int(max.value, max.base, &anim_length);

    if spec.rand_group == OWN_DRAW {
        let (low, high) = (lower.min(upper), lower.max(upper));

        return rng.roll(high.wrapping_sub(low).wrapping_add(1)).wrapping_add(low);
    }

    let draw = *groups.entry(spec.rand_group).or_insert_with(|| rng.roll(GROUP_BOUND));
    let span = upper.wrapping_sub(lower);

    (span.wrapping_mul(draw) / GROUP_SCALE).wrapping_add(lower)
}

pub(super) fn roll_float(
    rng: &mut Rng,
    frame: &Frame,
    spec: &Param<f32>,
    groups: &mut BTreeMap<i32, i32>,
    fallback: f32,
    anim_length: impl Fn() -> i32,
) -> f32 {
    if !spec.enabled {
        return fallback;
    }

    let (Some(min), Some(max)) = (spec.min, spec.max) else {
        let value = if spec.values.is_empty() {
            spec.value
        } else {
            let pick = rng.roll(spec.values.len() as i32);

            spec.values.get(pick as usize).copied().unwrap_or_default()
        };

        return frame.resolve_float(value, spec.base, anim_length);
    };

    let lower = frame.resolve_float(min.value, min.base, &anim_length);
    let upper = frame.resolve_float(max.value, max.base, &anim_length);

    if spec.rand_group == OWN_DRAW {
        let (low, high) = (lower.min(upper), lower.max(upper));
        let draw = rng.roll(truncated((high - low) * GROUP_SCALE as f32 + 1.0));

        return (low * GROUP_SCALE as f32 + draw as f32) / GROUP_SCALE as f32;
    }

    let draw = *groups.entry(spec.rand_group).or_insert_with(|| rng.roll(GROUP_BOUND));
    let span = upper - lower;

    lower + span * draw as f32 / GROUP_SCALE as f32
}

/// One spawned instance of an effect definition.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EffectInstance {
    /// The index of the definition the instance was spawned from.
    pub def_index: usize,
    /// The key of the model drawn, which is below zero for a definition naming its own files.
    pub model: i32,
    /// The depth layer.
    pub z: i32,
    /// The animation frame.
    pub frame: i32,
    /// The frames left before the instance moves and draws.
    pub wait: i32,
    /// The frames the instance lives before it respawns, or zero to live until destroyed.
    pub life_time: i32,
    /// The x position.
    pub x: f32,
    /// The y position.
    pub y: f32,
    /// The uniform scale.
    pub scale: f32,
    /// The horizontal scale.
    pub scale_x: f32,
    /// The vertical scale.
    pub scale_y: f32,
    /// The rotation in degrees.
    pub angle: f32,
    /// The rotation added each frame.
    pub angular_v: f32,
    /// The opacity, from zero to two hundred and fifty-five.
    pub alpha: f32,
    /// The speed along the heading.
    pub v: f32,
    /// The heading in degrees.
    pub move_angle: f32,
    /// The horizontal speed.
    pub vx: f32,
    /// The vertical speed.
    pub vy: f32,
    /// The x position below which the instance is destroyed.
    pub destroy_left: f32,
    /// The x position above which the instance is destroyed.
    pub destroy_right: f32,
    /// The y position below which the instance is destroyed.
    pub destroy_top: f32,
    /// The y position above which the instance is destroyed.
    pub destroy_bottom: f32,
    groups: BTreeMap<i32, i32>,
}

impl EffectInstance {
    pub(super) fn spawned(def_index: usize) -> Self {
        Self { def_index, ..Self::default() }
    }

    pub(super) fn roll(
        &mut self,
        rng: &mut Rng,
        frame: &Frame,
        def: &EffectDef,
        first_spawn: bool,
        anim_length: impl Fn(i32) -> i32,
    ) {
        self.groups.clear();

        let groups = &mut self.groups;

        if first_spawn || def.model.enabled {
            self.model = roll_int(rng, frame, &def.model, groups, 0, || anim_length(-1));
        }

        let model = self.model;
        let length = || anim_length(model);
        let starting = |start: &Param<f32>, steady: &Param<f32>| -> Option<Param<f32>> {
            if first_spawn {
                Some(if start.enabled { start.clone() } else { steady.clone() })
            } else {
                steady.enabled.then(|| steady.clone())
            }
        };
        let scaled = |start: &Param<f32>, steady: &Param<f32>| -> Option<(Param<f32>, f32)> {
            if first_spawn {
                Some(if start.enabled { (start.clone(), 0.0) } else { (steady.clone(), 1.0) })
            } else {
                steady.enabled.then(|| (steady.clone(), 1.0))
            }
        };

        if let Some(spec) = starting(&def.start_x, &def.x) {
            self.x = roll_float(rng, frame, &spec, groups, 0.0, length);
        }

        if let Some(spec) = starting(&def.start_y, &def.y) {
            self.y = roll_float(rng, frame, &spec, groups, 0.0, length);
        }

        if first_spawn || def.z.enabled {
            self.z = roll_int(rng, frame, &def.z, groups, 0, length);
        }

        if let Some((spec, fallback)) = scaled(&def.start_scale, &def.scale) {
            self.scale = roll_float(rng, frame, &spec, groups, fallback, length);
        }

        if let Some((spec, fallback)) = scaled(&def.start_scale_x, &def.scale_x) {
            self.scale_x = roll_float(rng, frame, &spec, groups, fallback, length);
        }

        if let Some((spec, fallback)) = scaled(&def.start_scale_y, &def.scale_y) {
            self.scale_y = roll_float(rng, frame, &spec, groups, fallback, length);
        }

        if first_spawn || def.angle.enabled {
            self.angle = roll_float(rng, frame, &def.angle, groups, 0.0, length);
        }

        if first_spawn || def.angular_v.enabled {
            self.angular_v = roll_float(rng, frame, &def.angular_v, groups, 0.0, length);
        }

        if first_spawn || def.alpha.enabled {
            self.alpha = roll_float(rng, frame, &def.alpha, groups, 255.0, length);
        }

        if let Some(spec) = starting(&def.start_v, &def.v) {
            self.v = roll_float(rng, frame, &spec, groups, 0.0, length);
        }

        if first_spawn || def.move_angle.enabled {
            self.move_angle = roll_float(rng, frame, &def.move_angle, groups, 0.0, length);
        }

        if let Some(spec) = starting(&def.start_vx, &def.vx) {
            self.vx = roll_float(rng, frame, &spec, groups, 0.0, length);
        }

        if let Some(spec) = starting(&def.start_vy, &def.vy) {
            self.vy = roll_float(rng, frame, &spec, groups, 0.0, length);
        }

        if first_spawn || def.destroy_left.enabled {
            self.destroy_left = roll_float(rng, frame, &def.destroy_left, groups, 0.0, length);
        }

        if first_spawn || def.destroy_right.enabled {
            self.destroy_right = roll_float(rng, frame, &def.destroy_right, groups, 0.0, length);
        }

        if first_spawn || def.destroy_top.enabled {
            self.destroy_top = roll_float(rng, frame, &def.destroy_top, groups, 0.0, length);
        }

        if first_spawn || def.destroy_bottom.enabled {
            self.destroy_bottom = roll_float(rng, frame, &def.destroy_bottom, groups, 0.0, length);
        }

        if first_spawn || def.frame.enabled {
            self.frame = roll_int(rng, frame, &def.frame, groups, 0, length);
        }

        if first_spawn || def.wait.enabled {
            self.wait = roll_int(rng, frame, &def.wait, groups, 0, length);
        }

        if first_spawn || def.life_time.enabled {
            self.life_time = roll_int(rng, frame, &def.life_time, groups, 0, length);
        }
    }

    pub(super) fn expired(&self, def: &EffectDef, aged: i32) -> bool {
        (self.life_time > 0 && aged >= self.life_time)
            || (def.destroy_left.enabled && self.destroy_left > self.x)
            || (def.destroy_right.enabled && self.x > self.destroy_right)
            || (def.destroy_top.enabled && self.destroy_top > self.y)
            || (def.destroy_bottom.enabled && self.y > self.destroy_bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_definition_with_its_own_files_is_keyed_below_zero() {
        let parsed = EffectsFile::parse(
            r#"{"data":[{"count":{"value":"3"}},{"file":{"image":"a.png","imgcut":"a.imgcut","model":"a.mamodel","anime":"a.maanim"},"x":{"min":{"value":0,"base":"worldLeft"},"max":{"value":0,"base":"worldRight"},"randGroup":2}}]}"#,
        )
        .expect("the file parses");
        let EffectsFile::Defs(defs) = parsed else {
            panic!("the file declares its own data");
        };

        assert_eq!(defs[0].count.value, 3);
        assert!(!defs[0].model.enabled);
        assert_eq!(defs[1].model.value, !1);
        assert_eq!(defs[1].file.as_ref().map(|file| file.anime.as_str()), Some("a.maanim"));
        assert_eq!(defs[1].x.rand_group, 2);
        assert_eq!(defs[1].x.max.map(|bound| bound.base), Some(Base::WorldRight));
    }

    #[test]
    fn a_redirecting_file_names_the_file_it_defers_to() {
        assert_eq!(EffectsFile::parse(r#"{"id":1015}"#), Ok(EffectsFile::Redirect(1015)));
    }

    #[test]
    fn a_parameter_without_a_group_draws_on_its_own() {
        let frame = Frame { origin_x: 0, origin_y: 0, stage_length: 16000, letterbox: 0, min_zoom: 61, width: 960, height: 640 };
        let spec = Param {
            enabled: true,
            min: Some(Bound { value: 0.0, base: Base::None }),
            max: Some(Bound { value: 1000.0, base: Base::None }),
            ..Param::default()
        };
        let mut groups = BTreeMap::new();
        let mut rng = Rng::new(100);

        let first = roll_float(&mut rng, &frame, &spec, &mut groups, 0.0, || 0);
        let second = roll_float(&mut rng, &frame, &spec, &mut groups, 0.0, || 0);

        assert!(groups.is_empty(), "an ungrouped draw records nothing");
        assert_ne!(first, second);
    }

    #[test]
    fn a_shared_random_group_draws_once() {
        let frame = Frame { origin_x: 0, origin_y: 0, stage_length: 16000, letterbox: 0, min_zoom: 61, width: 960, height: 640 };
        let spec = Param {
            enabled: true,
            min: Some(Bound { value: 0, base: Base::None }),
            max: Some(Bound { value: 100, base: Base::None }),
            rand_group: 4,
            ..Param::default()
        };
        let mut groups = BTreeMap::new();
        let mut rng = Rng::new(100);

        let first = roll_int(&mut rng, &frame, &spec, &mut groups, 0, || 0);
        let after = rng.state();
        let second = roll_int(&mut rng, &frame, &spec, &mut groups, 0, || 0);

        assert_eq!(first, second);
        assert_eq!(rng.state(), after);
    }
}
