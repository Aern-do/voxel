use glam::IVec3;
use voxel_util::{
    AsBindGroup, BasePipeline, Context, ShaderResource, Spritesheet, Texture, Uniform,
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, Buffer, BufferUsages, CompareFunction, Face, FrontFace, IndexFormat,
    RenderPass, RenderPipeline, TextureFormat, TextureUsages,
};

use crate::{
    asset,
    world2::{chunk::Volume, Chunk, RawMesh, World},
};

use super::{
    frustum_culling::{Frustum, AABB},
    vertex::Vertex,
    Draw,
};

type Transformation = (voxel_util::Vertex, Uniform<IVec3>);

#[derive(Debug)]
pub struct ChunkBuffer {
    vertices: Buffer,
    indices: Buffer,
    indices_len: u32,

    transformation_resource: ShaderResource,
    aabb: AABB,
}

impl ChunkBuffer {
    pub fn from_mesh(mesh: &RawMesh, transformation: IVec3, context: &Context) -> Self {
        let indices_len = mesh.indices().len() as u32;

        let vertices = context.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mesh.verticies()),
            usage: BufferUsages::VERTEX,
        });

        let indices = context.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(mesh.indices()),
            usage: BufferUsages::INDEX,
        });

        let min = transformation * Chunk::SIZE as i32;
        let aabb = AABB::new(min.as_vec3(), (min + Chunk::SIZE as i32 - 1).as_vec3());

        let transformation_resource = context
            .create_shader_resource::<Transformation>(&Uniform::new(transformation, context));

        Self {
            vertices,
            indices,
            indices_len,
            transformation_resource,
            aabb,
        }
    }
}

#[derive(Debug)]
pub struct WorldPass {
    render_pipeline: RenderPipeline,
    spritesheet_resource: ShaderResource,
}

impl WorldPass {
    pub fn new(camera_resource: &ShaderResource, context: &Context) -> Self {
        let spritesheet = image::load_from_memory(include_bytes!(asset!("texture.png")))
            .expect("failed to load spritesheet");
        let spritesheet = Texture::from_data(
            &spritesheet.to_rgba8(),
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            context,
        );

        let spritesheet = Spritesheet::new(spritesheet, 16, context);
        let spritesheet_resource = spritesheet.as_shader_resource(context);

        let render_pipeline = Self::create_pipeline(
            camera_resource.layout(),
            &spritesheet_resource.layout(),
            context,
        );

        Self {
            render_pipeline,
            spritesheet_resource,
        }
    }

    fn create_pipeline(
        camera_layout: &BindGroupLayout,
        spritesheet_layout: &BindGroupLayout,
        context: &Context,
    ) -> RenderPipeline {
        let shader = context
            .device()
            .create_shader_module(include_wgsl!(asset!("shaders/world.wgsl")));

        let transformation_layout = context.create_bind_group_layout::<Transformation>().erase();
        let pipeline_layout = context.create_pipeline_layout(&[
            camera_layout,
            spritesheet_layout,
            &transformation_layout,
        ]);

        context
            .create_render_pipeline::<Vertex>(BasePipeline {
                vertex: (&shader, "vs_main"),
                fragment: (&shader, "fs_main"),
            })
            .label("World Render Pipeline")
            .layout(&pipeline_layout)
            .target(context.config().format)
            .depth(TextureFormat::Depth32Float, CompareFunction::Less)
            .front_face(FrontFace::Cw)
            .cull_mode(Face::Back)
            .build()
    }
}

impl Draw for WorldPass {
    fn draw<'r>(&'r self, render_pass: &mut RenderPass<'r>, frustum: &Frustum, world: &'r World) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, self.spritesheet_resource.bind_group(), &[]);

        for chunk_buffer in world.meshes.values() {
            if chunk_buffer.aabb.is_on_frustum(&frustum) {
                render_pass.set_bind_group(
                    2,
                    chunk_buffer.transformation_resource.bind_group(),
                    &[],
                );
                render_pass.set_vertex_buffer(0, chunk_buffer.vertices.slice(..));
                render_pass.set_index_buffer(chunk_buffer.indices.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..chunk_buffer.indices_len, 0, 0..1);
            }
        }
    }
}
