use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
    tasks::{TaskPool, TaskPoolBuilder},
};
use building_blocks::{
    core::{Extent3i, Point, Point3i, PointN},
    storage::{ChunkIndexer, ChunkMapBuilder, ChunkMapBuilder3x1},
};
use std::{collections::VecDeque, ops::Deref};

use crate::voxel::Voxel;

mod world;

mod chunk_map;
pub use chunk_map::*;

use self::world::update_visible_chunks_run_criteria;

pub const CHUNK_LENGTH: i32 = 32;

pub const MAX_FRAME_CHUNK_GEN_COUNT: usize = 16;
pub const CHUNK_DATA_GEN_TIME: DiagnosticId = DiagnosticId::from_u128(975647521301976);

#[derive(StageLabel, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum WorldUpdateStage {
    Update,
    PostUpdate,
    Cleanup,
}

/// A component tracking the current loading state of a chunk.
#[derive(PartialEq, Eq, PartialOrd, Ord, Component)]
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

pub(crate) struct ChunkSpawnRequest(Point3i);
pub(crate) struct ChunkDespawnRequest(Entity);

pub struct ChunkMeshingRequest(pub Entity);

pub(crate) struct ChunkLoadRequest(Entity);

/// An event signaling that a chunk and its data have finished loading and are ready to be displayed.
pub struct ChunkReadyEvent(pub Point3i, pub Entity);

/// An event signaling that the data of a chunk has been modified.
pub struct ChunkUpdateEvent(pub Entity);

/// A component describing a chunk.
#[derive(Component)]
pub struct ChunkInfo {
    pub pos: Point3i,
}

#[derive(Bundle)]
pub struct ChunkDataBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub chunk_info: ChunkInfo,
}

pub struct WorldChunkIndexer(ChunkIndexer<[i32; 3]>);

impl Default for WorldChunkIndexer {
    fn default() -> Self {
        Self(ChunkIndexer::new(Point3i::fill(CHUNK_LENGTH)))
    }
}

impl Deref for WorldChunkIndexer {
    type Target = ChunkIndexer<[i32; 3]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WorldTaskPool(TaskPool);

impl Default for WorldTaskPool {
    fn default() -> Self {
        Self(
            TaskPoolBuilder::new()
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
    Extent3i::from_min_and_shape(PointN([0; 3]), Point3i::fill(CHUNK_LENGTH))
}

pub struct WorldSimulationPlugin;

impl Plugin for WorldSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntityMap>()
            .init_resource::<VecDeque<ChunkLoadRequest>>()
            .init_resource::<WorldTaskPool>()
            .init_resource::<WorldChunkIndexer>()
            .insert_resource(
                ChunkMapBuilder3x1::new(PointN::fill(CHUNK_LENGTH), Voxel::Empty)
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
            .add_stage(WorldUpdateStage::Update, SystemStage::parallel())
            .add_stage_after(
                WorldUpdateStage::Update,
                WorldUpdateStage::PostUpdate,
                SystemStage::parallel(),
            )
            .add_stage_after(
                WorldUpdateStage::PostUpdate,
                WorldUpdateStage::Cleanup,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                WorldUpdateStage::Update,
                world::update_visible_chunks
                    .system()
                    .label("update_visible_chunks")
                    .with_run_criteria(update_visible_chunks_run_criteria.system()),
            )
            .add_system_to_stage(
                WorldUpdateStage::Update,
                world::create_chunks
                    .system()
                    .label("create_chunks")
                    .after("update_visible_chunks"),
            )
            .add_system_to_stage(
                WorldUpdateStage::Update,
                world::load_chunk_data
                    .system()
                    .label("load_chunk_data")
                    .after("create_chunks"),
            )
            .add_system_to_stage(
                WorldUpdateStage::Update,
                world::generate_terrain_data
                    .system()
                    .label("generate_terrain_data")
                    .after("load_chunk_data"),
            )
            .add_system_to_stage(
                WorldUpdateStage::Update,
                world::mark_chunks_ready
                    .system()
                    .label("mark_chunks_ready")
                    .after("generate_terrain_data"),
            )
            .add_system_to_stage(
                WorldUpdateStage::Cleanup,
                world::prepare_for_unload
                    .system()
                    .label("prepare_for_unload"),
            )
            .add_system_to_stage(
                WorldUpdateStage::Cleanup,
                world::destroy_chunks
                    .system()
                    .label("destroy_chunks")
                    .after("prepare_for_unload"),
            );

        //registering debug diagnostics
        app.world
            .resource_scope(|_, mut diagnostics: Mut<Diagnostics>| {
                diagnostics.add(Diagnostic::new(
                    CHUNK_DATA_GEN_TIME,
                    "Avg. worldgen time (s)",
                    3,
                ));
            });
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
