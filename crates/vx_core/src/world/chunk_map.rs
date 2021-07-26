use bevy::ecs::system::SystemParam;
use bevy::{math::IVec2, prelude::*, utils::HashMap};
use building_blocks::storage::{Array3x1, ChunkHashMap3x1, ChunkKey3};

use super::{chunk2point, ChunkUpdateEvent};

pub type ChunkEntityMap = HashMap<IVec2, Entity>;

#[derive(SystemParam)]
pub struct ChunkMapReader<'a> {
    pub chunk_entities: Res<'a, ChunkEntityMap>,
    pub chunk_data: Res<'a, ChunkHashMap3x1<crate::voxel::Voxel>>,
}

#[derive(SystemParam)]
pub struct ChunkMapWriter<'a> {
    pub chunk_entities: ResMut<'a, ChunkEntityMap>,
    pub chunk_data: ResMut<'a, ChunkHashMap3x1<crate::voxel::Voxel>>,
    pub chunk_updates: EventWriter<'a, ChunkUpdateEvent>,
}

impl<'a> ChunkMapReader<'a> {
    #[inline]
    pub fn chunk_exists(&self, chunk_coords: &IVec2) -> bool {
        self.chunk_entities.contains_key(chunk_coords)
    }

    pub fn get_entity(&self, chunk_coords: &IVec2) -> Option<Entity> {
        self.chunk_entities.get(chunk_coords).map(|entity| *entity)
    }

    pub fn get_chunk_data(&self, chunk_coords: &IVec2) -> Option<&Array3x1<crate::voxel::Voxel>> {
        self.chunk_data
            .get_chunk(ChunkKey3::new(0, chunk2point(*chunk_coords)))
    }
}

impl<'a> ChunkMapWriter<'a> {
    pub fn get_chunk_data_mut(
        &mut self,
        chunk_coords: &IVec2,
    ) -> Option<&mut Array3x1<crate::voxel::Voxel>> {
        self.chunk_data
            .get_mut_chunk(ChunkKey3::new(0, chunk2point(*chunk_coords)))
    }

    pub fn mark_updated(&mut self, chunk_coords: &IVec2) {
        if let Some(entity) = self.chunk_entities.get(chunk_coords) {
            self.chunk_updates.send(ChunkUpdateEvent(*entity));
        }
    }
}
