use std::cell::RefCell;
use std::ops::Range;

use futures_channel::oneshot;
use tracing::{error, warn};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use omote::{Atlas, Captured, Fault, Painted, Pending, Scene, Shape, Surface};

const VERTEX_STRIDE: u64 = 36;
const VERTS_PER_PART: u32 = 4;
const INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];
const INDICES_PER_PART: u32 = INDICES.len() as u32;
const FLOATS_PER_PART: usize = 36;
const TAU: f64 = std::f64::consts::TAU;
const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const LINEARIZE_TOKEN: &str = "__LINEARIZE__";
const SHADER_SOURCE: &str = r#"
const LINEARIZE: bool = __LINEARIZE__;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) opacity: f32,
    @location(3) tint: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) opacity: f32,
    @location(2) tint: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    out.opacity = input.opacity;
    out.tint = input.tint;
    return out;
}

@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(atlas, atlas_sampler, input.uv);
    let lit = dot(sampled.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let wash = mix(sampled.rgb, input.tint.rgb * lit, input.tint.a);
    let shown = select(wash, linearized(wash), LINEARIZE);

    return vec4<f32>(shown, sampled.a) * input.opacity;
}

fn linearized(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));

    return select(low, high, color > vec3<f32>(0.04045));
}
"#;

struct Batch {
    variant: usize,
    range: Range<u32>,
}

struct Bound {
    key: usize,
    bind_group: wgpu::BindGroup,
}

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipelines: [wgpu::RenderPipeline; 4],
    layout_ref: wgpu::PipelineLayout,
    capture_pipelines: Option<[wgpu::RenderPipeline; 4]>,
    sampler: wgpu::Sampler,
    layout: wgpu::BindGroupLayout,
    atlas: Option<Bound>,
    vertices: Option<wgpu::Buffer>,
    vertex_capacity: u64,
    indices: Option<wgpu::Buffer>,
    index_capacity: u64,
}

pub(crate) struct WebSurface {
    canvas: HtmlCanvasElement,
    overlay: Option<CanvasRenderingContext2d>,
    gpu: RefCell<Gpu>,
    scale: RefCell<f32>,
}

fn linear(value: f32) -> f32 {
    if value <= 0.04045 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
}

fn pipelines(device: &wgpu::Device, layout: &wgpu::PipelineLayout, format: wgpu::TextureFormat) -> [wgpu::RenderPipeline; 4] {
    let shader = SHADER_SOURCE.replace(LINEARIZE_TOKEN, if format.is_srgb() { "true" } else { "false" });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("animation_viewer"), source: wgpu::ShaderSource::Wgsl(shader.into()) });
    blend_modes().map(|blend| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("animation_viewer"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: VERTEX_STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32, 3 => Float32x4],
                }],
            },
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..wgpu::PrimitiveState::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        })
    })
}

fn padded_row(width: u32) -> u32 {
    let bytes = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    bytes.div_ceil(align) * align
}

fn blend_modes() -> [wgpu::BlendState; 4] {
    let premultiplied = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add };
    let carried_alpha = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
    let color = |src_factor, dst_factor| wgpu::BlendComponent { src_factor, dst_factor, operation: wgpu::BlendOperation::Add };

    [
        wgpu::BlendState { color: premultiplied, alpha: premultiplied },
        wgpu::BlendState { color: color(wgpu::BlendFactor::One, wgpu::BlendFactor::One), alpha: carried_alpha },
        wgpu::BlendState { color: color(wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero), alpha: carried_alpha },
        wgpu::BlendState { color: color(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrc), alpha: carried_alpha },
    ]
}

fn blend_variant(glow: u8) -> usize {
    match glow {
        1 => 1,
        2 => 2,
        3 => 3,
        _ => 0,
    }
}

fn multiply_mat3(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    [
        a[0] * b[0] + a[3] * b[1] + a[6] * b[2],
        a[1] * b[0] + a[4] * b[1] + a[7] * b[2],
        a[2] * b[0] + a[5] * b[1] + a[8] * b[2],
        a[0] * b[3] + a[3] * b[4] + a[6] * b[5],
        a[1] * b[3] + a[4] * b[4] + a[7] * b[5],
        a[2] * b[3] + a[5] * b[4] + a[8] * b[5],
        a[0] * b[6] + a[3] * b[7] + a[6] * b[8],
        a[1] * b[6] + a[4] * b[7] + a[7] * b[8],
        a[2] * b[6] + a[5] * b[7] + a[8] * b[8],
    ]
}

fn build_vertices(parts: &[Painted], view_proj: &[f32; 9]) -> (Vec<f32>, Vec<u32>, Vec<Batch>) {
    let mut vertex_data = Vec::with_capacity(parts.len() * FLOATS_PER_PART);
    let mut index_data = Vec::with_capacity(parts.len() * INDICES_PER_PART as usize);
    let mut batches: Vec<Batch> = Vec::new();

    for (part_index, painted) in parts.iter().enumerate() {
        let part = &painted.frame;
        let mvp = multiply_mat3(view_proj, &part.final_matrix);

        for corner in 0..VERTS_PER_PART as usize {
            let x = part.vertices[2 * corner];
            let y = part.vertices[2 * corner + 1];

            vertex_data.push(mvp[0] * x + mvp[3] * y + mvp[6]);
            vertex_data.push(mvp[1] * x + mvp[4] * y + mvp[7]);
            vertex_data.push(part.uvs[2 * corner]);
            vertex_data.push(part.uvs[2 * corner + 1]);
            vertex_data.push(part.opacity);
            vertex_data.extend_from_slice(&painted.tint);
        }

        let base = part_index as u32 * VERTS_PER_PART;

        index_data.extend(INDICES.iter().map(|index| base + u32::from(*index)));

        let variant = blend_variant(part.glow);
        let start = part_index as u32 * INDICES_PER_PART;

        match batches.last_mut() {
            Some(batch) if batch.variant == variant => batch.range.end = start + INDICES_PER_PART,
            _ => batches.push(Batch { variant, range: start..start + INDICES_PER_PART }),
        }
    }

    (vertex_data, index_data, batches)
}

fn css_color(color: [f32; 4]) -> String {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

    format!("rgba({}, {}, {}, {:.3})", channel(color[0]), channel(color[1]), channel(color[2]), color[3].clamp(0.0, 1.0))
}

fn describe(err: impl std::fmt::Debug) -> Fault {
    Fault(format!("{err:?}"))
}

fn find_canvas(id: &str) -> Result<HtmlCanvasElement, Fault> {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
        .ok_or_else(|| Fault(format!("the page has no element called {id}")))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| Fault(format!("{id} is not a canvas")))
}

#[cfg(target_arch = "wasm32")]
fn target(canvas: HtmlCanvasElement) -> Result<wgpu::SurfaceTarget<'static>, Fault> {
    Ok(wgpu::SurfaceTarget::Canvas(canvas))
}

#[cfg(not(target_arch = "wasm32"))]
fn target(_canvas: HtmlCanvasElement) -> Result<wgpu::SurfaceTarget<'static>, Fault> {
    Err(Fault("A canvas surface only exists in a browser".to_owned()))
}

pub(crate) async fn open(canvas_id: &str, overlay_id: &str) -> Result<WebSurface, Fault> {
    let canvas = find_canvas(canvas_id)?;
    let overlay = find_canvas(overlay_id)
        .ok()
        .and_then(|overlay| overlay.get_context("2d").ok().flatten())
        .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok());

    if overlay.is_none() {
        warn!("The overlay canvas has no 2d context, rig and origin marks will not be drawn");
    }

    let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
        ..wgpu::InstanceDescriptor::default()
    })
    .await;
    let surface = instance.create_surface(target(canvas.clone())?).map_err(describe)?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..wgpu::RequestAdapterOptions::default() })
        .await
        .map_err(describe)?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("animation_viewer"),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            ..wgpu::DeviceDescriptor::default()
        })
        .await
        .map_err(describe)?;
    let width = canvas.width().max(1);
    let height = canvas.height().max(1);
    let mut config = surface.get_default_config(&adapter, width, height).ok_or_else(|| Fault("the canvas surface is not supported by this adapter".to_owned()))?;

    if let Some(plain) = surface.get_capabilities(&adapter).formats.iter().copied().find(|format| !format.is_srgb()) {
        config.format = plain;
    }

    config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    surface.configure(&device, &config);

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("animation_viewer"),
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
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("animation_viewer"), bind_group_layouts: &[&layout], push_constant_ranges: &[] });
    let pipelines = pipelines(&device, &pipeline_layout, config.format);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("animation_viewer_atlas"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..wgpu::SamplerDescriptor::default()
    });
    let gpu = Gpu {
        surface,
        device,
        queue,
        config,
        pipelines,
        layout_ref: pipeline_layout,
        capture_pipelines: None,
        sampler,
        layout,
        atlas: None,
        vertices: None,
        vertex_capacity: 0,
        indices: None,
        index_capacity: 0,
    };

    Ok(WebSurface { canvas, overlay, gpu: RefCell::new(gpu), scale: RefCell::new(1.0) })
}

impl Gpu {
    fn upload_atlas(&mut self, atlas: &Atlas<'_>) {
        if self.atlas.as_ref().is_some_and(|bound| bound.key == atlas.key) {
            return;
        }

        let size = wgpu::Extent3d { width: atlas.width, height: atlas.height, depth_or_array_layers: 1 };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("animation_viewer_atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            atlas.pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * atlas.width), rows_per_image: Some(atlas.height) },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("animation_viewer_atlas"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });

        self.atlas = Some(Bound { key: atlas.key, bind_group });
    }

    fn upload(&mut self, vertex_data: &[f32], index_data: &[u32]) {
        let vertex_bytes: &[u8] = bytemuck::cast_slice(vertex_data);
        let index_bytes: &[u8] = bytemuck::cast_slice(index_data);

        if self.vertices.is_none() || self.vertex_capacity < vertex_bytes.len() as u64 {
            self.vertices = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("animation_viewer_vertices"),
                size: vertex_bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = vertex_bytes.len() as u64;
        }

        if self.indices.is_none() || self.index_capacity < index_bytes.len() as u64 {
            self.indices = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("animation_viewer_indices"),
                size: index_bytes.len() as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.index_capacity = index_bytes.len() as u64;
        }

        if let Some(buffer) = &self.vertices {
            self.queue.write_buffer(buffer, 0, vertex_bytes);
        }

        if let Some(buffer) = &self.indices {
            self.queue.write_buffer(buffer, 0, index_bytes);
        }
    }

    fn present(&mut self, scene: &Scene<'_>, scale: f32) {
        let width = self.config.width as f32;
        let height = self.config.height as f32;

        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(err) => {
                error!("The animation canvas could not be acquired: {}", err);
                return;
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("animation_viewer") });
        let clear = self.clear_color(scene, self.config.format);
        let drawn = self.prepare(scene, scale, width, height);

        self.pass(&mut encoder, &view, clear, drawn.as_deref(), false);
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    fn clear_color(&self, scene: &Scene<'_>, format: wgpu::TextureFormat) -> wgpu::Color {
        let channel = |value: f32| if format.is_srgb() { f64::from(linear(value)) } else { f64::from(value) };

        wgpu::Color { r: channel(scene.clear[0]), g: channel(scene.clear[1]), b: channel(scene.clear[2]), a: f64::from(scene.clear[3]) }
    }

    fn prepare(&mut self, scene: &Scene<'_>, scale: f32, width: f32, height: f32) -> Option<Vec<Batch>> {
        let atlas = scene.atlas.as_ref().filter(|_| !scene.parts.is_empty())?;

        self.upload_atlas(atlas);

        let zoom = scene.zoom * scale;
        let projection = [2.0 / width, 0.0, 0.0, 0.0, -2.0 / height, 0.0, -1.0, 1.0, 1.0];
        let camera = [zoom, 0.0, 0.0, 0.0, zoom, 0.0, width / 2.0 + scene.pan.0 * zoom, height / 2.0 + scene.pan.1 * zoom, 1.0];
        let view_proj = multiply_mat3(&projection, &camera);
        let (vertex_data, index_data, batches) = build_vertices(scene.parts, &view_proj);

        self.upload(&vertex_data, &index_data);

        Some(batches)
    }

    fn pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, clear: wgpu::Color, drawn: Option<&[Batch]>, offscreen: bool) {
        let pipelines = if offscreen { self.capture_pipelines.as_ref().unwrap_or(&self.pipelines) } else { &self.pipelines };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("animation_viewer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let (Some(batches), Some(atlas), Some(vertices), Some(indices)) = (drawn, &self.atlas, &self.vertices, &self.indices) {
            pass.set_bind_group(0, &atlas.bind_group, &[]);
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

            for batch in batches {
                pass.set_pipeline(&pipelines[batch.variant]);
                pass.draw_indexed(batch.range.clone(), 0, 0..1);
            }
        }
    }

    fn capture(&mut self, scene: &Scene<'_>, scale: f32) -> Result<Readback, Fault> {
        let (width, height) = (self.config.width.max(1), self.config.height.max(1));

        if self.capture_pipelines.is_none() {
            self.capture_pipelines = Some(pipelines(&self.device, &self.layout_ref, CAPTURE_FORMAT));
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("animation_capture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CAPTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let row = padded_row(width);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("animation_capture"),
            size: u64::from(row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("animation_capture") });
        let clear = self.clear_color(scene, CAPTURE_FORMAT);
        let drawn = self.prepare(scene, scale, width as f32, height as f32);

        self.pass(&mut encoder, &view, clear, drawn.as_deref(), true);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(height) } },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        let (sender, receiver) = oneshot::channel();

        buffer.slice(..).map_async(wgpu::MapMode::Read, move |outcome| {
            let _ = sender.send(outcome);
        });

        if let Err(err) = self.device.poll(wgpu::PollType::Poll) {
            return Err(Fault(format!("the capture could not be polled: {err:?}")));
        }

        Ok(Readback { buffer, receiver, width, height, row })
    }
}

struct Readback {
    buffer: wgpu::Buffer,
    receiver: oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
    width: u32,
    height: u32,
    row: u32,
}

impl Readback {
    async fn finish(self) -> Result<Captured, Fault> {
        self.receiver.await.map_err(|_| Fault("the capture was dropped".to_owned()))?.map_err(|err| Fault(format!("the capture could not be read: {err}")))?;

        let rgba = {
            let mapped = self.buffer.slice(..).get_mapped_range();
            let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);

            for y in 0..self.height {
                let start = (y * self.row) as usize;

                rgba.extend_from_slice(&mapped[start..start + (self.width * 4) as usize]);
            }

            rgba
        };

        self.buffer.unmap();

        Ok(Captured { width: self.width, height: self.height, rgba })
    }
}

impl WebSurface {
    fn overlay(&self, shapes: &[Shape], scale: f32) {
        let Some(context) = &self.overlay else {
            return;
        };
        let Some(overlay) = context.canvas() else {
            return;
        };
        let (width, height) = (f64::from(overlay.width()), f64::from(overlay.height()));

        context.clear_rect(0.0, 0.0, width, height);

        if shapes.is_empty() {
            return;
        }

        let scale = f64::from(scale);

        for shape in shapes {
            match *shape {
                Shape::Line { from, to, color, width } => {
                    context.set_stroke_style_str(&css_color(color));
                    context.set_line_width(f64::from(width) * scale);
                    context.begin_path();
                    context.move_to(f64::from(from.0) * scale, f64::from(from.1) * scale);
                    context.line_to(f64::from(to.0) * scale, f64::from(to.1) * scale);
                    context.stroke();
                }
                Shape::Circle { center, radius, color } => {
                    context.set_fill_style_str(&css_color(color));
                    context.begin_path();

                    if context.arc(f64::from(center.0) * scale, f64::from(center.1) * scale, f64::from(radius) * scale, 0.0, TAU).is_ok() {
                        context.fill();
                    }
                }
                Shape::Rect { min, max, fill, stroke } => {
                    let (x, y) = (f64::from(min.0) * scale, f64::from(min.1) * scale);
                    let (w, h) = (f64::from(max.0 - min.0) * scale, f64::from(max.1 - min.1) * scale);

                    if fill[3] > 0.0 {
                        context.set_fill_style_str(&css_color(fill));
                        context.fill_rect(x, y, w, h);
                    }

                    if let Some((color, width)) = stroke {
                        context.set_stroke_style_str(&css_color(color));
                        context.set_line_width(f64::from(width) * scale);
                        context.stroke_rect(x, y, w, h);
                    }
                }
            }
        }
    }
}

impl Surface for WebSurface {
    fn resize(&self, width: u32, height: u32, scale: f32) {
        let (width, height) = (width.max(1), height.max(1));
        let mut gpu = self.gpu.borrow_mut();

        *self.scale.borrow_mut() = scale;

        if self.canvas.width() != width || self.canvas.height() != height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
        }

        if let Some(overlay) = self.overlay.as_ref().and_then(CanvasRenderingContext2d::canvas)
            && (overlay.width() != width || overlay.height() != height)
        {
            overlay.set_width(width);
            overlay.set_height(height);
        }

        if gpu.config.width != width || gpu.config.height != height {
            gpu.config.width = width;
            gpu.config.height = height;
            gpu.surface.configure(&gpu.device, &gpu.config);
        }
    }

    fn capture(&self, scene: &Scene<'_>) -> Pending<Result<Captured, Fault>> {
        let scale = *self.scale.borrow();
        let started = self.gpu.borrow_mut().capture(scene, scale);

        Box::pin(async move { started?.finish().await })
    }

    fn draw(&self, scene: &Scene<'_>) {
        let scale = *self.scale.borrow();

        self.gpu.borrow_mut().present(scene, scale);
        self.overlay(scene.shapes, scale);
    }
}
