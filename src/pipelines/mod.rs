use std::time::Instant;

use nalgebra::Vector2;
use wgpu::{
    Adapter, Buffer, BufferUsages, Device, IndexFormat, Instance, Queue, RenderPass, TextureFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    animation::Animation,
    pipelines::{
        background::{BackgroundPipeline, BackgroundUniform},
        pixel::{PixelsPipeline, PixelsUniform},
    },
};

pub mod background;
pub mod pixel;

pub struct Renderer {
    background: BackgroundPipeline,
    pixels: PixelsPipeline,
    index: Buffer,

    start: Instant,
    animation: Animation,
}

pub struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

    pub texture_format: TextureFormat,
}

impl Renderer {
    pub fn new(gpu: &Gpu, animation: Animation) -> Self {
        let index: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let index = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&index),
            usage: BufferUsages::INDEX,
        });

        Self {
            background: BackgroundPipeline::new(gpu),
            pixels: PixelsPipeline::new(gpu),
            index,

            start: Instant::now(),
            animation,
        }
    }

    pub fn render(&mut self, gpu: &Gpu, size: Vector2<u32>, render_pass: &mut RenderPass) {
        let aspect = size.x as f32 / size.y as f32;

        let time = self.start.elapsed().as_secs_f32();
        let t = (time / 60.0) % 1.0;

        let colormap = &self.animation.colormap;
        let foreground = colormap.get_foreground(t);

        let background_uniform = BackgroundUniform {
            start: colormap.get_background_top(t),
            end: colormap.get_background_bottom(t),
        };

        let (properties, image) = self.animation.scene(time);
        let pixels_uniform = PixelsUniform {
            view: properties.view_projection(aspect),
            image_size: image.size,
            window_size: size,
            color: foreground,
            scale: properties.scale,
            progress: properties.progress,
            progress_angle: properties.progress_angle,
        };

        self.background.prepare(gpu, &background_uniform);
        self.pixels.prepare(gpu, &pixels_uniform, &image.data);

        render_pass.set_index_buffer(self.index.slice(..), IndexFormat::Uint16);
        self.background.paint(render_pass);
        self.pixels.paint(render_pass);
    }
}
