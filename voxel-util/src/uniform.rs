use std::{fmt::Debug, num::NonZero};

use bytemuck::Pod;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
};

use crate::{Binding, Context};

#[derive(Debug)]
pub struct Uniform<T> {
    data: T,
    buffer: Buffer,
}

impl<T: Pod> Uniform<T> {
    pub fn new(data: T, context: &Context) -> Self {
        let buffer = context.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self { data, buffer }
    }

    pub fn map<F>(&mut self, map: F, context: &Context)
    where
        F: FnOnce(T) -> T,
    {
        self.update(map(self.data), context)
    }

    pub fn update(&mut self, data: T, context: &Context) {
        self.data = data;
        context
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T> Binding for Uniform<T> {
    fn resource(&self) -> BindingResource {
        self.buffer.as_entire_binding()
    }

    fn ty() -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    fn count() -> Option<NonZero<u32>> {
        None
    }
}
