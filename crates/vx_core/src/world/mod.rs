use bevy::{
    diagnostic::DiagnosticId,
    prelude::*,
    tasks::{TaskPool, TaskPoolBuilder},
};
use building_blocks::{
    core::{Extent3i, PointN},
    storage::{ChunkMapBuilder, ChunkMapBuilder3x1},
};
use heron::{CollisionShape, RigidBody};
use std::{collections::VecDeque, ops::Deref, sync::Arc};

mod meshing;
mod world;
mod worldgen;

mod chunk_map;
pub use chunk_map::*;

mod coords;
pub use coords::*;

use crate::{voxel::Voxel, worldgen::NoiseTerrainGenerator};

pub const CHUNK_HEIGHT: i32 = 128;
pub const CHUNK_WIDTH: i32 = 16;
pub const CHUNK_DEPTH: i32 = 16;

pub const MAX_FRAME_CHUNK_GEN_COUNT: usize = 16;

pub const CHUNK_MESHING_TIME: DiagnosticId = DiagnosticId::from_u128(489617772449846);
pub const CHUNK_DATA_GEN_TIME: DiagnosticId = DiagnosticId::from_u128(975647521301976);

/// A component tracking the current loading state of a chunk.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkLoadState {
    /// Chunk requested load of data from disk.
    LoadRequested,
    /// Chunk requested generation of its chunk data.
    Generate,
    /// The chunk is in a loading state (chunk data is loaded but still needs a few things like meshing before showing the chunk.)
    Loading,
    /// Chunk is done loading and is in an idle state.
    Idle,
    /// Chunk requested saving of data to disk.
    Unload,
    /// Chunk is queued to be despawned on next frame.
    Despawn,
}

pub(crate) struct ChunkSpawnRequest(IVec2);
pub(crate) struct ChunkDespawnRequest(Entity);

pub struct ChunkMeshingRequest(pub Entity);

pub(crate) struct ChunkLoadRequest(Entity);

/// An event signaling that a chunk and its data have finished loading and are ready to be displayed.
pub struct ChunkReadyEvent(pub IVec2, pub Entity);

/// An event signaling that the data of a chunk has been modified.
pub struct ChunkUpdateEvent(pub Entity);

/// A component describing a chunk.
pub struct ChunkInfo {
    pub pos: IVec2,
}

#[derive(Bundle)]
pub struct ChunkDataBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub chunk_info: ChunkInfo,
    pub mesh_info: ChunkMeshInfo,
    pub rigid_body: RigidBody,
    pub collision_shape: CollisionShape,
}

pub struct ChunkMeshInfo {
    pub fluid_mesh: Handle<Mesh>,
    pub chunk_mesh: Handle<Mesh>,
}

pub struct WorldTaskPool(TaskPool);

impl Default for WorldTaskPool {
    fn default() -> Self {
        Self(
            TaskPoolBuilder::new()
                .num_threads(4)
                .thread_name("WorldThreadPool".to_owned())
                .build(),
        )
    }
}

impl Deref for WorldTaskPool {
    type Target = TaskPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[inline]
pub fn chunk_extent() -> Extent3i {
    Extent3i::from_min_and_shape(
        PointN([0; 3]),
        PointN([CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH]),
    )
}

pub struct WorldSimulationPlugin;

impl Plugin for WorldSimulationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ChunkEntityMap>()
            .init_resource::<VecDeque<ChunkLoadRequest>>()
            .init_resource::<WorldTaskPool>()
            //todo: move this to a struct or smth else
            .init_resource::<Arc<NoiseTerrainGenerator>>()
            .insert_resource(
                ChunkMapBuilder3x1::new(
                    PointN([CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH]),
                    Voxel::Empty,
                )
                .build_with_hash_map_storage(),
            )
            // internal events
            .add_event::<ChunkSpawnRequest>()
            .add_event::<ChunkDespawnRequest>()
            .add_event::<ChunkUpdateEvent>()
            // public events
            .add_event::<ChunkReadyEvent>()
            .add_event::<ChunkMeshingRequest>()
            // systems
            .add_system(
                world::update_visible_chunks
                    .system()
                    .label("update_visible_chunks"),
            )
            .add_system(
                world::create_chunks
                    .system()
                    .label("create_chunks")
                    .after("update_visible_chunks"),
            )
            .add_system(
                world::load_chunk_data
                    .system()
                    .label("load_chunk_data")
                    .after("create_chunks"),
            )
            .add_system(
                worldgen::generate_terrain_data
                    .system()
                    .label("generate_terrain_data")
                    .after("load_chunk_data"),
            )
            .add_system(
                world::prepare_for_unload
                    .system()
                    .label("prepare_for_unload")
                    .after("generate_terrain_data"),
            )
            .add_system(
                world::mark_chunks_ready
                    .system()
                    .label("mark_chunks_ready")
                    .after("prepare_for_unload"),
            )
            .add_system(
                world::destroy_chunks
                    .system()
                    .label("destroy_chunks")
                    .after("mark_chunks_ready"),
            )
            .add_system(
                meshing::handle_chunk_loading_events
                    .system()
                    .label("handle_chunk_loading_events")
                    .after("destroy_chunks"),
            )
            .add_system(
                meshing::handle_chunk_update_events
                    .system()
                    .label("handle_chunk_update_events")
                    .after("handle_chunk_loading_events"),
            )
            .add_system(
                meshing::mesh_chunks
                    .system()
                    .label("mesh_chunks")
                    .after("handle_chunk_update_events"),
            );
    }
}
