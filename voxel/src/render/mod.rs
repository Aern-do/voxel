pub mod renderer;
pub mod vertex;
pub mod world_pass;
pub mod frustum_culling;
pub mod debug_pass;

pub use frustum_culling::Frustum;
pub use renderer::Renderer;
pub use vertex::Vertex;
pub use debug_pass::DebugPass;

use wgpu::RenderPass;
use crate::world::World;

pub trait Draw {
    fn draw<'r>(&'r self, render_pass: &mut RenderPass<'r>, frustum: &Frustum, world: &'r World);
}
