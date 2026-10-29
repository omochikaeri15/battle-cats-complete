use std::collections::BTreeMap;
use std::ops::Range;

use iced::wgpu;
use iced::widget::shader;
use tracing::trace;

use super::assets::Sheet;

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

const CLEARED: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

const BLENDS: [(wgpu::BlendFactor, wgpu::BlendFactor); 4] = [
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::One),
    (wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero),
    (wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrc),
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub struct Run {
    pub sheet: Option<Box<str>>,
    pub blend: u8,
    pub range: Range<u32>,
}

struct Binding {
    bind_group: wgpu::BindGroup,
    stamp: u64,
}

struct Reduced {
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

pub struct Pipeline {
    pipelines: [wgpu::RenderPipeline; 4],
    present: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    sheets: BTreeMap<Box<str>, Binding>,
    blank: Option<Binding>,
    void: Option<Binding>,
    vertices: Option<wgpu::Buffer>,
    capacity: u64,
    runs: Vec<Run>,
    reduced: Option<Reduced>,
    inset: Option<(f32, f32)>,
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        trace!(max_texture = device.limits().max_texture_dimension_2d, ?format, "emu: battle renderer created");

        let mut pipeline = Self::create(device, format);

        pipeline.blank = Some(pipeline.upload(device, queue, 1, 1, &[0xff, 0xff, 0xff, 0xff], 0));
        pipeline.void = Some(pipeline.upload(device, queue, 1, 1, &[0, 0, 0, 0xff], 0));
        pipeline
    }
}

impl Pipeline {
    fn create(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("emu_battle"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("emu_battle_sheet"),
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
            label: Some("emu_battle"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipelines = BLENDS.map(|(source, target)| {
            let component = wgpu::BlendComponent {
                src_factor: source,
                dst_factor: target,
                operation: wgpu::BlendOperation::Add,
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("emu_battle"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: component,
                            alpha: component,
                        }),
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        });

        let present = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("emu_battle_present"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_present"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_present"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("emu_battle"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        Self {
            pipelines,
            present,
            format,
            layout,
            sampler,
            sheets: BTreeMap::new(),
            blank: None,
            void: None,
            vertices: None,
            capacity: 0,
            runs: Vec::new(),
            reduced: None,
            inset: None,
        }
    }

    fn upload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
        stamp: u64,
    ) -> Binding {
        let size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("emu_battle_sheet"),
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
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * 4),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("emu_battle_sheet"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        Binding { bind_group, stamp }
    }

    pub fn reduce(&mut self, device: &wgpu::Device, scale: f32, padded: bool, full: iced::Size<u32>) {
        if scale >= 1.0 {
            self.reduced = None;
            self.inset = None;

            return;
        }

        let width = ((full.width as f32 * scale).round() as u32).max(1);
        let height = ((full.height as f32 * scale).round() as u32).max(1);

        self.inset = padded.then(|| (full.width.saturating_sub(width) / 2, full.height.saturating_sub(height) / 2)).map(|(left, top)| (left as f32, top as f32));

        if self.reduced.as_ref().is_some_and(|held| held.width == width && held.height == height) {
            return;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("emu_battle_reduced"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("emu_battle_reduced"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        trace!(width, height, scale, "emu: battle drawn at a reduced resolution");

        self.reduced = Some(Reduced { view, bind_group, width, height });
    }

    pub fn ensure_sheet(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str, sheet: &Sheet) -> bool {
        if self.sheets.get(name).is_some_and(|held| held.stamp == sheet.stamp) {
            return false;
        }

        let binding = self.upload(device, queue, sheet.width, sheet.height, &sheet.pixels, sheet.stamp);

        self.sheets.insert(Box::from(name), binding);

        true
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
        runs: Vec<Run>,
    ) {
        self.runs = runs;

        if vertices.is_empty() {
            return;
        }

        let needed = std::mem::size_of_val(vertices) as u64;

        if self.capacity < needed || self.vertices.is_none() {
            self.vertices = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("emu_battle_vertices"),
                size: needed,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.capacity = needed;
        }

        if let Some(buffer) = &self.vertices {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(vertices));
        }
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip: &iced::Rectangle<u32>,
    ) {
        let Some(buffer) = &self.vertices else {
            return;
        };

        if self.runs.is_empty() || clip.width == 0 || clip.height == 0 {
            return;
        }

        let Some(reduced) = &self.reduced else {
            let mut pass = Self::pass(encoder, target, wgpu::LoadOp::Load);

            pass.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
            self.paint(&mut pass, buffer);

            return;
        };

        {
            let mut pass = Self::pass(encoder, &reduced.view, wgpu::LoadOp::Clear(CLEARED));

            self.paint(&mut pass, buffer);
        }

        let mut pass = Self::pass(encoder, target, wgpu::LoadOp::Load);

        pass.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
        pass.set_pipeline(&self.present);

        if let Some((left, top)) = self.inset
            && let Some(void) = &self.void
        {
            pass.set_bind_group(0, &void.bind_group, &[]);
            pass.draw(0..3, 0..1);
            pass.set_viewport(left, top, reduced.width as f32, reduced.height as f32, 0.0, 1.0);
        }

        pass.set_bind_group(0, &reduced.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    fn pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView, load: wgpu::LoadOp<wgpu::Color>) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("emu_battle"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    fn paint(&self, pass: &mut wgpu::RenderPass<'_>, buffer: &wgpu::Buffer) {
        pass.set_vertex_buffer(0, buffer.slice(..));

        for run in &self.runs {
            let binding = match &run.sheet {
                Some(name) => self.sheets.get(name.as_ref()).or(self.blank.as_ref()),
                None => self.blank.as_ref(),
            };

            let Some(binding) = binding else {
                continue;
            };

            let Some(pipeline) = self.pipelines.get(run.blend as usize).or(self.pipelines.first()) else {
                continue;
            };

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &binding.bind_group, &[]);
            pass.draw(run.range.clone(), 0..1);
        }
    }
}
