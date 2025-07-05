// ↓ Needed due to a rust-analyzer bug
#![allow(dead_code)]

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
    prelude::StorageBuffer,
};
#[cfg(feature = "manual")]
use tufa::{
    export::egui::{Context, Slider, Window},
    interactive::ui::{dragger, vec3_dragger},
};

use crate::animation::Animation;
#[cfg(feature = "manual")]
use crate::animation::properties::Properties;

mod animation;
mod interpolate;

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
    image: StorageBuffer<Vec<u32>, Immutable>,

    background_uniform: UniformBuffer<BackgroundUniform>,
    background: RenderPipeline,

    start: Instant,
    animation: Animation,

    #[cfg(feature = "manual")]
    manual: (Properties, usize),
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        let aspect = window.width as f32 / window.height as f32;

        let time = self.start.elapsed().as_secs_f32();
        let t = (time / 60.0) % 1.0;

        let colormap = &self.animation.colormap;
        let foreground = colormap.get_foreground(t);
        self.background_uniform.upload(&BackgroundUniform {
            start: colormap.get_background_top(t),
            end: colormap.get_background_bottom(t),
        });

        #[cfg(not(feature = "manual"))]
        let (properties, image) = self.animation.scene(time);

        #[cfg(feature = "manual")]
        let (properties, image) = (
            &self.manual.0,
            self.animation.image(self.manual.1, self.manual.0.frame),
        );

        self.image.upload(&image.data);
        self.pixel_uniform.upload(&PixelUniform {
            view: properties.view_projection(aspect),
            image_size: image.size,
            window_size: Vector2::new(window.width, window.height),
            color: foreground,
            cutoff: 0.43,
            progress: properties.progress,
            progress_angle: properties.progress_angle,
        });

        self.background.draw_quad(render_pass, 0..1);
        self.pixels.draw_quad(render_pass, 0..1);
    }

    #[cfg(feature = "manual")]
    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        Window::new("Macintosh Dynamic Wallpaper").show(ctx, |ui| {
            let props = &mut self.manual.0;

            let max_scene = self.animation.scenes() - 1;
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut self.manual.1, 0..=max_scene));
                ui.label("Scene");
            });

            let max_frame = self.animation.frames(self.manual.1) - 1;
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut props.frame, 0..=max_frame));
                ui.label("Frame");
            });

            ui.horizontal(|ui| {
                vec3_dragger(ui, &mut props.camera_pos, |x| x.speed(0.01));
                ui.label("Camera Position");
            });

            ui.horizontal(|ui| {
                vec3_dragger(ui, &mut props.camera_dir, |x| x.speed(0.01));
                ui.label("Camera Direction");
            });

            dragger(ui, "Scale", &mut props.scale, |x| x.speed(0.01));
            dragger(ui, "Progress Anlge", &mut props.progress_angle, |x| {
                x.speed(0.01)
            });
            dragger(ui, "Progress", &mut props.progress, |x| x.speed(0.01));
        });
    }
}

fn main() -> Result<()> {
    let animation =
        Animation::load("/home/connorslade/Programming/macintosh_wallpaper/animation/config.toml")?;

    let gpu = Gpu::new()?;

    let image = gpu.create_storage_empty::<Vec<u32>, Immutable>((512_u64 * 342).div_ceil(32));
    let pixel_uniform = gpu.create_uniform(&PixelUniform::default());
    let background_uniform = gpu.create_uniform(&BackgroundUniform::default());
    let pixels = gpu
        .render_pipeline(include_wgsl!("../shaders/pixels.wgsl"))
        .depth_compare(CompareFunction::Always)
        .bind(&pixel_uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&image, ShaderStages::FRAGMENT)
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
            image,

            background_uniform,
            background,

            #[cfg(feature = "manual")]
            manual: (animation.config.scenes.properties.clone(), 0),

            start: Instant::now(),
            animation,
        },
    )
    .run()?;

    Ok(())
}
