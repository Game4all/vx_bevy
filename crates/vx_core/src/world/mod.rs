use bevy::prelude::*;
use building_blocks::core::{Extent3i, PointN};
use std::collections::VecDeque;

mod world;
pub use world::*;

mod worldgen;

pub const CHUNK_HEIGHT: i32 = 128;
pub const CHUNK_WIDTH: i32 = 16;
pub const CHUNK_DEPTH: i32 = 16;

#[inline]
pub fn chunk_extent() -> Extent3i {
    Extent3i::from_min_and_shape(
        PointN([0; 3]),
        PointN([CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH]),
    )
}

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunk(position: Vec3) -> IVec2 {
    IVec2::new(
        position.x.floor() as i32 / CHUNK_WIDTH,
        position.z.floor() as i32 / CHUNK_DEPTH,
    )
}

/// Transform a point in global space to a point in chunk space.
pub fn global2locali(pos: IVec3) -> IVec3 {
    IVec3::new(pos.x % CHUNK_WIDTH, pos.y, pos.z % CHUNK_DEPTH)
}

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunki(position: IVec3) -> IVec2 {
    IVec2::new(position.x / CHUNK_WIDTH, position.z / CHUNK_DEPTH)
}

/// Gets the origin of a chunk in global space from its chunk coordinates.
pub fn chunk2global(chunk_coords: IVec2) -> Vec3 {
    Vec3::new(
        (chunk_coords.x * CHUNK_WIDTH) as f32,
        0.,
        (chunk_coords.y * CHUNK_DEPTH) as f32,
    )
}

pub struct WorldSimulationPlugin;

impl Plugin for WorldSimulationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ChunkMap>()
            .init_resource::<VecDeque<ChunkLoadRequest>>()
            //todo: move this to a struct or smth else
            .init_resource::<worldgen::NoiseTerrainGenerator>()
            // internal events
            .add_event::<ChunkSpawnRequest>()
            .add_event::<ChunkDespawnRequest>()
            // public events
            .add_event::<ChunkReadyEvent>()
            // systems
            .add_system(world::update_visible_chunks.system())
            .add_system(world::create_chunks.system())
            .add_system(world::load_chunk_data.system())
            .add_system(world::generate_chunks.system())
            .add_system(world::prepare_for_unload.system())
            .add_system(world::mark_chunks_ready.system())
            .add_system(world::destroy_chunks.system());
    }
}
