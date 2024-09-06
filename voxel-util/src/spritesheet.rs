use bytemuck::{Pod, Zeroable};
use wgpu::FilterMode;

use crate::{AsBindGroup, BindingEntries, Context, Fragment, Sampler, Texture, Uniform, Vertex};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TextureAtlasUniform {
    rows: u32,
    columns: u32,
}

#[derive(Debug)]
pub struct Spritesheet {
    texture: Texture,
    sampler: Sampler,
    uniform: Uniform<TextureAtlasUniform>,
}

impl Spritesheet {
    pub fn new(texture: Texture, texture_size: u32, context: &Context) -> Self {
        let (width, height) = texture.size();

        let columns = width / texture_size;
        let rows = height / texture_size;

        Self {
            uniform: Uniform::new(TextureAtlasUniform { rows, columns }, context),
            texture,
            sampler: Sampler::new(FilterMode::Nearest, context),
        }
    }
}

impl AsBindGroup for Spritesheet {
    type BindingEntries = (
        (Fragment, Texture),
        (Fragment, Sampler),
        (Vertex, Uniform<TextureAtlasUniform>),
    );

    fn resources(&self) -> <Self::BindingEntries as BindingEntries>::Bindings<'_> {
        (&self.texture, &self.sampler, &self.uniform)
    }
}
