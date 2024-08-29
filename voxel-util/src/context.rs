use std::{marker::PhantomData, sync::Arc};

use thiserror::Error;
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    CreateSurfaceError, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor,
    PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
};
use winit::window::Window;

use crate::{
    bind_group::{IntoBindingResources, IntoLayout, Layout, ShaderResource},
    BasePipeline, RenderPipelineBuilder, VertexLayout,
};

#[derive(Debug, Error, Clone)]
pub enum ContextError {
    #[error("failed to create surface")]
    Surface(CreateSurfaceError),
    #[error("failed to get device: {0}")]
    Device(RequestDeviceError),
    #[error("invalid surface configuration")]
    Config,
    #[error("could not find adapter")]
    Adapter,
}

#[derive(Debug)]
pub struct Context {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl Context {
    pub async fn new(window: Arc<Window>) -> Result<Self, ContextError> {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window)
            .map_err(ContextError::Surface)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ContextError::Adapter)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::POLYGON_MODE_LINE,
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(ContextError::Device)?;

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(ContextError::Config)?;

        config.present_mode = PresentMode::AutoNoVsync;

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
        })
    }

    pub fn create_bind_group_layout<B: IntoLayout>(&self) -> Layout<B> {
        let entries = B::into_binding_entries();

        Layout(
            self.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries,
                }),
            PhantomData::default(),
        )
    }

    pub fn create_shader_resource<'s, L: IntoLayout>(
        &self,
        bindings: L::Bindings<'s>,
    ) -> ShaderResource {
        let bind_group_layout = self.create_bind_group_layout::<L>();
        let bind_group = self.create_bind_group(&bind_group_layout, bindings);

        ShaderResource::new(bind_group, bind_group_layout.erase())
    }

    pub fn create_bind_group<'s, L: IntoLayout>(
        &self,
        layout: &Layout<L>,
        bindings: L::Bindings<'s>,
    ) -> BindGroup {
        let resources = bindings.into_binding_resources();

        self.device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &layout.0,
            entries: &resources,
        })
    }

    pub fn create_pipeline_layout(
        &self,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        self.device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            })
    }

    pub fn create_render_pipeline<'c, V: VertexLayout>(
        &'c self,
        base_pipeline: BasePipeline<'c>,
    ) -> RenderPipelineBuilder<'c> {
        RenderPipelineBuilder::new::<V>(self, base_pipeline)
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
}
