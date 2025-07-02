use anyhow::Result;
use bitvec::{order::Lsb0, vec::BitVec};
use encase::ShaderType;
use image::GenericImageView;
use tufa::{
    bindings::buffer::{UniformBuffer, mutability::Immutable},
    export::{
        egui::{Context, Window},
        nalgebra::{Matrix4, Vector2, Vector3},
        wgpu::{RenderPass, ShaderStages, include_wgsl},
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
struct Uniform {
    view: Matrix4<f32>,
    image_size: Vector2<u32>,
    window_size: Vector2<u32>,
    cutoff: f32,
    progress: f32,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,

    ctx: Uniform,

    camera_pos: Vector3<f32>,
    camera_target: Vector3<f32>,
    scale: f32,
    cutoff: f32,
    progress: f32,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        let aspect = window.width as f32 / window.height as f32;

        let projection = if aspect < 1.0 {
            Matrix4::new_orthographic(-aspect, aspect, -1.0, 1.0, -100.0, 100.0)
        } else {
            Matrix4::new_orthographic(-1.0, 1.0, -aspect.recip(), aspect.recip(), -2.0, 2.0)
        };

        let scale = Matrix4::new_scaling(self.scale);
        let view = Matrix4::look_at_rh(
            &self.camera_pos.into(),
            &(self.camera_pos + self.camera_target.try_normalize(0.0).unwrap_or_default()).into(),
            &Vector3::z_axis(),
        );

        self.ctx.view = projection * view * scale;
        self.ctx.window_size = Vector2::new(window.width, window.height);
        self.ctx.cutoff = self.cutoff;
        self.ctx.progress = self.progress;

        self.uniform.upload(&self.ctx);
        self.render.draw_quad(render_pass, 0..1);
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
        });
    }
}

fn main() -> Result<()> {
    let gpu = Gpu::new()?;

    let image = image::load_from_memory(include_bytes!("../image.png"))?;
    let mut buffer = BitVec::<u32, Lsb0>::new();

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y).0[0] != 0;
            buffer.push(pixel);
        }
    }

    let buffer = gpu.create_storage::<_, Immutable>(&buffer.into_vec());
    let uniform = gpu.create_uniform(&Uniform::default());
    let render = gpu
        .render_pipeline(include_wgsl!("render.wgsl"))
        .bind(&uniform, ShaderStages::VERTEX_FRAGMENT)
        .bind(&buffer, ShaderStages::FRAGMENT)
        .finish();

    gpu.create_window(
        WindowAttributes::default().with_title("Macintosh Dynamic Wallpaper"),
        App {
            uniform,
            render,

            ctx: Uniform {
                image_size: Vector2::new(image.width(), image.height()),
                ..Uniform::default()
            },

            camera_pos: Vector3::new(-4.31, -1.12, -0.67),
            camera_target: Vector3::new(-0.48, 1.0, 1.0),
            scale: 6.0,
            cutoff: 0.38,
            progress: 0.0,
        },
    )
    .run()?;

    Ok(())
}
