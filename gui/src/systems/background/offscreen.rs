use std::sync::mpsc;
use std::sync::Arc;

use iced::futures::executor::block_on;
use iced::wgpu;
use iced::Rectangle;
use tracing::error;

use nyanko::graphics::backdrop::{Backdrop, Camera};

use super::pipeline::{self, Pipeline};

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
    target: wgpu::Texture,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded: u32,
    scale: f32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|failure| format!("No GPU adapter is available for background rendering: {failure}"))
            .inspect_err(|message| error!("{message}"))?;
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|failure| format!("The GPU device for background rendering could not be created: {failure}"))
            .inspect_err(|message| error!("{message}"))?;
        let max_dim = device.limits().max_texture_dimension_2d as f32;
        let (asked_w, asked_h) = (width.max(1) as f32, height.max(1) as f32);
        let scale = (max_dim / asked_w).min(max_dim / asked_h).min(1.0);
        let width = ((asked_w * scale).round() as u32).max(1);
        let height = ((asked_h * scale).round() as u32).max(1);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let pipeline = Pipeline::create(&device, &queue, format);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("background_export_target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let padded = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("background_export_readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self { device, queue, pipeline, target, readback, width, height, padded, scale })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn render(&mut self, backdrop: &mut Backdrop, camera: Camera) -> Result<Vec<u8>, String> {
        let camera = Camera { width: self.width as f32, height: self.height as f32, ..camera.rescaled(camera.zoom * self.scale) };
        let quads = backdrop.render(camera);
        let scene = pipeline::build(
            &quads,
            |surface| backdrop.image(surface).map(Arc::clone),
            |corner| [corner[0] / camera.width * 2.0 - 1.0, 1.0 - corner[1] / camera.height * 2.0],
        );

        self.pipeline.write(&self.device, &self.queue, &scene);

        let view = self.target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("background_export") });
        let clip = Rectangle { x: 0, y: 0, width: self.width, height: self.height };

        self.pipeline.draw(&mut encoder, &view, &clip, Some(wgpu::Color::TRANSPARENT));

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &self.target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(self.padded), rows_per_image: Some(self.height) },
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);

        let slice = self.readback.slice(..);
        let (sender, receiver) = mpsc::channel();

        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|failure| format!("The GPU stalled while reading a background frame back: {failure}"))?;

        match receiver.recv() {
            Ok(Ok(())) => {}
            Ok(Err(failure)) => return Err(format!("A background frame could not be read back: {failure}")),
            Err(_) => return Err("A background frame was never read back".to_owned()),
        }

        let row = (self.width * 4) as usize;
        let padded = self.padded as usize;
        let mut pixels = Vec::with_capacity(row * self.height as usize);

        {
            let mapped = slice.get_mapped_range();

            for line in (0..self.height as usize).rev() {
                let start = line * padded;

                pixels.extend_from_slice(&mapped[start..start + row]);
            }
        }

        self.readback.unmap();

        Ok(pixels)
    }
}








