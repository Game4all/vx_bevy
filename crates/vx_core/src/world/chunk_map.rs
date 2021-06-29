use bevy::ecs::system::SystemParam;
use bevy::{math::IVec2, prelude::*, utils::HashMap};
use building_blocks::storage::Array3x1;

pub type ChunkEntityMap = HashMap<IVec2, Entity>;
pub type ChunkDataMap = HashMap<IVec2, Array3x1<crate::voxel::Voxel>>;

#[derive(SystemParam)]
pub struct ChunkMapReader<'a> {
    pub chunk_entities: Res<'a, ChunkEntityMap>,
    pub chunk_data: Res<'a, ChunkDataMap>,
}

#[derive(SystemParam)]
pub struct ChunkMapWriter<'a> {
    pub chunk_entities: ResMut<'a, ChunkEntityMap>,
    pub chunk_data: ResMut<'a, ChunkDataMap>,
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
        self.chunk_data.get(chunk_coords)
    }
}

impl<'a> ChunkMapWriter<'a> {
    #[inline]
    pub fn insert_entity(&mut self, chunk_coords: IVec2, entity: Entity) {
        self.chunk_entities.insert(chunk_coords, entity);
    }

    #[inline]
    pub fn remove_entity(&mut self, chunk_coords: &IVec2) -> Entity {
        self.chunk_entities
            .remove(chunk_coords)
            .expect("Chunk is missing an attached entity")
    }

    pub fn get_chunk_data_mut(
        &mut self,
        chunk_coords: &IVec2,
    ) -> Option<&mut Array3x1<crate::voxel::Voxel>> {
        self.chunk_data.get_mut(chunk_coords)
    }
}
