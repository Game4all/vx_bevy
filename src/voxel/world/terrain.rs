use bevy::{
    prelude::{
        Added, Commands, Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, Res,
        ResMut, StageLabel, SystemLabel, SystemStage,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

use super::{
    chunks::{ChunkLoadingStage, DirtyChunks},
    Chunk, ChunkKey, ChunkShape, Grass, Rock, Voxel, CHUNK_LENGTH,
};
use crate::voxel::storage::{VoxelBuffer, VoxelMap};

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
                    generate_terrain(key, &mut chunk_data);
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

// Terrain-gen stuff

const DEFAULT_TERRAIN_HEIGHT: u32 = 128; // equals to 4 vertical chunks

fn generate_terrain(key: ChunkKey, data: &mut VoxelBuffer<Voxel, ChunkShape>) {
    let heightmap: Vec<u32> = simdnoise::NoiseBuilder::fbm_2d_offset(
        key.location().x as f32,
        CHUNK_LENGTH as usize,
        key.location().z as f32,
        CHUNK_LENGTH as usize,
    )
    .with_octaves(5)
    .generate()
    .0
    .iter()
    .map(|x| DEFAULT_TERRAIN_HEIGHT as i32 + ((x * 256.0).round() as i32)) //todo: add a default 128 default height
    .map(|x| x - key.location().y)
    .map(|x| x.max(0).min((CHUNK_LENGTH) as i32))
    .map(|x| x as u32)
    .collect();

    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_LENGTH {
            for h in 0..heightmap[(z * CHUNK_LENGTH + x) as usize] {
                *data.voxel_at_mut([x, h, z].into()) = Voxel(Grass::ID);
            }
        }
    }

    if key.location().y == 0 {
        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                *data.voxel_at_mut([x, 0, z].into()) = Voxel(Rock::ID);
            }
        }
    }
}
