use bevy::{math::IVec2, utils::HashMap, prelude::*};
use building_blocks::storage::Array3x1;

#[derive(Default)]
pub struct ChunkMap {
    pub entities: HashMap<IVec2, Entity>,

    //todo: this should use a ChunkMap from building-blocks once I understand how to do so.
    //todo: pool chunk data arrays.
    pub chunks: HashMap<IVec2, Array3x1<crate::voxel::Voxel>>,
}