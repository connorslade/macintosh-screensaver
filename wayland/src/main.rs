// Reference: https://github.com/Smithay/client-toolkit/blob/master/examples/wgpu.rs

// ↓ Needed due to a rust-analyzer bug
#![allow(dead_code)]

use std::{mem, ptr::NonNull};

use anyhow::Result;
use nalgebra::Vector2;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{Connection, Proxy, QueueHandle, globals::registry_queue_init},
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
};
use wgpu::{
    Color, CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, Operations,
    PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTargetUnsafe, TextureFormat, TextureUsages,
    rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle},
};

use macintosh_wallpaper::{
    animation::Animation,
    pipelines::{Gpu, Renderer},
};

mod impls;

struct App {
    output_state: OutputState,
    seat_state: SeatState,
    registry_state: RegistryState,

    renderer: Renderer,
    surface: Surface<'static>,
    gpu: Gpu,
    exit: bool,

    window: Window,
    needs_config: bool,
    size: Vector2<u32>,
}

fn main() -> Result<()> {
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh)?;
    let xdg_shell_state = XdgShell::bind(&globals, &qh)?;

    let surface = compositor_state.create_surface(&qh);
    let window = xdg_shell_state.create_window(surface, WindowDecorations::ServerDefault, &qh);
    window.set_title("Macintosh Wallpaper");
    window.set_app_id("com.connorcode.macintosh-wallpaper");
    window.commit();

    let instance = Instance::new(&InstanceDescriptor::default());

    let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
        NonNull::new(conn.backend().display_ptr() as _).unwrap(),
    ));
    let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
        NonNull::new(window.wl_surface().id().as_ptr() as _).unwrap(),
    ));

    let surface = unsafe {
        instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
            raw_display_handle,
            raw_window_handle,
        })?
    };

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .unwrap();

    let (device, queue) =
        pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None))?;

    let animation =
        Animation::load("/home/connorslade/Programming/macintosh_wallpaper/animation/config.toml")
            .unwrap();

    let gpu = Gpu {
        instance,
        adapter,
        device,
        queue,

        texture_format: TextureFormat::Bgra8UnormSrgb,
    };
    let renderer = Renderer::new(&gpu, animation);

    let mut app = App {
        output_state: OutputState::new(&globals, &qh),
        seat_state: SeatState::new(&globals, &qh),
        registry_state: RegistryState::new(&globals),

        renderer,
        surface,
        gpu,
        exit: false,

        window,
        needs_config: true,
        size: Vector2::zeros(),
    };

    while !app.exit {
        event_queue.blocking_dispatch(&mut app)?;
    }

    drop(app.surface);
    Ok(())
}

impl WindowHandler for App {
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.size = Vector2::new(configure.new_size.0, configure.new_size.1)
            .map(|x| x.map(|x| x.get()).unwrap_or(1));

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.gpu.texture_format,
            view_formats: vec![],
            alpha_mode: CompositeAlphaMode::Auto,
            width: self.size.x,
            height: self.size.y,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::Mailbox,
        };

        self.surface.configure(&self.gpu.device, &surface_config);

        if mem::take(&mut self.needs_config) {
            self.window
                .wl_surface()
                .frame(qh, self.window.wl_surface().clone());
        }

        self.render();
    }
}

impl App {
    fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
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

            self.renderer.render(&self.gpu, self.size, &mut render_pass);
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
