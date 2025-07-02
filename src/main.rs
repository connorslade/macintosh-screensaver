use std::time::Instant;

use anyhow::Result;
use bitvec::{order::Lsb0, vec::BitVec};
use encase::ShaderType;
use image::GenericImageView;
use tufa::{
    bindings::buffer::{UniformBuffer, mutability::Immutable},
    export::{
        egui::{Context, Window},
        nalgebra::{Matrix4, Vector2, Vector3},
        wgpu::{CompareFunction, RenderPass, ShaderStages, include_wgsl},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{
        GraphicsCtx, Interactive,
        ui::{dragger, vec3_dragger},
    },
    pipeline::render::RenderPipeline,
};

#[derive(ShaderType, Default)]
struct PixelUniform {
    view: Matrix4<f32>,
    image_size: Vector2<u32>,
    window_size: Vector2<u32>,
    cutoff: f32,
    progress: f32,
}

#[derive(ShaderType, Default)]
struct BackgroundUniform {
    start: Vector3<f32>,
    end: Vector3<f32>,
}

struct App {
    pixel_uniform: UniformBuffer<PixelUniform>,
    background: RenderPipeline,
    pixels: RenderPipeline,

    ctx: PixelUniform,

    camera_pos: Vector3<f32>,
    camera_target: Vector3<f32>,
    scale: f32,
    cutoff: f32,
    progress: f32,

    alpha: f32,
    beta: f32,

    start: Instant,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        let aspect = window.width as f32 / window.height as f32;

        // let t = self.start.elapsed().as_secs_f32() / 10.0;

        // self.progress = (-19.87) * (1.0 - t) + -6.0 * t;
        // self.camera_pos = Vector3::new(4.96, 3.5, -0.67).lerp(&Vector3::new(2.37, 3.92, -0.67), t);

        self.camera_target = Vector3::new(
            self.alpha.sin() * self.beta.cos(),
            self.alpha.sin() * self.beta.sin(),
            self.beta.cos(),
        );

        let depth = 100.0;
        let projection = if aspect < 1.0 {
            Matrix4::new_orthographic(-aspect, aspect, -1.0, 1.0, -depth, depth)
        } else {
            Matrix4::new_orthographic(-1.0, 1.0, -aspect.recip(), aspect.recip(), -depth, depth)
        };

        let scale = Matrix4::new_scaling(self.scale);
        let view = Matrix4::look_at_rh(
            &self.camera_pos.into(),
            &(self.camera_pos + self.camera_target).into(),
            &Vector3::z_axis(),
        );

        self.ctx.view = projection * view * scale;
        self.ctx.window_size = Vector2::new(window.width, window.height);
        self.ctx.cutoff = self.cutoff;
        self.ctx.progress = self.progress;

        self.pixel_uniform.upload(&self.ctx);
        self.background.draw_quad(render_pass, 0..1);
        self.pixels.draw_quad(render_pass, 0..1);
    }

    fn ui(&mut self, _gcx: GraphicsCtx, ctx: &Context) {
        Window::new("Macintosh Dynamic Wallpaper").show(ctx, |ui| {
            ui.horizontal(|ui| {
                vec3_dragger(ui, &mut self.camera_pos, |x| x.speed(0.01));
                ui.label("Camera Position");
            });

            ui.horizontal(|ui| {
                vec3_dragger(ui, &mut self.camera_target, |x| x.speed(0.01));
                ui.label("Camera Direction");
            });

            dragger(ui, "Scale", &mut self.scale, |x| x.speed(0.01));
            dragger(ui, "Cutoff", &mut self.cutoff, |x| x.speed(0.01));
            dragger(ui, "Progress", &mut self.progress, |x| x.speed(0.01));
            dragger(ui, "α", &mut self.alpha, |x| x.speed(0.01));
            dragger(ui, "β", &mut self.beta, |x| x.speed(0.01));
        });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let image = image::load_from_memory(include_bytes!("../image-2.png"))?;
    let mut buffer = BitVec::<u32, Lsb0>::new();

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y).0[0] != 0;
            buffer.push(pixel);
        }
    }

    let buffer = gpu.create_storage::<_, Immutable>(&buffer.into_vec());
    let pixel_uniform = gpu.create_uniform(&PixelUniform::default());
    let background_uniform = gpu.create_uniform(&BackgroundUniform {
        start: Vector3::new(0.376, 0.812, 0.663),
        end: Vector3::new(0.133, 0.749, 0.541),
    });
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
            background,
            pixels,

            ctx: PixelUniform {
                image_size: Vector2::new(image.width(), image.height()),
                ..PixelUniform::default()
            },

            camera_pos: Vector3::new(-4.31, -1.12, -0.67),
            camera_target: Vector3::new(-0.48, 1.0, 1.0),
            scale: 6.0,
            cutoff: 0.45,
            progress: 0.0,

            start: Instant::now(),

            alpha: 0.0,
            beta: 0.0,
        },
    )
    .run()?;

    Ok(())
}
