mod arena;
mod battle;
mod bridge;
mod protocol;
mod viewer;
mod worker;

use dioxus::LaunchBuilder;
use dioxus::logger::tracing::Level;
use wasm_bindgen::JsCast;
use web_sys::DedicatedWorkerGlobalScope;

use omote::Platform;

use bridge::WebHost;

fn main() {
    if let Ok(scope) = js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>() {
        worker::run(scope);
        return;
    }

    console_error_panic_hook::set_once();

    let _ = dioxus::logger::init(Level::INFO);

    LaunchBuilder::new().with_context_provider(|| Box::new(Platform::new(WebHost::spawn()))).launch(omote::App);
}
