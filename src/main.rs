use anyhow::Result;
use bitvec::{
    order::{Lsb0, Msb0},
    vec::BitVec,
};
use encase::ShaderType;
use image::GenericImageView;
use tufa::{
    bindings::{UniformBuffer, mutability::Immutable},
    export::{
        egui::Context,
        nalgebra::Vector2,
        wgpu::{RenderPass, ShaderStages, include_wgsl},
        winit::window::WindowAttributes,
    },
    gpu::Gpu,
    interactive::{GraphicsCtx, Interactive},
    pipeline::render::RenderPipeline,
};

#[derive(ShaderType, Default)]
struct Uniform {
    pan: Vector2<f32>,

    image_size: Vector2<u32>,
    window_size: Vector2<u32>,
}

struct App {
    uniform: UniformBuffer<Uniform>,
    render: RenderPipeline,

    ctx: Uniform,
}

impl Interactive for App {
    fn render(&mut self, gcx: GraphicsCtx, render_pass: &mut RenderPass) {
        let window = gcx.window.inner_size();
        self.ctx.window_size = Vector2::new(window.width, window.height);

        self.uniform.upload(&self.ctx);
        self.render.draw_quad(render_pass, 0..1);
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
        .bind(&uniform, ShaderStages::FRAGMENT)
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
        },
    )
    .run()?;

    Ok(())
}
