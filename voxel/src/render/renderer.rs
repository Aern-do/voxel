use std::{iter, sync::Arc, time::Duration};
use voxel_util::{Context, ShaderResource, Texture};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};

use crate::world::meshes::Meshes;

use super::{frustum_culling::Frustum, world_pass::WorldPass, DebugPass};

pub struct Renderer {
    context: Arc<Context>,
    camera_resource: ShaderResource,
    depth_texture: Texture,

    world_pass: WorldPass,
    debug_pass: DebugPass,
}

impl Renderer {
    pub fn new(camera_resource: ShaderResource, context: Arc<Context>) -> Self {
        let depth_texture = Texture::new(
            (context.config().width, context.config().height),
            TextureUsages::RENDER_ATTACHMENT,
            TextureFormat::Depth32Float,
            &context,
        );

        let world_pass = WorldPass::new(&camera_resource, &context);
        let debug_pass = DebugPass::new(&context);

        Self {
            context,
            camera_resource,
            depth_texture,
            world_pass,
            debug_pass,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.debug_pass.update(delta_time, &self.context);
    }

    pub fn draw(&mut self, frustum: &Frustum, meshes: &Meshes) {
        let output = self
            .context
            .surface()
            .get_current_texture()
            .expect("failed to get surface texture");

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .context
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::WHITE),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            render_pass.set_bind_group(0, self.camera_resource.bind_group(), &[]);
            self.world_pass.draw(&mut render_pass, frustum, meshes);
        }

        {
            let mut text_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.debug_pass.draw(&mut text_render_pass);
        }

        self.context.queue().submit(iter::once(encoder.finish()));
        output.present();
    }
}
