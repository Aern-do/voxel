use std::collections::HashMap;

use smallvec::SmallVec;
use wgpu::{
    BlendComponent, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Face, FragmentState, FrontFace, PipelineCompilationOptions, PipelineLayout,
    PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState,
    TextureFormat, VertexBufferLayout, VertexState,
};

use crate::Context;

pub trait VertexLayout {
    fn vertex_layout() -> VertexBufferLayout<'static>;
}

pub trait ColorTargetStateExt {
    fn builder(format: impl Into<TextureFormat>) -> ColorTargetStateBuilder;
}

impl ColorTargetStateExt for ColorTargetState {
    fn builder(format: impl Into<TextureFormat>) -> ColorTargetStateBuilder {
        ColorTargetStateBuilder::new(format)
    }
}

#[derive(Debug, Clone)]
pub struct ColorTargetStateBuilder {
    format: TextureFormat,
    blend: Option<BlendState>,
    write_mask: ColorWrites,
}

impl ColorTargetStateBuilder {
    pub fn new(format: impl Into<TextureFormat>) -> Self {
        Self {
            format: format.into(),
            blend: None,
            write_mask: ColorWrites::default(),
        }
    }

    pub fn blend(mut self, alpha: BlendComponent, color: BlendComponent) -> Self {
        self.blend = Some(BlendState { color, alpha });
        self
    }

    pub fn write_mask(mut self, write_mask: ColorWrites) -> Self {
        self.write_mask = write_mask;
        self
    }

    pub fn build(self) -> ColorTargetState {
        ColorTargetState {
            format: self.format,
            blend: self.blend,
            write_mask: self.write_mask,
        }
    }
}

impl Into<ColorTargetState> for ColorTargetStateBuilder {
    fn into(self) -> ColorTargetState {
        self.build()
    }
}

type Shader<'s> = (&'s ShaderModule, &'static str);

#[derive(Debug, Clone)]
pub struct BasePipeline<'s> {
    pub vertex: Shader<'s>,
    pub fragment: Shader<'s>,
}

#[derive(Debug, Clone)]
pub struct RenderPipelineBuilder<'c> {
    context: &'c Context,
    base_pipeline: BasePipeline<'c>,
    vertex_layout: VertexBufferLayout<'static>,
    targets: SmallVec<[Option<ColorTargetState>; 4]>,

    label: Option<&'static str>,
    layout: Option<&'c PipelineLayout>,
    depth: Option<(TextureFormat, CompareFunction)>,
    depth_write: bool,

    overrides: HashMap<String, f64>,

    cull_mode: Option<Face>,
    front_face: Option<FrontFace>,
}

impl<'c> RenderPipelineBuilder<'c> {
    pub fn new<V: VertexLayout>(context: &'c Context, base_pipeline: BasePipeline<'c>) -> Self {
        Self {
            context,
            base_pipeline,
            vertex_layout: V::vertex_layout(),
            targets: SmallVec::new(),
            layout: None,
            depth_write: true,
            label: None,
            depth: None,
            cull_mode: None,
            front_face: None,
            overrides: HashMap::new(),
        }
    }

    pub fn label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn layout(mut self, layout: &'c PipelineLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn depth(mut self, format: impl Into<TextureFormat>, compare: CompareFunction) -> Self {
        self.depth = Some((format.into(), compare));
        self
    }

    pub fn depth_write(mut self, depth_write: bool) -> Self {
        self.depth_write = depth_write;
        self
    }

    pub fn cull_mode(mut self, cull_mode: Face) -> Self {
        self.cull_mode = Some(cull_mode);
        self
    }

    pub fn front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = Some(front_face);
        self
    }

    pub fn target(mut self, target: impl Into<ColorTargetState>) -> Self {
        self.targets.push(Some(target.into()));
        self
    }

    pub fn override_const(mut self, name: impl Into<String>, value: f64) -> Self {
        self.overrides.insert(name.into(), value);
        self
    }

    pub fn build(self) -> RenderPipeline {
        let (vertex_shader, vertex_entry_point) = self.base_pipeline.vertex;
        let vertex_state = VertexState {
            module: &vertex_shader,
            entry_point: vertex_entry_point,
            compilation_options: PipelineCompilationOptions {
                constants: &self.overrides,
                ..Default::default()
            },
            buffers: &[self.vertex_layout],
        };

        let (fragment_shader, fragment_entry_point) = self.base_pipeline.fragment;
        let fragment_state = FragmentState {
            module: &fragment_shader,
            entry_point: fragment_entry_point,
            compilation_options: PipelineCompilationOptions {
                constants: &self.overrides,
                ..Default::default()
            },
            targets: &self.targets,
        };

        let primitive_state = PrimitiveState {
            front_face: self.front_face.unwrap_or_default(),
            cull_mode: self.cull_mode,
            ..Default::default()
        };

        let depth = self.depth.map(|(format, depth_compare)| DepthStencilState {
            format,
            depth_write_enabled: self.depth_write,
            depth_compare,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });

        self.context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: self.label,
                layout: self.layout,
                vertex: vertex_state,
                primitive: primitive_state,
                depth_stencil: depth,
                multisample: Default::default(),
                fragment: Some(fragment_state),
                multiview: None,
            })
    }
}
