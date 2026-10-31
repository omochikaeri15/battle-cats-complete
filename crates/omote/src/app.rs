use dioxus::prelude::*;

use kokoro::domains::settings::Settings as GameSettings;

use crate::domains::cat::{self, Cats};
use crate::domains::enemy::{self, Enemies};
use crate::domains::home::Home;
use crate::domains::sandbox::{self, Sandbox};
use crate::domains::settings::Settings;
use crate::domains::stage::{self, Stages};
use crate::platform::Platform;
use crate::systems::library::Library;
use crate::systems::{animation, catalog, context, export, images};
use crate::theme::{self, AppTheme};

static ICON: Asset = asset!("/assets/icon.ico");
const THEME_KEY: &str = "theme";
const SETTINGS_KEY: &str = "settings";
const OPEN_ARROW: &str = "\u{25b6}";
const CLOSED_ARROW: &str = "\u{25c0}";

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/cats/:..segments")]
        Cats { segments: Vec<String> },
        #[route("/enemies/:..segments")]
        Enemies { segments: Vec<String> },
        #[route("/stages")]
        Stages {},
        #[route("/sandbox")]
        Sandbox {},
        #[route("/settings")]
        Settings {},
}

const TABS: [Route; 6] = [Route::Home {}, Route::Sandbox {}, Route::Cats { segments: Vec::new() }, Route::Enemies { segments: Vec::new() }, Route::Stages {}, Route::Settings {}];

impl Route {
    pub fn cat(id: u32) -> Self {
        Self::Cats { segments: vec![format!("{id:03}")] }
    }

    pub fn enemy(id: u32) -> Self {
        Self::Enemies { segments: vec![format!("{id:03}")] }
    }

    pub fn picked(segments: &[String]) -> Option<u32> {
        segments.first()?.parse().ok()
    }

    pub fn tab_name(&self) -> &'static str {
        match self {
            Self::Home {} => "Home",
            Self::Cats { .. } => "Cats",
            Self::Enemies { .. } => "Enemies",
            Self::Stages {} => "Stages",
            Self::Sandbox {} => "Sandbox",
            Self::Settings {} => "Settings",
        }
    }
}

pub fn remember_settings(platform: &Platform, settings: &GameSettings) {
    if let Ok(json) = serde_json::to_string(settings) {
        platform.host().remember(SETTINGS_KEY, &json);
    }
}

#[component]
pub fn App() -> Element {
    let platform = use_context::<Platform>();
    let saved = platform.host().recall(THEME_KEY).and_then(|key| AppTheme::from_key(&key)).unwrap_or_default();

    use_context_provider(|| Signal::new(saved));

    let restored = platform.host().recall(SETTINGS_KEY).and_then(|json| serde_json::from_str::<GameSettings>(&json).ok()).unwrap_or_default();
    let settings = use_context_provider(|| Signal::new(restored));
    images::provide(&platform);
    animation::provide_rigs(&platform);

    sandbox::provide(&platform);

    let library = Library::provide(platform);

    context::provide(library, settings);
    catalog::provide();
    export::provide();
    cat::provide();
    animation::provide();
    enemy::provide();
    stage::provide();

    rsx! {
        document::Link { rel: "icon", href: ICON }
        for (family, face) in theme::FONT_FACES {
            document::Link { key: "{family}", rel: "preload", r#as: "font", r#type: "font/ttf", href: *face, crossorigin: "anonymous" }
        }
        document::Style { {theme::stylesheet()} }
        document::Link { rel: "stylesheet", href: theme::THEME }
        Router::<Route> {}
    }
}

#[component]
fn Shell() -> Element {
    let theme = use_context::<Signal<AppTheme>>();
    let mut open = use_signal(|| true);
    let route = use_route::<Route>();
    let class = format!("app {}", theme().class());
    let slot = if open() { "sidebar-slot" } else { "sidebar-slot closed" };
    let arrow = if open() { OPEN_ARROW } else { CLOSED_ARROW };

    rsx! {
        div { class: "{class}",
            div { class: "content", Outlet::<Route> {} }
            div { class: "sidebar-layer",
                button { class: "btn btn-primary nav-toggle symbols", onclick: move |_| open.toggle(), "{arrow}" }
                div { class: "{slot}",
                    nav { class: "sidebar",
                        div { class: "sidebar-scroll",
                            div { class: "tabs",
                                for tab in TABS {
                                    Link {
                                        to: tab.clone(),
                                        class: if route.tab_name() == tab.tab_name() { "tab active" } else { "tab" },
                                        "{tab.tab_name()}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
