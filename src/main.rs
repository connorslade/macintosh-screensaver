use std::time::Instant;

use anyhow::Result;
use encase::ShaderType;
use tufa::{
    bindings::buffer::{UniformBuffer, mutability::Immutable},
    export::{
        nalgebra::{Matrix4, Vector2, Vector3},
        wgpu::{CompareFunction, RenderPass, ShaderStages, include_wgsl},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};

use crate::animation::Animation;

mod animation;
mod colormap;

#[derive(ShaderType, Default)]
struct PixelUniform {
    view: Matrix4<f32>,
    image_size: Vector2<u32>,
    window_size: Vector2<u32>,
    color: Vector3<f32>,
    cutoff: f32,
    progress: f32,
    progress_angle: f32,
}

#[derive(ShaderType, Default)]
struct BackgroundUniform {
    start: Vector3<f32>,
    end: Vector3<f32>,
}

struct App {
    pixel_uniform: UniformBuffer<PixelUniform>,
    pixels: RenderPipeline,

    background_uniform: UniformBuffer<BackgroundUniform>,
    background: RenderPipeline,

    start: Instant,
    animation: Animation,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        let aspect = window.width as f32 / window.height as f32;

        let time = self.start.elapsed().as_secs_f32();
        let t = (time / 60.0) % 1.0;
        let properties = self.animation.properties(time);

        let colormap = &self.animation.colormap;
        self.background_uniform.upload(&BackgroundUniform {
            start: colormap.get_background_top(t),
            end: colormap.get_background_bottom(t),
        });

        self.pixel_uniform.upload(&PixelUniform {
            view: properties.view_projection(aspect),
            image_size: Vector2::zeros(),
            window_size: Vector2::new(window.width, window.height),
            color: colormap.get_foreground(t),
            cutoff: 0.43,
            progress: properties.progress,
            progress_angle: properties.progress_angle,
        });

        self.background.draw_quad(render_pass, 0..1);
        self.pixels.draw_quad(render_pass, 0..1);
    }
}

fn main() -> Result<()> {
    let animation = Animation::load("animation/config.toml")?;

    let gpu = Gpu::new()?;

    let buffer = gpu.create_storage_empty::<Vec<u32>, Immutable>((512_u64 * 342).div_ceil(8));
    let pixel_uniform = gpu.create_uniform(&PixelUniform::default());
    let background_uniform = gpu.create_uniform(&BackgroundUniform::default());
    let pixels = gpu
        .render_pipeline(include_wgsl!("../shaders/pixels.wgsl"))
        .depth_compare(CompareFunction::Always)
        .bind(&pixel_uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&buffer, ShaderStages::FRAGMENT)
        .finish();
    let background = gpu
        .render_pipeline(include_wgsl!("../shaders/background.wgsl"))
        .bind(&background_uniform, ShaderStages::FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Macintosh Dynamic Wallpaper"),
        App {
            pixel_uniform,
            pixels,

            background_uniform,
            background,

            start: Instant::now(),
            animation,
        },
    )
    .run()?;

    Ok(())
}
