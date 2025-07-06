use std::borrow::Cow;

use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    VertexState,
};

use crate::pipelines::Gpu;

pub struct BackgroundPipeline {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    uniform: Buffer,
}

#[derive(ShaderType, Default)]
pub struct BackgroundUniform {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
}

impl BackgroundPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(concat!(
                include_str!("../../shaders/common.wgsl"),
                include_str!("../../shaders/background.wgsl")
            ))),
        });

        let uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: BackgroundUniform::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: gpu.texture_format,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            uniform,
        }
    }

    pub fn prepare(&mut self, gpu: &Gpu, data: &BackgroundUniform) {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(data).unwrap();
        gpu.queue
            .write_buffer(&self.uniform, 0, &buffer.into_inner());
    }

    pub fn paint(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
