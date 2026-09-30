//! Sprite indices into the `img015` ability icon atlas.
//!
//! The engine addresses ability icons by their position within a shared
//! atlas rather than by name. These constants give those positions stable
//! identifiers so that ability definitions can reference an icon without
//! embedding a bare number.

/// The sprite index of the icon marking resistance to weaken.
pub const ICON_RESIST_WEAKEN: usize = 43;
/// The sprite index of the icon marking resistance to freeze.
pub const ICON_RESIST_FREEZE: usize = 45;
/// The sprite index of the icon marking resistance to slow.
pub const ICON_RESIST_SLOW: usize = 47;
/// The sprite index of the icon marking resistance to knockback.
pub const ICON_RESIST_KNOCKBACK: usize = 49;
/// The sprite index of the icon marking resistance to wave.
pub const ICON_RESIST_WAVE: usize = 51;
/// The sprite index of the icon marking resistance to warp.
pub const ICON_RESIST_WARP: usize = 53;
/// The sprite index of the cost down icon.
pub const ICON_COST_DOWN: usize = 92;
/// The sprite index of the recover speed up icon.
pub const ICON_RECOVER_SPEED_UP: usize = 94;
/// The sprite index of the move speed icon.
pub const ICON_MOVE_SPEED: usize = 96;
/// The sprite index of the improve knockback count icon.
pub const ICON_IMPROVE_KNOCKBACK_COUNT: usize = 98;
/// The sprite index of the icon marking resistance to curse.
pub const ICON_RESIST_CURSE: usize = 109;
/// The sprite index of the Eva Angel killer icon.
pub const ICON_EVA_KILLER: usize = 110;
/// The sprite index of the omni strike icon.
pub const ICON_OMNI_STRIKE: usize = 112;
/// The sprite index of the insane damage icon.
pub const ICON_INSANE_DAMAGE: usize = 114;
/// The sprite index of the icon marking immunity to curse.
pub const ICON_IMMUNE_CURSE: usize = 116;
/// The sprite index of the attack buff icon.
pub const ICON_ATTACK_BUFF: usize = 118;
/// The sprite index of the health buff icon.
pub const ICON_HEALTH_BUFF: usize = 120;
/// The sprite index of the insanely tough icon.
pub const ICON_INSANELY_TOUGH: usize = 122;
/// The sprite index of the weaken icon.
pub const ICON_WEAKEN: usize = 195;
/// The sprite index of the strengthen icon.
pub const ICON_STRENGTHEN: usize = 196;
/// The sprite index of the freeze icon.
pub const ICON_FREEZE: usize = 197;
/// The sprite index of the slow icon.
pub const ICON_SLOW: usize = 198;
/// The sprite index of the survive icon.
pub const ICON_SURVIVE: usize = 199;
/// The sprite index of the base destroyer icon.
pub const ICON_BASE_DESTROYER: usize = 200;
/// The sprite index of the critical hit icon.
pub const ICON_CRITICAL_HIT: usize = 201;
/// The sprite index of the attack only icon.
pub const ICON_ATTACK_ONLY: usize = 202;
/// The sprite index of the strong against icon.
pub const ICON_STRONG_AGAINST: usize = 203;
/// The sprite index of the resist icon.
pub const ICON_RESIST: usize = 204;
/// The sprite index of the double bounty icon.
pub const ICON_DOUBLE_BOUNTY: usize = 205;
/// The sprite index of the massive damage icon.
pub const ICON_MASSIVE_DAMAGE: usize = 206;
/// The sprite index of the knockback icon.
pub const ICON_KNOCKBACK: usize = 207;
/// The sprite index of the wave icon.
pub const ICON_WAVE: usize = 208;
/// The sprite index of the metal icon.
pub const ICON_METAL: usize = 209;
/// The sprite index of the icon marking immunity to wave.
pub const ICON_IMMUNE_WAVE: usize = 210;
/// The sprite index of the area attack icon.
pub const ICON_AREA_ATTACK: usize = 211;
/// The sprite index of the long distance icon.
pub const ICON_LONG_DISTANCE: usize = 212;
/// The sprite index of the icon marking immunity to weaken.
pub const ICON_IMMUNE_WEAKEN: usize = 213;
/// The sprite index of the icon marking immunity to freeze.
pub const ICON_IMMUNE_FREEZE: usize = 214;
/// The sprite index of the icon marking immunity to slow.
pub const ICON_IMMUNE_SLOW: usize = 215;
/// The sprite index of the icon marking immunity to knockback.
pub const ICON_IMMUNE_KNOCKBACK: usize = 216;
/// The sprite index of the single attack icon.
pub const ICON_SINGLE_ATTACK: usize = 217;
/// The sprite index of the wave block icon.
pub const ICON_WAVE_BLOCK: usize = 218;
/// The sprite index of the trait red icon.
pub const ICON_TRAIT_RED: usize = 219;
/// The sprite index of the trait floating icon.
pub const ICON_TRAIT_FLOATING: usize = 220;
/// The sprite index of the trait black icon.
pub const ICON_TRAIT_BLACK: usize = 221;
/// The sprite index of the trait metal icon.
pub const ICON_TRAIT_METAL: usize = 222;
/// The sprite index of the trait angel icon.
pub const ICON_TRAIT_ANGEL: usize = 223;
/// The sprite index of the trait alien icon.
pub const ICON_TRAIT_ALIEN: usize = 224;
/// The sprite index of the trait zombie icon.
pub const ICON_TRAIT_ZOMBIE: usize = 225;
/// The sprite index of the trait relic icon.
pub const ICON_TRAIT_RELIC: usize = 226;
/// The sprite index of the trait traitless icon.
pub const ICON_TRAIT_TRAITLESS: usize = 227;
/// The sprite index of the savage blow icon.
pub const ICON_SAVAGE_BLOW: usize = 229;
/// The sprite index of the dodge icon.
pub const ICON_DODGE: usize = 231;
/// The sprite index of the icon marking resistance to toxic.
pub const ICON_RESIST_TOXIC: usize = 235;
/// The sprite index of the icon marking immunity to toxic.
pub const ICON_IMMUNE_TOXIC: usize = 237;
/// The sprite index of the surge icon.
pub const ICON_SURGE: usize = 239;
/// The sprite index of the surge resist icon.
pub const ICON_SURGE_RESIST: usize = 241;
/// The sprite index of the icon marking immunity to surge.
pub const ICON_IMMUNE_SURGE: usize = 243;
/// The sprite index of the witch killer icon.
pub const ICON_WITCH_KILLER: usize = 258;
/// The sprite index of the zombie killer icon.
pub const ICON_ZOMBIE_KILLER: usize = 260;
/// The sprite index of the icon marking immunity to warp.
pub const ICON_IMMUNE_WARP: usize = 262;
/// The sprite index of the barrier breaker icon.
pub const ICON_BARRIER_BREAKER: usize = 264;
/// The sprite index of the warp icon.
pub const ICON_WARP: usize = 266;
/// The sprite index of the empty icon.
pub const ICON_EMPTY: usize = 270;
/// The sprite index of the small gold frame border.
pub const BORDER_GOLD_SMALL: usize = 271;
/// The sprite index of the red frame border.
pub const BORDER_RED: usize = 272;
/// The sprite index of the gold frame border.
pub const BORDER_GOLD: usize = 273;
/// The sprite index of the curse icon.
pub const ICON_CURSE: usize = 289;
/// The sprite index of the mini wave icon.
pub const ICON_MINI_WAVE: usize = 293;
/// The sprite index of the trait aku icon.
pub const ICON_TRAIT_AKU: usize = 294;
/// The sprite index of the shield piercer icon.
pub const ICON_SHIELD_PIERCER: usize = 296;
/// The sprite index of the colossus slayer icon.
pub const ICON_COLOSSUS_SLAYER: usize = 297;
/// The sprite index of the soulstrike icon.
pub const ICON_SOULSTRIKE: usize = 300;
/// The sprite index of the behemoth slayer icon.
pub const ICON_BEHEMOTH_SLAYER: usize = 302;
/// The sprite index of the tba down icon.
pub const ICON_TBA_DOWN: usize = 305;
/// The sprite index of the mini surge icon.
pub const ICON_MINI_SURGE: usize = 310;
/// The sprite index of the counter surge icon.
pub const ICON_COUNTER_SURGE: usize = 315;
/// The sprite index of the conjure icon.
pub const ICON_CONJURE: usize = 317;
/// The sprite index of the sage slayer icon.
pub const ICON_SAGE_SLAYER: usize = 319;
/// The sprite index of the metal killer icon.
pub const ICON_METAL_KILLER: usize = 321;
/// The sprite index of the colossus icon.
pub const ICON_COLOSSUS: usize = 324;
/// The sprite index of the behemoth icon.
pub const ICON_BEHEMOTH: usize = 325;
/// The sprite index of the sage icon.
pub const ICON_SAGE: usize = 326;
/// The sprite index of the witch icon.
pub const ICON_WITCH: usize = 327;
/// The sprite index of the Eva Angel icon.
pub const ICON_EVA: usize = 328;
/// The sprite index of the toxic icon.
pub const ICON_TOXIC: usize = 329;
/// The sprite index of the barrier icon.
pub const ICON_BARRIER: usize = 330;
/// The sprite index of the shield icon.
pub const ICON_SHIELD: usize = 331;
/// The sprite index of the death surge icon.
pub const ICON_DEATH_SURGE: usize = 332;
/// The sprite index of the explosion icon.
pub const ICON_EXPLOSION: usize = 335;
/// The sprite index of the icon marking immunity to explosion.
pub const ICON_IMMUNE_EXPLOSION: usize = 337;
/// The sprite index of the Supervillain icon.
pub const ICON_SUPERVILLIAN: usize = 384;
/// The sprite index of the icon marking resistance to explosion.
pub const ICON_RESIST_EXPLOSION: usize = 386;
/// The sprite index of the drain icon.
pub const ICON_DRAIN: usize = 389;
/// The sprite index of the drain immune icon.
pub const ICON_IMMUNE_DRAIN: usize = 391;