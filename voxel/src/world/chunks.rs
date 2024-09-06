use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use glam::IVec3;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::chunk::Chunk;

type RawChunks = HashMap<IVec3, Chunk>;

#[derive(Default, Clone)]
pub struct Chunks {
    chunks: Arc<RwLock<RawChunks>>,
}

impl Chunks {
    pub fn read(&self) -> ChunksReadGuard<'_> {
        ChunksReadGuard(self.chunks.read())
    }

    pub fn write(&self) -> ChunksWriteGuard<'_> {
        ChunksWriteGuard(self.chunks.write())
    }
}

pub struct ChunksReadGuard<'s>(RwLockReadGuard<'s, RawChunks>);

impl Deref for ChunksReadGuard<'_> {
    type Target = RawChunks;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ChunksWriteGuard<'s>(RwLockWriteGuard<'s, RawChunks>);

impl Deref for ChunksWriteGuard<'_> {
    type Target = RawChunks;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChunksWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
