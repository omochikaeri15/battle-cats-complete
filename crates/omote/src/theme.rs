use std::fmt::Write;

use dioxus::prelude::*;

pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;
const CONTROLS_PANEL_SHADE: f32 = 0.15;
const CONTROLS_PANEL_ALPHA: f32 = 160.0 / 255.0;
const VIEWER_SHADE: f32 = 0.6;
const POPUP_CHROME_SHADE: f32 = 0.7;
pub const WEAK_TEXT_ALPHA: f32 = 0.4;
pub const HEADER_SEPARATOR: &str = " :: ";
pub const TEXT_SIZE: f32 = 16.0;

pub const SIDEBAR_WIDTH: f32 = 180.0;
pub const SIDEBAR_PADDING: f32 = 15.0;
pub const SIDEBAR_BAR_WIDTH: f32 = 6.0;
pub const SIDEBAR_BAR_MARGIN: f32 = 4.0;
pub const TAB_TEXT_SIZE: f32 = 16.0;
pub const TAB_HEIGHT: f32 = 42.0;
pub const TAB_SPACING: f32 = 6.0;
pub const NAV_TOGGLE_SIZE: f32 = 37.0;
pub const NAV_TOGGLE_TOP: f32 = 2.5;
pub const NAV_TOGGLE_RIGHT: f32 = 10.0;
pub const SLIDE_MS: u32 = 260;
pub const TOAST_OVERHANG: f32 = 4.0;

const GLASS: f32 = 0.95;
const SIDEBAR_RADIUS: f32 = 10.0;
const SIDEBAR_SHADE: f32 = 0.35;
const SIDEBAR_BORDER_SHADE: f32 = 0.6;
const DISABLED_BUTTON_SHADE: f32 = 0.6;
const OUTLINED_SHADE: f32 = 0.78;
const OUTLINED_ALPHA: f32 = 0.35;
const DARK_PANEL_SHADE: f32 = 0.62;
const WORKSPACE_SHADE: f32 = 0.78;
const WORKSPACE_OUTLINE: f32 = 0.28;
const WORKSPACE_HEADER_SHADE: f32 = 0.5;
const CARD_SHADE: f32 = 0.15;
const CARD_ALPHA: f32 = 0.92;
const CARD_MUTED_ALPHA: f32 = 0.4;
const CARD_TINT_SHADE: f32 = 0.55;
const CARD_TINT_ALPHA: f32 = 0.5;
const CARD_PRIMARY_EDGE: f32 = 0.35;
const CARD_OUTLINE_LIGHTEN: f32 = 0.4;
const INPUT_SHADE: f32 = 0.5;
const HOVER_ALPHA: f32 = 0.8;
const LIST_HOVER_ALPHA: f32 = 0.15;
const PROGRESS_TRACK_SHADE: f32 = 0.55;
const TABLE_HEADER_SHADE: f32 = 0.55;
const TABLE_ROW_SHADE: f32 = 0.8;
const CONSOLE_BORDER_SHADE: f32 = 0.25;
const CONSOLE_BACKGROUND_SHADE: f32 = 0.4;
const HINT_BANNER_ALPHA: f32 = 160.0 / 255.0;
const HINT_BANNER_SHADE: f32 = 0.15;
const INFO_BORDER_LIGHTEN: f32 = 0.25;
const NOTICE_ALPHA: f32 = 220.0 / 255.0;
const TOGGLE_KNOB_HOVER_ALPHA: f32 = 0.5;

static SYMBOLS: Asset = asset!("/assets/fonts/Miscellaneous-Symbols.ttf");
static FIRA: Asset = asset!("/assets/fonts/FiraSans-Regular.ttf");
static FIRA_BOLD: Asset = asset!("/assets/fonts/FiraSans-Bold.ttf");
pub static NOTO_JP: Asset = asset!("/assets/fonts/NotoSansJP-Regular.ttf");
pub static NOTO_KR: Asset = asset!("/assets/fonts/NotoSansKR-Regular.ttf");
pub static NOTO_TC: Asset = asset!("/assets/fonts/NotoSansTC-Regular.ttf");
pub static NOTO_TH: Asset = asset!("/assets/fonts/NotoSansThai-Regular.ttf");
pub static GAME_FONTS: [Asset; 6] = [
    asset!("/assets/fonts/Roboto-Bold.ttf"),
    asset!("/assets/fonts/NotoSansJP-Bold.otf"),
    asset!("/assets/fonts/NotoSansKR-Bold.otf"),
    asset!("/assets/fonts/NotoSansTC-Bold.otf"),
    asset!("/assets/fonts/NotoSansThai-Bold.ttf"),
    asset!("/assets/fonts/Miscellaneous-Symbols.ttf"),
];
pub const FONT_FACES: [(&str, &Asset); 4] = [("Noto Sans JP", &NOTO_JP), ("Noto Sans KR", &NOTO_KR), ("Noto Sans TC", &NOTO_TC), ("Noto Sans Thai", &NOTO_TH)];
pub static THEME: Asset = asset!("/assets/theme.css");

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AppTheme {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

pub struct Palette {
    pub background: Rgb,
    pub text: Rgb,
    pub primary: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
    pub weakest: Rgb,
    pub weaker: Rgb,
    pub weak: Rgb,
    pub strong: Rgb,
    pub stronger: Rgb,
    pub strongest: Rgb,
    pub primary_weak: Rgb,
    pub primary_strong: Rgb,
    pub primary_text: Rgb,
    pub secondary: Rgb,
    pub stat_header: Rgb,
    pub stat_value: Rgb,
    pub stat_border: Rgb,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn css(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn alpha(self, alpha: f32) -> String {
        format!("rgba({}, {}, {}, {:.3})", self.r, self.g, self.b, alpha)
    }

    pub fn channels(self) -> [f32; 3] {
        [f32::from(self.r) / 255.0, f32::from(self.g) / 255.0, f32::from(self.b) / 255.0]
    }

    pub fn shade(self, factor: f32) -> Self {
        self.map(|c| c * factor)
    }

    pub fn darken(self, factor: f32) -> Self {
        self.map(|c| c * (1.0 - factor))
    }

    pub fn lighten(self, factor: f32) -> Self {
        self.map(|c| c + (1.0 - c) * factor)
    }

    fn map(self, f: impl Fn(f32) -> f32) -> Self {
        let channel = |c: u8| (f(f32::from(c) / 255.0).clamp(0.0, 1.0) * 255.0).round() as u8;

        Self { r: channel(self.r), g: channel(self.g), b: channel(self.b) }
    }
}

impl AppTheme {
    pub const ALL: [Self; 2] = [Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.key() == key)
    }

    pub const fn palette(self) -> Palette {
        match self {
            Self::Light => Palette {
                background: Rgb::new(245, 245, 245),
                text: Rgb::new(25, 25, 25),
                primary: Rgb::new(31, 106, 165),
                success: Rgb::new(30, 130, 80),
                warning: Rgb::new(180, 130, 20),
                danger: Rgb::new(190, 40, 40),
                weakest: Rgb::new(0xeb, 0xeb, 0xeb),
                weaker: Rgb::new(0xde, 0xde, 0xde),
                weak: Rgb::new(0xd4, 0xd4, 0xd4),
                strong: Rgb::new(0xc4, 0xc4, 0xc4),
                stronger: Rgb::new(0xbc, 0xbc, 0xbc),
                strongest: Rgb::new(0xb4, 0xb4, 0xb4),
                primary_weak: Rgb::new(0xa4, 0xb3, 0xca),
                primary_strong: Rgb::new(0x00, 0x88, 0xdf),
                primary_text: Rgb::new(0xf9, 0xfa, 0xfb),
                secondary: Rgb::new(0x90, 0x90, 0x90),
                stat_header: Rgb::new(0xdb, 0xdb, 0xdb),
                stat_value: Rgb::new(0xff, 0xff, 0xff),
                stat_border: Rgb::new(0xee, 0xee, 0xee),
            },
            Self::Dark => Palette {
                background: Rgb::new(33, 33, 33),
                text: Rgb::new(230, 230, 230),
                primary: Rgb::new(31, 106, 165),
                success: Rgb::new(46, 160, 90),
                warning: Rgb::new(210, 180, 60),
                danger: Rgb::new(210, 60, 60),
                weakest: Rgb::new(0x28, 0x28, 0x28),
                weaker: Rgb::new(0x32, 0x32, 0x32),
                weak: Rgb::new(0x3a, 0x3a, 0x3a),
                strong: Rgb::new(0x47, 0x47, 0x47),
                stronger: Rgb::new(0x4e, 0x4e, 0x4e),
                strongest: Rgb::new(0x55, 0x55, 0x55),
                primary_weak: Rgb::new(0x20, 0x56, 0x84),
                primary_strong: Rgb::new(0x00, 0x88, 0xdf),
                primary_text: Rgb::new(0xf9, 0xfa, 0xfb),
                secondary: Rgb::new(0x84, 0x84, 0x84),
                stat_header: Rgb::new(0x0f, 0x0f, 0x0f),
                stat_value: Rgb::new(0x3a, 0x3a, 0x3a),
                stat_border: Rgb::new(0x2a, 0x2a, 0x2a),
            },
        }
    }

    pub fn class(self) -> String {
        format!("theme-{}", self.key())
    }
}

fn variables(theme: AppTheme) -> String {
    let p = theme.palette();
    let workspace = p.background.shade(WORKSPACE_SHADE);
    let card = p.background.shade(CARD_SHADE);
    let pairs = [
        ("bg", p.background.css()),
        ("text", p.text.css()),
        ("primary", p.primary.css()),
        ("success", p.success.css()),
        ("warning", p.warning.css()),
        ("danger", p.danger.css()),
        ("bg-weakest", p.weakest.css()),
        ("bg-weaker", p.weaker.css()),
        ("bg-weak", p.weak.css()),
        ("bg-strong", p.strong.css()),
        ("bg-stronger", p.stronger.css()),
        ("bg-strongest", p.strongest.css()),
        ("primary-weak", p.primary_weak.css()),
        ("primary-strong", p.primary_strong.css()),
        ("primary-text", p.primary_text.css()),
        ("secondary", p.secondary.css()),
        ("weak-text", p.text.alpha(WEAK_TEXT_ALPHA)),
        ("primary-hover", p.primary.alpha(HOVER_ALPHA)),
        ("success-hover", p.success.alpha(HOVER_ALPHA)),
        ("warning-hover", p.warning.alpha(HOVER_ALPHA)),
        ("danger-hover", p.danger.alpha(HOVER_ALPHA)),
        ("sidebar-bg", p.background.shade(SIDEBAR_SHADE).alpha(GLASS)),
        ("sidebar-border", p.background.shade(SIDEBAR_BORDER_SHADE).css()),
        ("outlined-bg", p.background.shade(OUTLINED_SHADE).css()),
        ("outlined-border", p.text.alpha(OUTLINED_ALPHA)),
        ("dark-panel", p.background.shade(DARK_PANEL_SHADE).css()),
        ("workspace-bg", workspace.css()),
        ("workspace-border", workspace.lighten(WORKSPACE_OUTLINE).css()),
        ("workspace-header", p.background.shade(WORKSPACE_HEADER_SHADE).css()),
        ("card-bg", card.alpha(CARD_ALPHA)),
        ("card-muted", card.alpha(CARD_ALPHA * CARD_MUTED_ALPHA)),
        ("card-danger", p.danger.darken(CARD_TINT_SHADE).alpha(CARD_TINT_ALPHA)),
        ("card-primary-border", p.primary.darken(CARD_PRIMARY_EDGE).css()),
        ("outline", p.background.lighten(CARD_OUTLINE_LIGHTEN).css()),
        ("disabled-bg", p.weak.shade(DISABLED_BUTTON_SHADE).css()),
        ("input-bg", p.background.shade(INPUT_SHADE).css()),
        ("list-hover", p.text.alpha(LIST_HOVER_ALPHA)),
        ("progress-track", p.background.shade(PROGRESS_TRACK_SHADE).css()),
        ("table-header", p.background.shade(TABLE_HEADER_SHADE).css()),
        ("table-row", p.background.shade(TABLE_ROW_SHADE).css()),
        ("console-border", p.background.shade(CONSOLE_BORDER_SHADE).css()),
        ("console-bg", p.background.shade(CONSOLE_BACKGROUND_SHADE).css()),
        ("hint-bg", p.background.shade(HINT_BANNER_SHADE).alpha(HINT_BANNER_ALPHA)),
        ("info-border", p.strong.lighten(INFO_BORDER_LIGHTEN).css()),
        ("notice-bg", Rgb::new(0, 0, 0).alpha(NOTICE_ALPHA)),
        ("toggle-knob-hover", p.primary_text.alpha(TOGGLE_KNOB_HOVER_ALPHA)),
        ("stat-header", p.stat_header.css()),
        ("stat-value", p.stat_value.css()),
        ("stat-border", p.stat_border.css()),
        ("controls-bg", p.background.shade(CONTROLS_PANEL_SHADE).alpha(CONTROLS_PANEL_ALPHA)),
        ("viewer-missing", p.background.shade(VIEWER_SHADE).css()),
        ("popup-chrome", p.background.shade(POPUP_CHROME_SHADE).css()),
    ];
    let mut block = format!(".{} {{\n", theme.class());

    for (name, value) in pairs {
        let _ = writeln!(block, "  --{name}: {value};");
    }

    block.push_str("}\n");
    block
}

pub fn stylesheet() -> String {
    let mut css = String::new();

    for theme in AppTheme::ALL {
        css.push_str(&variables(theme));
    }

    let _ = writeln!(css, "@font-face {{ font-family: \"Miscellaneous-Symbols\"; src: url(\"{SYMBOLS}\") format(\"truetype\"); font-display: block; }}");
    let _ = writeln!(css, "@font-face {{ font-family: \"Fira Sans\"; src: url(\"{FIRA}\") format(\"truetype\"); font-weight: 400; font-display: block; }}");
    let _ = writeln!(css, "@font-face {{ font-family: \"Fira Sans\"; src: url(\"{FIRA_BOLD}\") format(\"truetype\"); font-weight: 700; font-display: block; }}");

    for (family, face) in FONT_FACES {
        let _ = writeln!(css, "@font-face {{ font-family: \"{family}\"; src: url(\"{face}\") format(\"truetype\"); font-weight: 400; font-display: block; }}");
    }
    let _ = writeln!(css, ":root {{");

    let tokens: [(&str, String); 17] = [
        ("text-size", format!("{TEXT_SIZE}px")),
        ("radius-sm", format!("{RADIUS_SM}px")),
        ("radius-md", format!("{RADIUS_MD}px")),
        ("radius-lg", format!("{RADIUS_LG}px")),
        ("sidebar-width", format!("{SIDEBAR_WIDTH}px")),
        ("sidebar-padding", format!("{SIDEBAR_PADDING}px")),
        ("sidebar-radius", format!("{SIDEBAR_RADIUS}px")),
        ("sidebar-bar", format!("{SIDEBAR_BAR_WIDTH}px")),
        ("sidebar-bar-margin", format!("{SIDEBAR_BAR_MARGIN}px")),
        ("tab-text", format!("{TAB_TEXT_SIZE}px")),
        ("tab-height", format!("{TAB_HEIGHT}px")),
        ("tab-spacing", format!("{TAB_SPACING}px")),
        ("nav-toggle", format!("{NAV_TOGGLE_SIZE}px")),
        ("nav-toggle-top", format!("{NAV_TOGGLE_TOP}px")),
        ("nav-toggle-right", format!("{NAV_TOGGLE_RIGHT}px")),
        ("slide-ms", format!("{SLIDE_MS}ms")),
        ("toast-overhang", format!("{TOAST_OVERHANG}px")),
    ];

    for (name, value) in tokens {
        let _ = writeln!(css, "  --{name}: {value};");
    }

    css.push_str("}\n");
    css
}
