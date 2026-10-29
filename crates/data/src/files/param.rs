use std::collections::HashMap;
use std::fmt;

use crate::common::file;

/// Represents an error encountered during the parameter deserialization phase.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParamError {
    /// Indicates that the provided byte payload yielded no actionable key-value pairs.
    EmptyFile,
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::EmptyFile => write!(f, "The provided file bytes contained no valid param data."),
        }
    }
}

impl std::error::Error for ParamError {}

/// The engine's global tuning parameters.
///
/// Raw integer constants governing combat, interface, and event behavior. Many
/// columns have no established meaning and are named positionally.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Param {
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_1: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_2: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_3: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_4: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_5: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_6: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_7: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_8: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_9: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_10: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_11: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_12: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_13: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_14: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_15: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_16: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_17: i32,
    /// The delay in frames before holding a unit opens its detail panel.
    pub hold_cat_details_time: i32,
    /// The delay in frames between an entity dying and its death surge appearing.
    pub time_until_death_surge: i32,
    /// The attack multiplier Behemoth Slayer applies against Behemoth targets.
    pub behemoth_slayer_attack_multiplier: i32,
    /// The damage reduction Behemoth Slayer applies against Behemoth targets.
    pub behemoth_slayer_defense_multiplier: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_22: i32,
    /// The interval in frames between attempts to play the alternate title screen animation.
    pub logo_superfeline_roll_timer: i32,
    /// The percentage chance that an attempt to play the alternate title screen animation succeeds.
    pub logo_superfeline_chance_success: i32,
    /// The percentage added to the chance after each failed attempt.
    pub logo_superfeline_failure_increase: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_26: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_27: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_28: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_29: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_30: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_31: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_32: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_33: i32,
    /// The output level applied at the lowest volume setting.
    pub volume_setting_percent_small: i32,
    /// The output level applied at the middle volume setting.
    pub volume_setting_percent_medium: i32,
    /// The output level applied at the highest volume setting.
    pub volume_setting_percent_large: i32,
    /// The horizontal offset applied to a conjured spirit relative to its summoner.
    pub spirit_position_offset: i32,
    /// The weaken duration reduction the Sage trait confers.
    pub sage_type_resist_weaken: i32,
    /// The freeze duration reduction the Sage trait confers.
    pub sage_type_resist_freeze: i32,
    /// The slow duration reduction the Sage trait confers.
    pub sage_type_resist_slow: i32,
    /// The curse duration reduction the Sage trait confers.
    pub sage_type_resist_curse: i32,
    /// The knockback chance reduction the Sage trait confers.
    pub sage_type_resist_knockback: i32,
    /// The weaken duration reduction Sage Slayer confers against Sage targets.
    pub sage_slayer_resist_weaken: i32,
    /// The freeze duration reduction Sage Slayer confers against Sage targets.
    pub sage_slayer_resist_freeze: i32,
    /// The slow duration reduction Sage Slayer confers against Sage targets.
    pub sage_slayer_resist_slow: i32,
    /// The curse duration reduction Sage Slayer confers against Sage targets.
    pub sage_slayer_resist_curse: i32,
    /// The reduction Sage Slayer confers against Sage effects not covered by a dedicated field.
    pub sage_slayer_resist_other: i32,
    /// The warp duration reduction Sage Slayer confers against Sage targets.
    pub sage_slayer_resist_warp: i32,
    /// The attack multiplier Sage Slayer applies against Sage targets.
    pub sage_slayer_attack_multiplier: i32,
    /// The damage reduction Sage Slayer applies against Sage targets.
    pub sage_slayer_defense_multiplier: i32,
    /// The rule selecting how Metal Killer computes damage from a target's health.
    pub metal_killer_hitpoint_behavior: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_52: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_53: i32,
    /// The playback rate applied while a story sequence is being held to fast-forward.
    pub story_sequence_hold_speed: i32,
    /// The delay in frames between an explosion being created and becoming active.
    pub explosion_start_time: i32,
    /// The number of frames an explosion deals damage for.
    pub explosion_hitbox_duration: i32,
    /// The delay in frames between an explosion ending and its removal.
    pub explosion_delay_before_deletion: i32,
    /// The delay in frames before an explosion produces its spike effect.
    pub explosion_delay_until_spike: i32,
    /// The hitbox width of a large explosion.
    pub explosion_big_width: i32,
    /// The hitbox width of a medium explosion.
    pub explosion_medium_width: i32,
    /// The hitbox width of a small explosion.
    pub explosion_small_width: i32,
    /// The damage multiplier applied by a large explosion.
    pub explosion_big_damage_multiplier: i32,
    /// The damage multiplier applied by a medium explosion.
    pub explosion_medium_damage_multiplier: i32,
    /// The damage multiplier applied by a small explosion.
    pub explosion_small_damage_multiplier: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_65: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_66: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_67: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_68: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_69: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_70: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_71: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_72: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_73: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_74: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_75: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_76: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_77: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_78: i32,
    /// The number of days the returning-player treasure festival runs for.
    pub comeback_treasure_festival_days: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_80: i32,
    /// A parameter present in the raw data whose effect on the engine is not established.
    pub unknown_81: i32,
    /// Any trailing columns beyond the known layout, retained verbatim.
    pub rest: Vec<i32>,
}

impl Param {
    /// Parses the global parameter file into its constants.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the parameter file.
    ///
    /// # Returns
    /// A `Result` containing the populated `Param` on success, or a `ParamError`
    /// if the payload held no usable values.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, ParamError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Param, ParamError> {
    let file_content = file::scrub(bytes);

    let mut map = HashMap::with_capacity(150);
    let mut raw_ordered_values = Vec::with_capacity(150);

    for line in file_content.lines() {
        let clean_line = line.split("//").next().unwrap_or("").trim();

        if clean_line.is_empty() {
            continue;
        }

        let mut parts = clean_line.split_whitespace();
        let Some(key) = parts.next() else { continue; };
        let Some(val_str) = parts.last() else { continue; };

        let val = val_str.parse::<i32>().unwrap_or(0);

        map.insert(key, val);
        raw_ordered_values.push(val);
    }

    if map.is_empty() {
        return Err(ParamError::EmptyFile);
    }

    let get_val = |target_key: &str| -> i32 {
        map.get(target_key).copied().unwrap_or(0)
    };

    let mut rest = Vec::new();
    if raw_ordered_values.len() > 81 {
        rest.extend_from_slice(&raw_ordered_values[81..]);
    }

    Ok(Param {
        unknown_1: get_val("slot_font_size"),
        unknown_2: get_val("slot_bg_y"),
        unknown_3: get_val("slot_name_margin_top"),
        unknown_4: get_val("slot_name_margin_left"),
        unknown_5: get_val("slot_page_y_offset"),
        unknown_6: get_val("slot_font_size2"),
        unknown_7: get_val("slot_name_margin_top2"),
        unknown_8: get_val("slot_name_margin_left2"),
        unknown_9: get_val("battle_wave_s_st"),
        unknown_10: get_val("battle_wave_s_inter"),
        unknown_11: get_val("battle_wave_s_frame1"),
        unknown_12: get_val("battle_wave_s_frame2"),
        unknown_13: get_val("battle_slot_2lines_line"),
        unknown_14: get_val("battle_slot_2lines_y"),
        unknown_15: get_val("battle_zoom_y"),
        unknown_16: get_val("battle_zoom_slot_y"),
        unknown_17: get_val("battle_zoom_castle_y"),
        hold_cat_details_time: get_val("battle_slot_desc_wait"),
        time_until_death_surge: get_val("battle_death_volcano_time"),
        behemoth_slayer_attack_multiplier: get_val("battle_super_beast_hunter"),
        behemoth_slayer_defense_multiplier: get_val("battle_super_beast_hunter_df"),
        unknown_22: get_val("boost_speed"),
        logo_superfeline_roll_timer: get_val("logo_gimmick_sec"),
        logo_superfeline_chance_success: get_val("logo_gimmick_probability"),
        logo_superfeline_failure_increase: get_val("logo_gimmick_increase"),
        unknown_26: get_val("battle_demon_summon_frame"),
        unknown_27: get_val("battle_demon_summon_se_frame"),
        unknown_28: get_val("battle_castle_x0"),
        unknown_29: get_val("battle_castle_y0"),
        unknown_30: get_val("battle_castle_x1"),
        unknown_31: get_val("battle_castle_y1"),
        unknown_32: get_val("battle_castle_x2"),
        unknown_33: get_val("battle_castle_y2"),
        volume_setting_percent_small: get_val("volume_percentage_s"),
        volume_setting_percent_medium: get_val("volume_percentage_m"),
        volume_setting_percent_large: get_val("volume_percentage_l"),
        spirit_position_offset: get_val("summon_position"),
        sage_type_resist_weaken: get_val("battle_super_sage_weaken"),
        sage_type_resist_freeze: get_val("battle_super_sage_freeze"),
        sage_type_resist_slow: get_val("battle_super_sage_slow"),
        sage_type_resist_curse: get_val("battle_super_sage_curse"),
        sage_type_resist_knockback: get_val("battle_super_sage_knockback"),
        sage_slayer_resist_weaken: get_val("battle_super_sage_hunter_weaken"),
        sage_slayer_resist_freeze: get_val("battle_super_sage_hunter_freeze"),
        sage_slayer_resist_slow: get_val("battle_super_sage_hunter_slow"),
        sage_slayer_resist_curse: get_val("battle_super_sage_hunter_curse"),
        sage_slayer_resist_other: get_val("battle_super_sage_hunter_knockback"),
        sage_slayer_resist_warp: get_val("battle_super_sage_hunter_warp"),
        sage_slayer_attack_multiplier: get_val("battle_super_sage_hunter_damage1"),
        sage_slayer_defense_multiplier: get_val("battle_super_sage_hunter_damage2"),
        metal_killer_hitpoint_behavior: get_val("battle_metal_killer_hp_type"),
        unknown_52: get_val("catbase_banner1"),
        unknown_53: get_val("catbase_banner2"),
        story_sequence_hold_speed: get_val("story_fast_forward_speed"),
        explosion_start_time: get_val("battle_explosion_frame1"),
        explosion_hitbox_duration: get_val("battle_explosion_frame2"),
        explosion_delay_before_deletion: get_val("battle_explosion_frame3"),
        explosion_delay_until_spike: get_val("battle_explosion_frame4"),
        explosion_big_width: get_val("battle_explosion_width1"),
        explosion_medium_width: get_val("battle_explosion_width2"),
        explosion_small_width: get_val("battle_explosion_width3"),
        explosion_big_damage_multiplier: get_val("battle_explosion_damage1"),
        explosion_medium_damage_multiplier: get_val("battle_explosion_damage2"),
        explosion_small_damage_multiplier: get_val("battle_explosion_damage3"),
        unknown_65: get_val("battle_af_death_volcano_distance"),
        unknown_66: get_val("battle_af_death_volcano_width"),
        unknown_67: get_val("battle_af_condition_production_0"),
        unknown_68: get_val("battle_af_condition_production_1"),
        unknown_69: get_val("battle_af_condition_production_2"),
        unknown_70: get_val("battle_af_condition_production_3"),
        unknown_71: get_val("battle_af_condition_production_4"),
        unknown_72: get_val("battle_af_condition_production_5"),
        unknown_73: get_val("battle_af_condition_production_type"),
        unknown_74: get_val("autoset_level_param"),
        unknown_75: get_val("autoset_must_param"),
        unknown_76: get_val("autoset_cut_score_rate"),
        unknown_77: get_val("autoset_non_target_offset"),
        unknown_78: get_val("battle_powerup_kill_enemy"),
        comeback_treasure_festival_days: get_val("comeback_treasure_day"),
        unknown_80: get_val("update_version_notification_interval"),
        unknown_81: get_val("battle_sentai_recast"),
        rest,
    })
}