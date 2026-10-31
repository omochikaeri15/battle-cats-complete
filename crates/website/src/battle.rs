use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::{Float32Array, Uint8Array};
use tracing::{debug, trace, warn};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode, HtmlCanvasElement};

use omote::host::{Bounds, Canvas, Frame, Scene, Vertex, paint};
use omote::platform::battle::{AudioCommand, BattleEvent, SheetWire};

const SHADER_SOURCE: &str = r"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var sheet: texture_2d<f32>;
@group(0) @binding(1) var sheet_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(sheet, sheet_sampler, input.uv);
    return texel * input.color;
}

struct PresentOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_present(@builtin(vertex_index) index: u32) -> PresentOutput {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: PresentOutput;
    out.clip = vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(corner.x, 1.0 - corner.y);
    return out;
}

@fragment
fn fs_present(input: PresentOutput) -> @location(0) vec4<f32> {
    return textureSample(sheet, sheet_sampler, input.uv);
}
";

const CLEARED: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
const BLENDS: [(wgpu::BlendFactor, wgpu::BlendFactor); 4] = [
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::One),
    (wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrc),
];
const VERTEX_STRIDE: u64 = 32;
const MIN_PIXEL_RATIO: f64 = 0.5;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Packed {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

struct Binding {
    bind_group: wgpu::BindGroup,
}

struct Reduced {
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipelines: [wgpu::RenderPipeline; 4],
    present: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    sheets: HashMap<Box<str>, Binding>,
    blank: Binding,
    void: Binding,
    vertices: Option<wgpu::Buffer>,
    capacity: u64,
    reduced: Option<Reduced>,
}

struct Voice {
    source: AudioBufferSourceNode,
    gain: GainNode,
}

#[derive(Default)]
struct Speaker {
    context: Option<AudioContext>,
    buffers: HashMap<Box<str>, AudioBuffer>,
    decoding: HashMap<Box<str>, Vec<(u32, bool, f32)>>,
    voices: HashMap<u32, Voice>,
}

#[derive(Default)]
pub(crate) struct Screen {
    canvas: Option<HtmlCanvasElement>,
    gpu: Option<Gpu>,
    opening: bool,
    frame: Option<Frame>,
    dirty: bool,
    sizes: HashMap<Box<str>, (u32, u32)>,
    pending: Vec<SheetWire>,
    scale: f32,
    padded: bool,
    surfaced: bool,
    speaker: Speaker,
    painted: Option<Box<dyn Fn()>>,
}

pub(crate) type SharedScreen = Rc<RefCell<Screen>>;

#[wasm_bindgen(inline_js = "export function hide_renderer_info() {
    if (typeof WebGL2RenderingContext === 'undefined' || !navigator.userAgent.includes('Firefox/')) {
        return;
    }

    const prototype = WebGL2RenderingContext.prototype;
    const listed = prototype.getSupportedExtensions;

    prototype.getSupportedExtensions = function () {
        const names = listed.call(this);

        return names && names.filter((name) => name !== 'WEBGL_debug_renderer_info');
    };
}")]
extern "C" {
    #[wasm_bindgen(js_name = hide_renderer_info)]
    pub(crate) fn hide_renderer_info();
}

pub(crate) struct Darkroom {
    canvas: HtmlCanvasElement,
    gpu: Gpu,
    sizes: HashMap<Box<str>, (u32, u32)>,
    width: u32,
    height: u32,
}

impl Darkroom {
    pub(crate) async fn open(width: u32, height: u32) -> Result<Self, String> {
        let canvas: HtmlCanvasElement = web_sys::window()
            .and_then(|window| window.document())
            .ok_or("the page has no document")?
            .create_element("canvas")
            .map_err(describe)?
            .dyn_into()
            .map_err(|_| "the video canvas is not a canvas")?;
        let gpu = open(canvas.clone(), Some((width, height))).await?;

        Ok(Self { canvas, gpu, sizes: HashMap::new(), width, height })
    }

    pub(crate) fn develop(&mut self, sheets: Vec<SheetWire>) {
        for sheet in sheets {
            let binding = self.gpu.upload(sheet.width, sheet.height, &sheet.pixels);

            self.sizes.insert(sheet.name.clone(), (sheet.width, sheet.height));
            self.gpu.sheets.insert(sheet.name, binding);
        }
    }

    pub(crate) fn expose(&mut self, frame: &Frame) -> Result<(), String> {
        let bounds = Bounds { width: self.width as f32, height: self.height as f32 };
        let scene = paint(frame, &self.sizes, Canvas { surfaced: true, scale: 1.0, padded: false }, bounds);

        self.gpu.render(&scene, 1.0, false)
    }

    pub(crate) fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
}

fn describe(err: JsValue) -> String {
    err.as_string().unwrap_or_else(|| format!("{err:?}"))
}

fn find_canvas(id: &str) -> Option<HtmlCanvasElement> {
    web_sys::window()?.document()?.get_element_by_id(id)?.dyn_into::<HtmlCanvasElement>().ok()
}

fn alpha_of(source: wgpu::BlendFactor, target: wgpu::BlendFactor) -> wgpu::BlendComponent {
    let carried = matches!(target, wgpu::BlendFactor::One | wgpu::BlendFactor::Zero);

    if carried {
        wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
    } else {
        wgpu::BlendComponent { src_factor: source, dst_factor: target, operation: wgpu::BlendOperation::Add }
    }
}

fn pipelines(device: &wgpu::Device, layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, format: wgpu::TextureFormat) -> [wgpu::RenderPipeline; 4] {
    BLENDS.map(|(source, target)| {
        let component = wgpu::BlendComponent { src_factor: source, dst_factor: target, operation: wgpu::BlendOperation::Add };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("battle"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState { color: component, alpha: alpha_of(source, target) }), write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..wgpu::PrimitiveState::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    })
}

fn upload_texture(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, width: u32, height: u32, pixels: &[u8]) -> Binding {
    let size = wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("battle_sheet"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        pixels,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size.width * 4), rows_per_image: Some(size.height) },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    Binding { bind_group: bind_texture(device, layout, sampler, &view) }
}

fn bind_texture(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, view: &wgpu::TextureView) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("battle_sheet"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
        ],
    })
}

impl Gpu {
    fn upload(&self, width: u32, height: u32, pixels: &[u8]) -> Binding {
        upload_texture(&self.device, &self.queue, &self.layout, &self.sampler, width, height, pixels)
    }

    fn reduce(&mut self, scale: f32) {
        if scale >= 1.0 {
            self.reduced = None;

            return;
        }

        let width = ((self.config.width as f32 * scale).round() as u32).max(1);
        let height = ((self.config.height as f32 * scale).round() as u32).max(1);

        if self.reduced.as_ref().is_some_and(|held| held.width == width && held.height == height) {
            return;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("battle_reduced"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = bind_texture(&self.device, &self.layout, &self.sampler, &view);

        self.reduced = Some(Reduced { view, bind_group, width, height });
    }

    fn write(&mut self, vertices: &[Vertex]) {
        if vertices.is_empty() {
            return;
        }

        let packed: Vec<Packed> = vertices.iter().map(|vertex| Packed { position: vertex.position, uv: vertex.uv, color: vertex.color }).collect();
        let needed = std::mem::size_of_val(packed.as_slice()) as u64;

        if self.capacity < needed || self.vertices.is_none() {
            self.vertices = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("battle_vertices"),
                size: needed,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.capacity = needed;
        }

        if let Some(buffer) = &self.vertices {
            self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&packed));
        }
    }

    fn pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("battle"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(CLEARED), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    fn paint(&self, pass: &mut wgpu::RenderPass<'_>, scene: &Scene) {
        let Some(buffer) = &self.vertices else {
            return;
        };

        pass.set_vertex_buffer(0, buffer.slice(..));

        for run in &scene.runs {
            let binding = match &run.sheet {
                Some(name) => self.sheets.get(name.as_ref()).unwrap_or(&self.blank),
                None => &self.blank,
            };
            let Some(pipeline) = self.pipelines.get(run.blend as usize).or(self.pipelines.first()) else {
                continue;
            };

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &binding.bind_group, &[]);
            pass.draw(run.start..run.end, 0..1);
        }
    }

    fn render(&mut self, scene: &Scene, scale: f32, padded: bool) -> Result<(), String> {
        self.reduce(scale);
        self.write(&scene.vertices);

        let target = self.surface.get_current_texture().map_err(|err| format!("the battle surface had no frame: {err}"))?;
        let view = target.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("battle") });

        match &self.reduced {
            None => {
                let mut pass = Self::pass(&mut encoder, &view);

                self.paint(&mut pass, scene);
            }
            Some(reduced) => {
                {
                    let mut pass = Self::pass(&mut encoder, &reduced.view);

                    self.paint(&mut pass, scene);
                }

                let mut pass = Self::pass(&mut encoder, &view);

                pass.set_pipeline(&self.present);

                if padded {
                    pass.set_bind_group(0, &self.void.bind_group, &[]);
                    pass.draw(0..3, 0..1);

                    let left = (self.config.width.saturating_sub(reduced.width) / 2) as f32;
                    let top = (self.config.height.saturating_sub(reduced.height) / 2) as f32;

                    pass.set_viewport(left, top, reduced.width as f32, reduced.height as f32, 0.0, 1.0);
                }

                pass.set_bind_group(0, &reduced.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        target.present();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn target(canvas: HtmlCanvasElement) -> Result<wgpu::SurfaceTarget<'static>, String> {
    Ok(wgpu::SurfaceTarget::Canvas(canvas))
}

#[cfg(not(target_arch = "wasm32"))]
fn target(_canvas: HtmlCanvasElement) -> Result<wgpu::SurfaceTarget<'static>, String> {
    Err("a canvas surface only exists in a browser".to_owned())
}

fn prepare_context(canvas: &HtmlCanvasElement) {
    let navigator = web_sys::window().map(|window| window.navigator());
    let webgpu = navigator.as_ref().is_some_and(|navigator| js_sys::Reflect::has(navigator, &JsValue::from_str("gpu")).unwrap_or(false));

    if webgpu {
        debug!("The battle canvas will be a WebGPU surface");

        return;
    }

    let options = js_sys::Object::new();

    for (key, value) in [("alpha", true), ("premultipliedAlpha", true), ("antialias", false)] {
        let _ = js_sys::Reflect::set(&options, &JsValue::from_str(key), &JsValue::from_bool(value));
    }

    match canvas.get_context_with_context_options("webgl2", &options) {
        Ok(Some(context)) => {
            let attributes = js_sys::Reflect::get(&context, &JsValue::from_str("getContextAttributes"))
                .ok()
                .and_then(|method| method.dyn_into::<js_sys::Function>().ok())
                .and_then(|method| method.call0(&context).ok())
                .and_then(|attributes| js_sys::JSON::stringify(&attributes).ok())
                .map(String::from);

            debug!(attributes = ?attributes, "The battle canvas WebGL context was created with alpha");
        }
        Ok(None) => warn!("The battle canvas gave no WebGL2 context"),
        Err(err) => warn!("The battle canvas WebGL2 context could not be created: {}", describe(err)),
    }
}

fn backing(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let ratio = web_sys::window().map_or(1.0, |window| window.device_pixel_ratio()).max(MIN_PIXEL_RATIO);
    let rect = canvas.get_bounding_client_rect();

    ((rect.width() * ratio).round().max(1.0) as u32, (rect.height() * ratio).round().max(1.0) as u32)
}

async fn open(canvas: HtmlCanvasElement, sized: Option<(u32, u32)>) -> Result<Gpu, String> {
    prepare_context(&canvas);

    let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU, ..wgpu::InstanceDescriptor::default() }).await;
    let surface = instance.create_surface(target(canvas.clone())?).map_err(|err| err.to_string())?;
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..wgpu::RequestAdapterOptions::default() }).await.map_err(|err| err.to_string())?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("battle"),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            ..wgpu::DeviceDescriptor::default()
        })
        .await
        .map_err(|err| err.to_string())?;
    let (width, height) = sized.unwrap_or_else(|| backing(&canvas));

    canvas.set_width(width);
    canvas.set_height(height);

    let mut config = surface.get_default_config(&adapter, width, height).ok_or("the battle canvas is not supported by this adapter")?;

    let capabilities = surface.get_capabilities(&adapter);

    if let Some(plain) = capabilities.formats.iter().copied().find(|format| !format.is_srgb()) {
        config.format = plain;
    }

    if adapter.get_info().backend == wgpu::Backend::BrowserWebGpu || capabilities.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
    } else {
        warn!("The battle canvas cannot be composited over the page; the curtain will sweep over black");
    }

    config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    surface.configure(&device, &config);

    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("battle"), source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()) });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("battle_sheet"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("battle"), bind_group_layouts: &[&layout], push_constant_ranges: &[] });
    let pipelines = pipelines(&device, &pipeline_layout, &module, config.format);
    let present = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("battle_present"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_present"), compilation_options: wgpu::PipelineCompilationOptions::default(), buffers: &[] },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_present"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
        }),
        primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..wgpu::PrimitiveState::default() },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("battle"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..wgpu::SamplerDescriptor::default()
    });
    let blank = upload_texture(&device, &queue, &layout, &sampler, 1, 1, &[0xff, 0xff, 0xff, 0xff]);
    let void = upload_texture(&device, &queue, &layout, &sampler, 1, 1, &[0, 0, 0, 0xff]);

    Ok(Gpu { surface, device, queue, config, pipelines, present, layout, sampler, sheets: HashMap::new(), blank, void, vertices: None, capacity: 0, reduced: None })
}

impl Screen {
    pub(crate) fn attach(screen: &SharedScreen, canvas: &str) {
        let Some(canvas) = find_canvas(canvas) else {
            warn!("The battle canvas {canvas} is not on the page");

            return;
        };

        {
            let mut held = screen.borrow_mut();

            if held.canvas.as_ref().is_some_and(|held| *held == canvas) {
                return;
            }

            held.canvas = Some(canvas.clone());
            held.gpu = None;
            held.opening = true;
        }

        let screen = Rc::clone(screen);

        wasm_bindgen_futures::spawn_local(async move {
            match open(canvas, None).await {
                Ok(gpu) => {
                    let mut held = screen.borrow_mut();

                    held.gpu = Some(gpu);
                    held.opening = false;
                    held.dirty = true;
                    debug!("The battle canvas is open");
                }
                Err(reason) => {
                    screen.borrow_mut().opening = false;
                    warn!("The battle canvas could not be opened: {reason}");
                }
            }
        });
    }

    pub(crate) fn attached(&self) -> bool {
        self.canvas.is_some()
    }

    pub(crate) fn detach(&mut self) {
        self.canvas = None;
        self.gpu = None;
        self.frame = None;
        self.sizes.clear();
        self.pending.clear();
        self.speaker.silence();
    }

    pub(crate) fn display(&mut self, scale: f32, padded: bool) {
        self.scale = scale;
        self.padded = padded;
        self.dirty = true;
    }

    pub(crate) fn on_painted(&mut self, painted: Box<dyn Fn()>) {
        self.painted = Some(painted);
    }

    pub(crate) fn handle(&mut self, event: BattleEvent) {
        match event {
            BattleEvent::Frame(frame) => {
                self.frame = Some(frame);
                self.dirty = true;
            }
            BattleEvent::Sheets(sheets) => {
                for sheet in &sheets {
                    self.sizes.insert(sheet.name.clone(), (sheet.width, sheet.height));
                }

                self.pending.extend(sheets);
                self.dirty = true;
            }
            BattleEvent::Audio(commands) => {
                for command in &commands {
                    self.speaker.handle(command);
                }
            }
            BattleEvent::Standing(standing) => {
                self.surfaced = standing.surfaced;

                if standing.phase == omote::host::Phase::Idle {
                    self.frame = None;
                    self.speaker.silence();
                    self.dirty = true;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw(&mut self) {
        let Some(canvas) = self.canvas.clone() else {
            return;
        };
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        let (width, height) = backing(&canvas);

        if canvas.width() != width || canvas.height() != height {
            canvas.set_width(width);
            canvas.set_height(height);
        }

        if gpu.config.width != width || gpu.config.height != height {
            gpu.config.width = width;
            gpu.config.height = height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            self.dirty = true;
        }

        if !self.dirty {
            return;
        }

        for sheet in self.pending.drain(..) {
            let binding = gpu.upload(sheet.width, sheet.height, &sheet.pixels);

            gpu.sheets.insert(sheet.name, binding);
        }

        self.dirty = false;

        let Some(frame) = &self.frame else {
            if let Err(reason) = gpu.render(&Scene::default(), 1.0, false) {
                trace!("{reason}");
            }

            return;
        };
        let bounds = Bounds { width: width as f32, height: height as f32 };
        let canvas = Canvas { surfaced: self.surfaced, scale: self.scale, padded: self.padded };
        let scene = paint(frame, &self.sizes, canvas, bounds);
        let (scale, padded) = if self.surfaced { (self.scale, self.padded) } else { (1.0, false) };

        if let Err(reason) = gpu.render(&scene, scale, padded) {
            trace!("{reason}");

            return;
        }

        if let Some(painted) = &self.painted {
            painted();
        }
    }
}

impl Speaker {
    fn context(&mut self) -> Option<&AudioContext> {
        if self.context.is_none() {
            match AudioContext::new() {
                Ok(context) => self.context = Some(context),
                Err(err) => warn!("The battle has no audio output: {}", describe(err)),
            }
        }

        self.context.as_ref()
    }

    fn silence(&mut self) {
        for (_, voice) in self.voices.drain() {
            let _ = web_sys::AudioScheduledSourceNode::stop(&voice.source);
        }

        self.decoding.clear();
    }

    fn handle(&mut self, command: &AudioCommand) {
        match command {
            AudioCommand::Effect { clip, channels, rate, samples } => {
                let Some(context) = self.context() else {
                    return;
                };
                let channels = u32::from((*channels).max(1));
                let frames = (samples.len() as u32 / channels).max(1);
                let Ok(buffer) = context.create_buffer(channels, frames, (*rate).max(8000) as f32) else {
                    warn!("A sound buffer could not be created for {clip}");

                    return;
                };

                for channel in 0..channels {
                    let lane: Vec<f32> = samples.iter().skip(channel as usize).step_by(channels as usize).copied().collect();
                    let array = Float32Array::from(lane.as_slice());

                    if let Err(err) = buffer.copy_to_channel(&array.to_vec(), channel as i32) {
                        warn!("A sound channel could not be filled for {clip}: {}", describe(err));
                    }
                }

                self.buffers.insert(clip.clone(), buffer);
            }
            AudioCommand::Track { clip, bytes } => self.decode(clip.clone(), bytes),
            AudioCommand::Start { voice, clip, looped, level } => {
                if let Some(held) = self.voices.remove(voice) {
                    let _ = web_sys::AudioScheduledSourceNode::stop(&held.source);
                }

                if self.buffers.contains_key(clip) {
                    self.play(*voice, clip, *looped, *level);
                } else {
                    self.decoding.entry(clip.clone()).or_default().push((*voice, *looped, *level));
                }
            }
            AudioCommand::Level { voice, level } => {
                if let Some(held) = self.voices.get(voice) {
                    held.gain.gain().set_value(*level);
                }
            }
            AudioCommand::Stop { voice } => {
                if let Some(held) = self.voices.remove(voice) {
                    let _ = web_sys::AudioScheduledSourceNode::stop(&held.source);
                }

                for waiting in self.decoding.values_mut() {
                    waiting.retain(|(held, _, _)| held != voice);
                }
            }
        }
    }

    fn play(&mut self, voice: u32, clip: &str, looped: bool, level: f32) {
        let Some(buffer) = self.buffers.get(clip).cloned() else {
            return;
        };
        let Some(context) = self.context() else {
            return;
        };
        let (Ok(source), Ok(gain)) = (context.create_buffer_source(), context.create_gain()) else {
            warn!("A voice could not be created for {clip}");

            return;
        };

        source.set_buffer(Some(&buffer));
        source.set_loop(looped);
        gain.gain().set_value(level);

        if source.connect_with_audio_node(&gain).is_err() || gain.connect_with_audio_node(&context.destination()).is_err() {
            warn!("A voice could not be connected for {clip}");

            return;
        }

        if let Err(err) = source.start() {
            warn!("A voice could not start for {clip}: {}", describe(err));

            return;
        }

        self.voices.insert(voice, Voice { source, gain });
    }

    fn decode(&mut self, clip: Box<str>, bytes: &[u8]) {
        let Some(context) = self.context().cloned() else {
            return;
        };
        let array = Uint8Array::from(bytes);
        let promise = match context.decode_audio_data(&array.buffer()) {
            Ok(promise) => promise,
            Err(err) => {
                warn!("{clip} could not be decoded: {}", describe(err));

                return;
            }
        };

        wasm_bindgen_futures::spawn_local(async move {
            let decoded = JsFuture::from(promise).await.ok().and_then(|value| value.dyn_into::<AudioBuffer>().ok());

            SPEAKER_DECODED.with(|slot| slot.borrow_mut().push((clip, decoded)));
        });
    }

    fn settle(&mut self) {
        let landed: Vec<(Box<str>, Option<AudioBuffer>)> = SPEAKER_DECODED.with(|slot| std::mem::take(&mut *slot.borrow_mut()));

        for (clip, buffer) in landed {
            let Some(buffer) = buffer else {
                warn!("{clip} decoded to nothing");
                self.decoding.remove(&clip);

                continue;
            };

            self.buffers.insert(clip.clone(), buffer);

            for (voice, looped, level) in self.decoding.remove(&clip).unwrap_or_default() {
                self.play(voice, &clip, looped, level);
            }
        }
    }
}

thread_local! {
    static SPEAKER_DECODED: RefCell<Vec<(Box<str>, Option<AudioBuffer>)>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn settle_audio(screen: &SharedScreen) {
    screen.borrow_mut().speaker.settle();
}
