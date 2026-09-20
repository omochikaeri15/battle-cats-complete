use std::collections::BTreeMap;
use std::ops::Range;

use iced::wgpu;
use iced::widget::shader;

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
}

pub struct Pipeline {
    pipelines: [wgpu::RenderPipeline; 4],
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    sheets: BTreeMap<Box<str>, Binding>,
    blank: Option<Binding>,
    vertices: Option<wgpu::Buffer>,
    capacity: u64,
    runs: Vec<Run>,
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let mut pipeline = Self::create(device, format);

        pipeline.blank = Some(pipeline.upload(device, queue, 1, 1, &[0xff, 0xff, 0xff, 0xff]));
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("emu_battle"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        Self {
            pipelines,
            layout,
            sampler,
            sheets: BTreeMap::new(),
            blank: None,
            vertices: None,
            capacity: 0,
            runs: Vec::new(),
        }
    }

    fn upload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        pixels: &[u8],
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

        Binding { bind_group }
    }

    pub fn ensure_sheet(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        if self.sheets.contains_key(name) {
            return;
        }

        let binding = self.upload(device, queue, width, height, pixels);

        self.sheets.insert(Box::from(name), binding);
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

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("emu_battle"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
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
