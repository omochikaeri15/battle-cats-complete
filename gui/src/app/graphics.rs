use std::env;
use std::sync::OnceLock;
use std::time::Instant;

use iced::wgpu;
use tracing::{info, warn};

use kore::domains::settings::{AutoGraphics, GraphicsBackend};

const WGPU_BACKEND: &str = "WGPU_BACKEND";
const ICED_BACKEND: &str = "ICED_BACKEND";
const SOFTWARE_RENDERER: &str = "tiny-skia";
const MAX_BIND_GROUPS: u32 = 2;
const MAX_NON_SAMPLER_BINDINGS: u32 = 2048;

#[cfg(windows)]
const OFFERED: [GraphicsBackend; 3] = [GraphicsBackend::DirectX12, GraphicsBackend::Vulkan, GraphicsBackend::OpenGl];
#[cfg(target_os = "macos")]
const OFFERED: [GraphicsBackend; 1] = [GraphicsBackend::Metal];
#[cfg(not(any(windows, target_os = "macos")))]
const OFFERED: [GraphicsBackend; 2] = [GraphicsBackend::Vulkan, GraphicsBackend::OpenGl];

#[cfg(windows)]
const NATIVE: wgpu::Backends = wgpu::Backends::DX12.union(wgpu::Backends::GL);
#[cfg(not(windows))]
const NATIVE: wgpu::Backends = wgpu::Backends::all();

const NAMES: [(wgpu::Backends, &str); 4] = [
    (wgpu::Backends::DX12, "dx12"),
    (wgpu::Backends::VULKAN, "vulkan"),
    (wgpu::Backends::GL, "gl"),
    (wgpu::Backends::METAL, "metal"),
];

static LAUNCH: OnceLock<Launch> = OnceLock::new();

#[derive(Clone, Copy, Debug)]
pub(crate) struct Launch {
    pub selected: GraphicsBackend,
    pub auto: AutoGraphics,
    backends: wgpu::Backends,
    software: bool,
    external: bool,
    probe_ms: Option<u128>,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub cpu: bool,
    pub software: bool,
    pub auto: AutoGraphics,
}

struct Adapter {
    name: String,
    backend: wgpu::Backend,
    cpu: bool,
}

fn backends_of(backend: GraphicsBackend) -> Option<wgpu::Backends> {
    match backend {
        GraphicsBackend::DirectX12 => Some(wgpu::Backends::DX12),
        GraphicsBackend::Vulkan => Some(wgpu::Backends::VULKAN),
        GraphicsBackend::OpenGl => Some(wgpu::Backends::GL),
        GraphicsBackend::Metal => Some(wgpu::Backends::METAL),
        GraphicsBackend::Auto | GraphicsBackend::Software => None,
    }
}

fn env_names(backends: wgpu::Backends) -> String {
    NAMES
        .iter()
        .filter(|(flag, _)| backends.contains(*flag))
        .map(|(_, name)| *name)
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn configure(selected: GraphicsBackend, cached: AutoGraphics) {
    let external = env::var_os(WGPU_BACKEND).is_some() || env::var_os(ICED_BACKEND).is_some();
    let mut launch = Launch { selected, auto: cached, backends: NATIVE, software: false, external, probe_ms: None };

    if external {
        launch.backends = wgpu::Backends::from_env().unwrap_or(NATIVE);
        launch.software = env::var(ICED_BACKEND).is_ok_and(|renderer| renderer.starts_with(SOFTWARE_RENDERER));
    } else {
        match selected {
            GraphicsBackend::Auto => {
                if cached == AutoGraphics::Unprobed {
                    let started = Instant::now();

                    launch.auto = resolve();
                    launch.probe_ms = Some(started.elapsed().as_millis());
                }

                launch.backends = match launch.auto {
                    AutoGraphics::Fallback(backend) => backends_of(backend).unwrap_or(NATIVE),
                    AutoGraphics::Unprobed | AutoGraphics::Native => NATIVE,
                };
            }
            GraphicsBackend::Software => launch.software = true,
            explicit => launch.backends = backends_of(explicit).unwrap_or(NATIVE),
        }

        if launch.software {
            unsafe { env::set_var(ICED_BACKEND, SOFTWARE_RENDERER) };
        } else if launch.backends != wgpu::Backends::all() {
            unsafe { env::set_var(WGPU_BACKEND, env_names(launch.backends)) };
        }
    }

    let _ = LAUNCH.set(launch);
}

pub(crate) fn launch() -> Launch {
    LAUNCH.get().copied().unwrap_or(Launch {
        selected: GraphicsBackend::Auto,
        auto: AutoGraphics::Unprobed,
        backends: NATIVE,
        software: false,
        external: false,
        probe_ms: None,
    })
}

pub(crate) fn log_launch() {
    let launch = launch();

    info!(
        selected = %launch.selected,
        auto = ?launch.auto,
        backends = env_names(launch.backends),
        software = launch.software,
        external = launch.external,
        probe_ms = ?launch.probe_ms,
        "Graphics backend chosen for this launch"
    );
}

pub(crate) fn offered(available: Option<&[GraphicsBackend]>, selected: GraphicsBackend) -> Vec<GraphicsBackend> {
    let backends = OFFERED.into_iter().filter(|backend| available.is_none_or(|available| available.contains(backend) || *backend == selected));

    [GraphicsBackend::Auto].into_iter().chain(backends).chain([GraphicsBackend::Software]).collect()
}

pub(crate) fn survey() -> Report {
    let launch = launch();
    let active = if launch.software {
        None
    } else {
        probe(launch.backends)
            .inspect_err(|reason| warn!("The graphics adapter could not be used, so the app is drawn in software: {reason}"))
            .ok()
    };

    if let Some(adapter) = &active {
        info!(adapter = adapter.name, backend = ?adapter.backend, cpu = adapter.cpu, "Graphics adapter in use");
    } else {
        info!("The app is drawn by the software renderer");
    }

    let auto = match (launch.selected, launch.external) {
        (GraphicsBackend::Auto, false) if launch.backends == NATIVE && active.is_some() => AutoGraphics::Native,
        (GraphicsBackend::Auto, false) => resolve(),
        _ => launch.auto,
    };

    Report { cpu: active.as_ref().is_none_or(|adapter| adapter.cpu), software: active.is_none(), auto }
}

pub(crate) fn available() -> Vec<GraphicsBackend> {
    let available: Vec<GraphicsBackend> = OFFERED
        .into_iter()
        .filter(|backend| {
            backends_of(*backend).is_some_and(|backends| {
                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..wgpu::InstanceDescriptor::default() });

                !instance.enumerate_adapters(backends).is_empty()
            })
        })
        .collect();

    info!(?available, "Graphics backends with an adapter on this PC");

    available
}

fn resolve() -> AutoGraphics {
    if probe(NATIVE).is_ok() {
        return AutoGraphics::Native;
    }

    OFFERED
        .into_iter()
        .find(|backend| backends_of(*backend).is_some_and(|backends| probe(backends).is_ok()))
        .map_or(AutoGraphics::Native, AutoGraphics::Fallback)
}

fn probe(backends: wgpu::Backends) -> Result<Adapter, String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends, ..wgpu::InstanceDescriptor::default() });
    let options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::HighPerformance),
        compatible_surface: None,
        force_fallback_adapter: false,
    };
    let adapter = smol::block_on(instance.request_adapter(&options)).map_err(|error| format!("no adapter on {}: {error}", env_names(backends)))?;
    let info = adapter.get_info();
    let mut failures = Vec::new();

    for limits in [wgpu::Limits::default(), wgpu::Limits::downlevel_defaults()] {
        let required_limits = wgpu::Limits { max_bind_groups: MAX_BIND_GROUPS, max_non_sampler_bindings: MAX_NON_SAMPLER_BINDINGS, ..limits };
        let requested = smol::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits,
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }));

        let Err(error) = requested else {
            return Ok(Adapter { name: info.name, backend: info.backend, cpu: info.device_type == wgpu::DeviceType::Cpu });
        };

        failures.push(error.to_string());
    }

    Err(format!("{} on {:?} refused a device: {}", info.name, info.backend, failures.join("; ")))
}
