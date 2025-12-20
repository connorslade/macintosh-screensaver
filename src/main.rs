#![windows_subsystem = "windows"]

use std::sync::Arc;

use anyhow::{Context, Result};
use nalgebra::Vector2;
use wgpu::{
    Color, CommandEncoderDescriptor, CompositeAlphaMode, Instance, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
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

    cursor_start: Option<PhysicalPosition<f64>>,
    preview: bool,
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
        let config = include_bytes!("../animation/animation.bin");
        let animation = Animation::load(config).unwrap().runtime_from_args();
        let rt = &animation.runtime;
        let preview = rt.preview.is_some();

        #[cfg(windows)]
        if rt.configure {
            message_box("This screensaver has no configuration options.");
            event_loop.exit();
            return;
        }

        #[allow(unused_mut)]
        let mut attrs = WindowAttributes::default()
            .with_fullscreen(rt.full_screen.then_some(Fullscreen::Borderless(None)))
            .with_title("Macintosh Wallpaper")
            .with_visible(false);

        #[cfg(windows)]
        if let Some(hwnd) = rt.preview {
            use ::{wgpu::rwh::Win32WindowHandle, winit::dpi::PhysicalSize};
            let parent = Win32WindowHandle::new(NonZeroIsize::new(hwnd).unwrap());
            unsafe {
                attrs = attrs
                    .with_inner_size(PhysicalSize::new(152, 112))
                    .with_parent_window(Some(parent.into()))
                    .with_decorations(false)
            }
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        window.set_cursor_visible(!rt.full_screen);
        window.set_visible(true);

        let surface = self.gpu.instance.create_surface(window.clone()).unwrap();
        let renderer = Renderer::new(&self.gpu, animation);
        self.state = Some(State {
            surface,
            window,
            renderer,

            cursor_start: None,
            preview,
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

        let full_screen = state.window.fullscreen().is_some();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_size) => self.resize_surface(),
            WindowEvent::CursorMoved { position, .. } if !state.preview && full_screen => {
                match state.cursor_start {
                    Some(start) if start != position => {
                        state.window.set_visible(false);
                        event_loop.exit()
                    }
                    None => state.cursor_start = Some(position),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let output = state.surface.get_current_texture().unwrap();

                let mut encoder =
                    (self.gpu.device).create_command_encoder(&CommandEncoderDescriptor::default());
                let view = (output.texture).create_view(&TextureViewDescriptor::default());

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
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        state.surface.configure(&self.gpu.device, &config);
    }
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "C" {
    fn ShellMessageBoxA(
        app: isize,
        hwnd: isize,
        caption: *const i8,
        title: *const i8,
        u_type: u32,
    ) -> i32;
}

#[cfg(windows)]
fn message_box(message: impl Into<Vec<u8>>) {
    use std::{ffi::CString, num::NonZeroIsize};

    unsafe {
        let title = CString::new("Macintosh Screensaver").unwrap();
        let caption = CString::new(message.into()).unwrap();
        ShellMessageBoxA(0, 0, caption.as_ptr(), title.as_ptr(), 0);
    }
}
