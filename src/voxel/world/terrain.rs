use super::{
    chunks::{ChunkLoadingStage, DirtyChunks},
    Chunk, ChunkShape,
};
use crate::voxel::{
    storage::{VoxelBuffer, VoxelMap},
    terraingen::TERRAIN_GENERATOR,
    Voxel,
};
use bevy::{
    prelude::{
        Added, Commands, Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, Res,
        ResMut, StageLabel, SystemLabel, SystemStage,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

/// Queues the terrain gen async tasks for the newly created chunks.
fn queue_terrain_gen(
    mut commands: Commands,
    new_chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    new_chunks
        .iter()
        .filter(|(_, key)| key.0.location().y < 288)
        .map(|(entity, key)| (entity, key.0.clone()))
        .map(|(entity, key)| {
            (
                entity,
                (TerrainGenTask(task_pool.spawn(async move {
                    let mut chunk_data = VoxelBuffer::<Voxel, ChunkShape>::new_empty(ChunkShape {});
                    TERRAIN_GENERATOR
                        .read()
                        .unwrap()
                        .generate(key, &mut chunk_data);
                    chunk_data
                }))),
            )
        })
        .for_each(|(entity, gen_task)| {
            commands.entity(entity).insert(gen_task);
        });
}

/// Polls for finished gen tasks and put back the generated terrain into the voxel map
fn process_terrain_gen(
    mut chunk_data: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut commands: Commands,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut gen_chunks: Query<(Entity, &Chunk, &mut TerrainGenTask)>,
) {
    gen_chunks.for_each_mut(|(entity, chunk, mut gen_task)| {
        if let Some(data) = future::block_on(future::poll_once(&mut gen_task.0)) {
            chunk_data.insert(chunk.0, data);
            dirty_chunks.mark_dirty(chunk.0);
            commands.entity(entity).remove::<TerrainGenTask>();
        }
    });
}

/// Handles terrain generation.
pub struct VoxelWorldTerrainGenPlugin;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
/// Labels for the systems added by [`VoxelWorldTerrainGenPlugin`]
pub enum TerrainGenSystem {
    /// Queues the terrain gen async tasks for the newly created chunks.
    QueueTerrainGen,
    /// Polls for finished gen tasks and put back the generated terrain into the voxel map
    ProcessTerrainGen,
}

// we need to use a whole system stage for this in order to enable the usage of added component querries.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
struct TerrainGenStage;

impl Plugin for VoxelWorldTerrainGenPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_stage_after(
            ChunkLoadingStage,
            TerrainGenStage,
            SystemStage::parallel()
                .with_system(queue_terrain_gen.label(TerrainGenSystem::QueueTerrainGen))
                .with_system(
                    process_terrain_gen
                        .label(TerrainGenSystem::ProcessTerrainGen)
                        .after(TerrainGenSystem::QueueTerrainGen),
                ),
        );
    }
}

#[derive(Component)]
struct TerrainGenTask(Task<VoxelBuffer<Voxel, ChunkShape>>);
