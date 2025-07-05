// ↓ Needed due to a rust-analyzer bug
#![allow(dead_code)]

use std::{ptr::NonNull, time::Instant};

use anyhow::Result;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_seat, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, Proxy, QueueHandle,
        globals::registry_queue_init,
        protocol::{
            wl_output::{Transform, WlOutput},
            wl_seat::WlSeat,
            wl_surface::WlSurface,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        WaylandSurface,
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions,
    Surface, SurfaceTargetUnsafe,
    rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle},
};

struct App {
    output_state: OutputState,
    seat_state: SeatState,
    registry_state: RegistryState,

    gpu: GpuCtx,

    start: Instant,
    exit: bool,
}

struct GpuCtx {
    adapter: Adapter,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
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

    let mut app = App {
        output_state: OutputState::new(&globals, &qh),
        seat_state: SeatState::new(&globals, &qh),
        registry_state: RegistryState::new(&globals),

        gpu: GpuCtx {
            adapter,
            surface,
            device,
            queue,
        },

        start: Instant::now(),
        exit: false,
    };

    while !app.exit {
        event_queue.blocking_dispatch(&mut app)?;
    }

    Ok(())
}

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        new_transform: Transform,
    ) {
    }

    fn frame(&mut self, conn: &Connection, qh: &QueueHandle<Self>, surface: &WlSurface, time: u32) {
    }

    fn surface_enter(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &WlSurface,
        output: &WlOutput,
    ) {
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut smithay_client_toolkit::output::OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}
    fn update_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}
    fn output_destroyed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: WlOutput) {}
}

impl WindowHandler for App {
    fn request_close(&mut self, conn: &Connection, qh: &QueueHandle<Self>, window: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        serial: u32,
    ) {
        let (new_width, new_height) = configure.new_size;
        let width = new_width.map_or(256, |v| v.get());
        let height = new_height.map_or(256, |v| v.get());

        let cap = self.gpu.surface.get_capabilities(&self.gpu.adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: cap.formats[0],
            view_formats: vec![cap.formats[0]],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: width,
            height: height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        self.gpu
            .surface
            .configure(&self.gpu.device, &surface_config);

        // We don't plan to render much in this example, just clear the surface.
        let surface_texture = self
            .gpu
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        {
            let _renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: (self.start.elapsed().as_secs_f64().sin() + 1.0) / 2.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, conn: &Connection, qh: &QueueHandle<Self>, seat: WlSeat) {}

    fn new_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, conn: &Connection, qh: &QueueHandle<Self>, seat: WlSeat) {}
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}

delegate_compositor!(App);
delegate_output!(App);

delegate_seat!(App);

delegate_xdg_shell!(App);
delegate_xdg_window!(App);

delegate_registry!(App);
