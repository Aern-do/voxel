pub mod renderer;
pub mod vertex;
pub mod world_pass;
pub mod frustum_culling;

use frustum_culling::Frustum;
pub use renderer::Renderer;
pub use vertex::Vertex;
use wgpu::RenderPass;

use crate::world2::World;

pub trait Draw {
    fn draw<'r>(&'r self, render_pass: &mut RenderPass<'r>, frustum: &Frustum, world: &'r World);
}
