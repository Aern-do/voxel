use std::num::NonZero;
use wgpu::{BindingResource, BindingType, FilterMode, SamplerBindingType, SamplerDescriptor};

use crate::{Binding, Context};

#[derive(Debug)]
pub struct Sampler(wgpu::Sampler);

impl Sampler {
    pub fn new(filter: FilterMode, context: &Context) -> Self {
        let sampler = context.device().create_sampler(&SamplerDescriptor {
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        });

        Self(sampler)
    }
}

impl Binding for Sampler {
    fn ty() -> BindingType {
        BindingType::Sampler(SamplerBindingType::Filtering)
    }

    fn count() -> Option<NonZero<u32>> {
        None
    }

    fn resource(&self) -> BindingResource {
        BindingResource::Sampler(&self.0)
    }
}
