use std::collections::BTreeMap;

use crate::Fault;

use super::{bg_param_enabled, bg_param_roll_float, bg_param_roll_int, AppContext, BgEquallySpaced, BgParamSpec};

const SITE: &str = "bg_effect_roll_params";

#[derive(Clone, Default, PartialEq, Debug)]
pub struct BgEffectDef {
    pub count: BgParamSpec<i32>,
    pub model: BgParamSpec<i32>,
    pub z: BgParamSpec<i32>,
    pub frame: BgParamSpec<i32>,
    pub start_frame: BgParamSpec<i32>,
    pub wait: BgParamSpec<i32>,
    pub life_time: BgParamSpec<i32>,
    pub x: BgParamSpec<f32>,
    pub start_x: BgParamSpec<f32>,
    pub y: BgParamSpec<f32>,
    pub start_y: BgParamSpec<f32>,
    pub scale: BgParamSpec<f32>,
    pub start_scale: BgParamSpec<f32>,
    pub scale_x: BgParamSpec<f32>,
    pub start_scale_x: BgParamSpec<f32>,
    pub scale_y: BgParamSpec<f32>,
    pub start_scale_y: BgParamSpec<f32>,
    pub angle: BgParamSpec<f32>,
    pub angular_v: BgParamSpec<f32>,
    pub alpha: BgParamSpec<f32>,
    pub v: BgParamSpec<f32>,
    pub start_v: BgParamSpec<f32>,
    pub move_angle: BgParamSpec<f32>,
    pub vx: BgParamSpec<f32>,
    pub start_vx: BgParamSpec<f32>,
    pub vy: BgParamSpec<f32>,
    pub start_vy: BgParamSpec<f32>,
    pub destroy_left: BgParamSpec<f32>,
    pub destroy_right: BgParamSpec<f32>,
    pub destroy_top: BgParamSpec<f32>,
    pub destroy_bottom: BgParamSpec<f32>,
    pub equally_spaced: BgEquallySpaced,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct BgEffectRolls {
    pub def_index: i32,
    pub model: i32,
    pub z: i32,
    pub frame: i32,
    pub wait: i32,
    pub life_time: i32,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub angle: f32,
    pub angular_v: f32,
    pub alpha: f32,
    pub v: f32,
    pub move_angle: f32,
    pub vx: f32,
    pub vy: f32,
    pub destroy_left: f32,
    pub destroy_right: f32,
    pub destroy_top: f32,
    pub destroy_bottom: f32,
    pub groups: BTreeMap<i32, i32>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct BgEffects {
    pub origin_x: i32,
    pub origin_y: i32,
    pub defs: Vec<BgEffectDef>,
    pub instances: Vec<BgEffectRolls>,
    pub image_names: BTreeMap<i32, Vec<u8>>,
    pub model_names: BTreeMap<i32, Vec<u8>>,
    pub model_anims: BTreeMap<i32, Vec<u8>>,
}

pub fn bg_effect_roll_params(ctx: &mut AppContext, instance: usize, first_spawn: i32) -> Result<(), Fault> {
    let mut rolls = ctx.bg_effects.instances.get(instance).cloned().ok_or(Fault::IndexOutOfRange { site: SITE, index: instance as i64, limit: ctx.bg_effects.instances.len() as i64 })?;

    rolls.groups.clear();

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.model) != 0 {
        let spec = def.model.clone();

        rolls.model = bg_param_roll_int(ctx, &spec, &mut rolls.groups, -1, 0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_x) != 0 { def.start_x.clone() } else { def.x.clone() })
    } else if bg_param_enabled(&def.x) != 0 {
        Some(def.x.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.x = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_y) != 0 { def.start_y.clone() } else { def.y.clone() })
    } else if bg_param_enabled(&def.y) != 0 {
        Some(def.y.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.y = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.z) != 0 {
        let spec = def.z.clone();

        rolls.z = bg_param_roll_int(ctx, &spec, &mut rolls.groups, rolls.model, 0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_scale) != 0 { def.start_scale.clone() } else { def.scale.clone() })
    } else if bg_param_enabled(&def.scale) != 0 {
        Some(def.scale.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.scale = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 1.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_scale_x) != 0 { def.start_scale_x.clone() } else { def.scale_x.clone() })
    } else if bg_param_enabled(&def.scale_x) != 0 {
        Some(def.scale_x.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.scale_x = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 1.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_scale_y) != 0 { def.start_scale_y.clone() } else { def.scale_y.clone() })
    } else if bg_param_enabled(&def.scale_y) != 0 {
        Some(def.scale_y.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.scale_y = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 1.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.angle) != 0 {
        let spec = def.angle.clone();

        rolls.angle = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.angular_v) != 0 {
        let spec = def.angular_v.clone();

        rolls.angular_v = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.alpha) != 0 {
        let spec = def.alpha.clone();

        rolls.alpha = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 255.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_v) != 0 { def.start_v.clone() } else { def.v.clone() })
    } else if bg_param_enabled(&def.v) != 0 {
        Some(def.v.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.v = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.move_angle) != 0 {
        let spec = def.move_angle.clone();

        rolls.move_angle = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_vx) != 0 { def.start_vx.clone() } else { def.vx.clone() })
    } else if bg_param_enabled(&def.vx) != 0 {
        Some(def.vx.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.vx = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;
    let spec = if first_spawn != 0 {
        Some(if bg_param_enabled(&def.start_vy) != 0 { def.start_vy.clone() } else { def.vy.clone() })
    } else if bg_param_enabled(&def.vy) != 0 {
        Some(def.vy.clone())
    } else {
        None
    };

    if let Some(spec) = spec {
        rolls.vy = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.destroy_left) != 0 {
        let spec = def.destroy_left.clone();

        rolls.destroy_left = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.destroy_right) != 0 {
        let spec = def.destroy_right.clone();

        rolls.destroy_right = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.destroy_top) != 0 {
        let spec = def.destroy_top.clone();

        rolls.destroy_top = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.destroy_bottom) != 0 {
        let spec = def.destroy_bottom.clone();

        rolls.destroy_bottom = bg_param_roll_float(ctx, &spec, &mut rolls.groups, rolls.model, 0.0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.frame) != 0 {
        let spec = def.frame.clone();

        rolls.frame = bg_param_roll_int(ctx, &spec, &mut rolls.groups, rolls.model, 0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.wait) != 0 {
        let spec = def.wait.clone();

        rolls.wait = bg_param_roll_int(ctx, &spec, &mut rolls.groups, rolls.model, 0)?;
    }

    let def = ctx.bg_effects.defs.get(rolls.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: rolls.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

    if first_spawn != 0 || bg_param_enabled(&def.life_time) != 0 {
        let spec = def.life_time.clone();

        rolls.life_time = bg_param_roll_int(ctx, &spec, &mut rolls.groups, rolls.model, 0)?;
    }

    if let Some(slot) = ctx.bg_effects.instances.get_mut(instance) {
        *slot = rolls;
    }

    Ok(())
}
