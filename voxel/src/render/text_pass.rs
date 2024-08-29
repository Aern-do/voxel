use std::{borrow::Cow, mem::size_of, sync::Arc};

use glam::Mat4;
use glyph_brush::{
    ab_glyph::{FontArc, FontRef},
    BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder, OwnedSection, Section,
};
use log::{debug, info};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BlendState, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Extent3d,
    FilterMode, FragmentState, ImageCopyTexture, ImageCopyTextureBase, IndexFormat,
    MultisampleState, Origin3d, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureAspect, TextureFormat, TextureUsages, VertexState,
};

use crate::{
    context::Context,
    include_asset_str,
    util::{
        texture::{Texture2d, TextureData},
        BindGroup, Sampler, Texture, Uniform,
    },
};

use super::vertex::GlyphVertex;

#[derive(Debug)]
pub struct TextPass {
    glyph_brush: GlyphBrush<GlyphVertex, glyph_brush::Extra>,

    _projection_uniform: Uniform<[[f32; 4]; 4]>,
    glyph_texture: Texture2d,

    glyph_vertex_buffer: Buffer,
    glyph_vertices: u32,

    render_pipeline: RenderPipeline,
    bind_group: BindGroup,
    context: Arc<Context>,
}

impl TextPass {
    pub fn new(context: Arc<Context>, font: FontArc) -> Self {
        let glyph_brush =
            GlyphBrushBuilder::using_font(font.clone()).build::<GlyphVertex, glyph_brush::Extra>();

        let glyph_texture = Texture2d::new(
            glyph_brush.texture_dimensions(),
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            TextureFormat::R8Unorm,
            &context,
        );

        let glyph_texture_sampler = Sampler::new(FilterMode::Linear, &context);
        pub fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
            [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, -2.0 / height, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ]
        }

        let projection_uniform = Uniform::new(
            ortho(
                context.config().width as f32,
                context.config().height as f32,
            ),
            &context,
        );

        let glyph_vertex_buffer = context.device().create_buffer(&BufferDescriptor {
            label: None,
            size: 0,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = context.create_bind_group((
            (ShaderStages::VERTEX, &projection_uniform),
            (ShaderStages::FRAGMENT, &glyph_texture),
            (ShaderStages::FRAGMENT, &glyph_texture_sampler),
        ));

        let render_pipeline = Self::create_pipeline(&bind_group, &context);

        Self {
            glyph_brush: GlyphBrushBuilder::using_font(font.clone())
                .build::<GlyphVertex, glyph_brush::Extra>(),
            glyph_vertex_buffer,
            glyph_vertices: 0,
            glyph_texture,
            _projection_uniform: projection_uniform,
            render_pipeline,
            bind_group,
            context,
        }
    }

    fn create_pipeline(bind_group: &BindGroup, context: &Context) -> RenderPipeline {
        let shader = context
            .device()
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(Cow::Borrowed(include_asset_str!("shaders/text.wgsl"))),
            });
        let layout = context
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Text Render Pipeline Layout"),
                bind_group_layouts: &[bind_group.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let render_pipeline = context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Text Render Pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[GlyphVertex::layout()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleStrip,
                    strip_index_format: Some(IndexFormat::Uint16),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &shader,
                    targets: &[Some(ColorTargetState {
                        format: context.config().format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                    entry_point: "fs_main",
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                multiview: None,
            });

        render_pipeline
    }

    pub fn queue(&mut self, lines: &[OwnedSection<glyph_brush::Extra>]) {
        for line in lines {
            self.glyph_brush.queue(line)
        }

        loop {
            match self.glyph_brush.process_queued(
                |region, texture| {
                    self.glyph_texture.upload_data_into_region(
                        TextureData::new(
                            texture,
                            (region.width(), region.height()),
                            TextureFormat::R8Unorm,
                        ),
                        (region.min[0], region.min[1], region.max[0], region.max[1]),
                        &self.context,
                    )
                },
                |glyph_vertex| GlyphVertex::from(glyph_vertex),
            ) {
                Ok(BrushAction::Draw(glyph_vertices)) => {
                    if glyph_vertices.len() as u32 > self.glyph_vertices {
                        info!(
                            "grow of glyph vertex buffer {}({} bytes) -> {}({} bytes)",
                            self.glyph_vertices,
                            self.glyph_vertices as usize * size_of::<GlyphVertex>(),
                            glyph_vertices.len(),
                            glyph_vertices.len() * size_of::<GlyphVertex>()
                        );
                        self.glyph_vertex_buffer =
                            self.context
                                .device()
                                .create_buffer_init(&BufferInitDescriptor {
                                    label: None,
                                    contents: bytemuck::cast_slice(&glyph_vertices),
                                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                                });
                        self.glyph_vertices = glyph_vertices.len() as u32;
                    } else {
                        self.context.queue().write_buffer(
                            &self.glyph_vertex_buffer,
                            0,
                            bytemuck::cast_slice(&glyph_vertices),
                        )
                    }
                    break;
                }
                Ok(BrushAction::ReDraw) => break,
                Err(BrushError::TextureTooSmall { suggested }) => {
                    self.glyph_texture = Texture2d::new(
                        suggested,
                        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                        TextureFormat::R8Unorm,
                        &self.context,
                    )
                }
            }
        }
    }

    pub fn draw<'r>(&'r mut self, render_pass: &mut RenderPass<'r>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.glyph_vertex_buffer.slice(..));
        render_pass.set_bind_group(0, self.bind_group.bind_group(), &[]);
        render_pass.draw(0..4, 0..self.glyph_vertices);
    }
}
