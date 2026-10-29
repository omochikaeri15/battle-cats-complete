use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use iced::wgpu;
use iced::widget::shader;
use image::RgbaImage;

use codex::graphics::backdrop::{Quad, Surface};

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
";

const BLENDS: [(wgpu::BlendFactor, wgpu::BlendFactor); 4] = [
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::One),
    (wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrc),
];

const VERTEX_STRIDE: u64 = 32;
const BLANK_PIXEL: [u8; 4] = [0xff; 4];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub struct Run {
    pub texture: Option<usize>,
    pub blend: u8,
    pub range: Range<u32>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Key {
    pub stamp: u64,
    pub bounds: iced::Rectangle,
    pub factor: f32,
    pub window: iced::Size,
}

pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub runs: Vec<Run>,
    pub textures: Vec<Arc<RgbaImage>>,
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("vertices", &self.vertices.len())
            .field("runs", &self.runs.len())
            .field("textures", &self.textures.len())
            .finish()
    }
}

impl Scene {
    pub fn empty() -> Self {
        Self { vertices: Vec::new(), runs: Vec::new(), textures: Vec::new() }
    }
}

pub fn build(
    quads: &[Quad],
    image_of: impl Fn(Surface) -> Option<Arc<RgbaImage>>,
    to_clip: impl Fn([f32; 2]) -> [f32; 2],
) -> Scene {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(quads.len() * 6);
    let mut runs: Vec<Run> = Vec::new();
    let mut textures: Vec<Arc<RgbaImage>> = Vec::new();

    for quad in quads {
        let texture = quad.surface.and_then(&image_of).map(|image| {
            match textures.iter().position(|held| Arc::ptr_eq(held, &image)) {
                Some(index) => index,
                None => {
                    textures.push(image);
                    textures.len() - 1
                }
            }
        });

        if quad.surface.is_some() && texture.is_none() {
            continue;
        }

        let uv = texture.map_or([[0.0f32; 2]; 4], |index| {
            let image = &textures[index];
            let width = image.width().max(1) as f32;
            let height = image.height().max(1) as f32;
            let left = quad.source[0] / width;
            let top = quad.source[1] / height;
            let right = (quad.source[0] + quad.source[2]) / width;
            let bottom = (quad.source[1] + quad.source[3]) / height;

            [[left, top], [left, bottom], [right, bottom], [right, top]]
        });
        let corner = |slot: usize| Vertex { position: to_clip(quad.corners[slot]), uv: uv[slot], color: quad.colors[slot] };
        let start = vertices.len() as u32;

        vertices.extend([corner(0), corner(1), corner(2), corner(0), corner(2), corner(3)]);

        let end = vertices.len() as u32;

        match runs.last_mut() {
            Some(run) if run.texture == texture && run.blend == quad.blend => run.range.end = end,
            _ => runs.push(Run { texture, blend: quad.blend, range: start..end }),
        }
    }

    Scene { vertices, runs, textures }
}

struct Binding {
    bind_group: wgpu::BindGroup,
    image: Arc<RgbaImage>,
}

pub struct Pipeline {
    pipelines: [wgpu::RenderPipeline; 4],
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    bindings: HashMap<usize, Binding>,
    blank: Option<wgpu::BindGroup>,
    vertices: Option<wgpu::Buffer>,
    capacity: u64,
    runs: Vec<Run>,
    keys: Vec<usize>,
    held: Option<Key>,
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self::create(device, queue, format)
    }
}

impl Pipeline {
    pub fn create(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("background_viewer"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("background_viewer_sheet"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("background_viewer"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipelines = BLENDS.map(|(source, destination)| {
            let blend = wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: source, dst_factor: destination, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: source, dst_factor: destination, operation: wgpu::BlendOperation::Add },
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("background_viewer"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: VERTEX_STRIDE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    ..wgpu::PrimitiveState::default()
                },
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
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("background_viewer_sheet"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        let mut pipeline = Self {
            pipelines,
            layout,
            sampler,
            bindings: HashMap::new(),
            blank: None,
            vertices: None,
            capacity: 0,
            runs: Vec::new(),
            keys: Vec::new(),
            held: None,
        };

        pipeline.blank = Some(pipeline.upload(device, queue, 1, 1, &BLANK_PIXEL));
        pipeline
    }

    fn upload(&self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, pixels: &[u8]) -> wgpu::BindGroup {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("background_viewer_sheet"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: Some(height) },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("background_viewer_sheet"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn key_of(image: &Arc<RgbaImage>) -> usize {
        Arc::as_ptr(image) as usize
    }

    pub fn holds(&self, key: &Key) -> bool {
        self.held.as_ref() == Some(key)
    }

    pub fn hold(&mut self, key: Key) {
        self.held = Some(key);
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scene: &Scene) {
        self.held = None;
        self.keys.clear();

        for image in &scene.textures {
            let key = Self::key_of(image);

            if !self.bindings.get(&key).is_some_and(|binding| Arc::ptr_eq(&binding.image, image)) {
                let bind_group = self.upload(device, queue, image.width(), image.height(), image.as_raw());

                self.bindings.insert(key, Binding { bind_group, image: Arc::clone(image) });
            }

            self.keys.push(key);
        }

        let kept: Vec<usize> = self.keys.clone();

        self.bindings.retain(|key, _| kept.contains(key));

        let bytes: &[u8] = bytemuck::cast_slice(&scene.vertices);
        let needed = bytes.len() as u64;

        if needed > self.capacity || self.vertices.is_none() {
            let capacity = needed.max(VERTEX_STRIDE * 6).next_power_of_two();

            self.vertices = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("background_viewer_vertices"),
                size: capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.capacity = capacity;
        }

        if let Some(buffer) = &self.vertices
            && !bytes.is_empty()
        {
            queue.write_buffer(buffer, 0, bytes);
        }

        self.runs = scene.runs.iter().map(|run| Run { texture: run.texture, blend: run.blend, range: run.range.clone() }).collect();
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, clip: &iced::Rectangle<u32>, clear: Option<wgpu::Color>) {
        let Some(vertices) = &self.vertices else {
            return;
        };

        if clip.width == 0 || clip.height == 0 {
            return;
        }

        let load = clear.map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("background_viewer"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
        pass.set_vertex_buffer(0, vertices.slice(..));

        for run in &self.runs {
            let bind_group = match run.texture.and_then(|index| self.keys.get(index)).and_then(|key| self.bindings.get(key)) {
                Some(binding) => &binding.bind_group,
                None => match &self.blank {
                    Some(blank) if run.texture.is_none() => blank,
                    _ => continue,
                },
            };

            pass.set_pipeline(&self.pipelines[usize::from(run.blend).min(BLENDS.len() - 1)]);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(run.range.clone(), 0..1);
        }
    }
}
