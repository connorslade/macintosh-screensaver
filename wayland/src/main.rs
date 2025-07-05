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
        wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
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
    gpu: Gpu,
    exit: bool,

    outputs: Vec<Output>,
}

struct Output {
    surface: Surface<'static>,
    layer: LayerSurface,
    scale_factor: u32,
    needs_config: bool,
    size: Vector2<u32>,
}

fn main() -> Result<()> {
    let instance = Instance::new(&InstanceDescriptor::default());
    let adapter =
        pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
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

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;

    let mut app = App {
        output_state: OutputState::new(&globals, &qh),
        seat_state: SeatState::new(&globals, &qh),
        registry_state: RegistryState::new(&globals),

        renderer,
        gpu,
        exit: false,

        outputs: Vec::new(),
    };

    event_queue.roundtrip(&mut app)?;

    for output in app.output_state.outputs() {
        let surface = compositor_state.create_surface(&qh);
        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Background,
            Some("com.connorcode.macintosh-wallpaper"),
            Some(&output),
        );
        layer.commit();

        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(layer.wl_surface().id().as_ptr() as _).unwrap(),
        ));

        let handle = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle,
            raw_window_handle,
        };
        let surface = unsafe { app.gpu.instance.create_surface_unsafe(handle)? };

        let scale = app.output_state.info(&output).unwrap().scale_factor as u32;
        app.outputs.push(Output::new(layer, surface, scale));
    }

    while !app.exit {
        event_queue.blocking_dispatch(&mut app)?;
    }

    Ok(())
}

impl Output {
    pub fn new(layer: LayerSurface, surface: Surface<'static>, scale_factor: u32) -> Self {
        Self {
            surface,
            layer,
            scale_factor,
            needs_config: true,
            size: Vector2::zeros(),
        }
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _window: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let Some(layer) = self.outputs.iter().position(|x| &x.layer == layer) else {
            return;
        };
        let output = &mut self.outputs[layer];

        let size = Vector2::new(configure.new_size.0, configure.new_size.1) * output.scale_factor;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.gpu.texture_format,
            view_formats: vec![],
            alpha_mode: CompositeAlphaMode::Auto,
            width: size.x,
            height: size.y,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::Mailbox,
        };

        output.surface.configure(&self.gpu.device, &surface_config);
        output.size = size;

        if mem::take(&mut output.needs_config) {
            let wl_surface = output.layer.wl_surface();
            wl_surface.frame(qh, wl_surface.clone());
        }

        self.render(layer);
    }
}

impl App {
    fn render(&mut self, output: usize) {
        let output = &mut self.outputs[output];

        let surface_texture = output.surface.get_current_texture().unwrap();
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

            self.renderer
                .render(&self.gpu, output.size, &mut render_pass);
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
