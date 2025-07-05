use std::sync::Arc;

use anyhow::{Context, Result};
use nalgebra::Vector2;
use wgpu::{
    Color, CommandEncoderDescriptor, Instance, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use macintosh_wallpaper::{
    animation::Animation,
    pipelines::{Gpu, Renderer},
};

struct Application {
    gpu: Gpu,
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,

    renderer: Renderer,
}

fn main() -> Result<()> {
    let instance = Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
        .context("No adapter found")?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))?;

    let mut app = Application {
        gpu: Gpu {
            instance,
            adapter,
            device,
            queue,

            texture_format: TextureFormat::Bgra8UnormSrgb,
        },
        state: None,
    };

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;

    Ok(())
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let animation = Animation::load(include_bytes!("../animation/animation.bin")).unwrap();

        let attrs = WindowAttributes::default().with_title("Macintosh Wallpaper");
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let surface = self.gpu.instance.create_surface(window.clone()).unwrap();
        let renderer = Renderer::new(&self.gpu, animation);
        self.state = Some(State {
            surface,
            window,
            renderer,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        if window_id != state.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_size) => self.resize_surface(),
            WindowEvent::RedrawRequested => {
                let output = state.surface.get_current_texture().unwrap();

                let mut encoder = self
                    .gpu
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor::default());

                let view = output
                    .texture
                    .create_view(&TextureViewDescriptor::default());

                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLACK),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    let size = state.window.inner_size();
                    let size = Vector2::new(size.width, size.height);
                    state.renderer.render(&self.gpu, size, &mut render_pass);
                }

                self.gpu.queue.submit([encoder.finish()]);

                output.present();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl Application {
    fn resize_surface(&mut self) {
        let state = self.state.as_mut().unwrap();
        let size = state.window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.gpu.texture_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        state.surface.configure(&self.gpu.device, &config);
    }
}
