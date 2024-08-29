use thiserror::Error;
use voxel_util::context::ContextError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create context")]
    Context(#[from] ContextError),
}