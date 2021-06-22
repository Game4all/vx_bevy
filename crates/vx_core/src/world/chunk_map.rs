use bevy::{math::IVec2, prelude::*, utils::HashMap};
use building_blocks::storage::Array3x1;

use super::chunk_extent;

#[derive(Default)]
pub struct ChunkMap {
    pub entities: HashMap<IVec2, Entity>,

    //todo: this should use a ChunkMap from building-blocks.
    //todo: pool chunk data arrays.
    pub chunks: HashMap<IVec2, Array3x1<crate::voxel::Voxel>>,
}

impl ChunkMap {
    pub fn is_chunk_loaded(&self, chunk_coords: &IVec2) -> bool {
        self.entities.contains_key(chunk_coords) && self.chunks.contains_key(chunk_coords)
    }

    pub fn attach_chunk(&mut self, chunk_coords: IVec2, entity: Entity) {
        self.entities.insert(chunk_coords, entity);
        self.chunks.insert(
            chunk_coords,
            Array3x1::fill(chunk_extent().padded(1), crate::voxel::Voxel::Empty),
        );
    }

    pub fn detach_chunk(&mut self, chunk_coords: &IVec2) -> Entity {
        self.chunks.remove(chunk_coords);
        self.entities.remove(chunk_coords).unwrap()
    }
}
