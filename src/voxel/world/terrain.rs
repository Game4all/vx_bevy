use bevy::{
    prelude::{
        Added, EventWriter, ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut,
        StageLabel, SystemStage,
    },
    tasks::ComputeTaskPool,
};
use std::collections::VecDeque;

use super::{
    chunks::{ChunkLoadingStage, ChunkUpdateEvent},
    Chunk, ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH,
};
use crate::voxel::storage::VoxelMap;

//todo: move this in the gen_terrain system.
fn queue_terrain_gen(chunks: Query<&Chunk, Added<Chunk>>, mut gen_queue: ResMut<TerrainGenQueue>) {
    gen_queue.0.extend(chunks.iter().map(|chunk| chunk.0));
}

fn gen_terrain(
    mut chunk_data: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut gen_queue: ResMut<TerrainGenQueue>,
    mut updates: EventWriter<ChunkUpdateEvent>,
    task_pool: Res<ComputeTaskPool>,
    gen_budget: Res<WorldTerrainGenFrameBudget>,
) {
    let drain_size = if gen_queue.0.len() < gen_budget.gen_per_frame {
        gen_queue.0.len()
    } else {
        gen_budget.gen_per_frame
    };
    //do the terrain gen here
    let generated_terrain = task_pool.scope(|scope| {
        gen_queue
            .0
            .drain(..drain_size)
            .map(|chunk_pos| (chunk_pos, chunk_data.remove(chunk_pos).unwrap()))
            .map(|(chunk_pos, mut buffer)| {
                scope.spawn_local(async move {
                    for x in (0..CHUNK_LENGTH).step_by(31) {
                        for z in 0..CHUNK_LENGTH {
                            *buffer.voxel_at_mut([x, 0, z].into()) = Voxel(1);
                            *buffer.voxel_at_mut([z, 0, x].into()) = Voxel(1);
                        }
                    }
                    (chunk_pos, buffer)
                })
            })
            .collect()
    });

    for (chunk_pos, buffer) in generated_terrain {
        chunk_data.insert(chunk_pos, buffer);
        updates.send(ChunkUpdateEvent(chunk_pos));
    }
}

/// Handles terrain generation.
pub struct VoxelWorldTerrainGenPlugin;

pub struct WorldTerrainGenFrameBudget {
    pub gen_per_frame: usize,
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
                .with_system(queue_terrain_gen.label("queue_terrain_gen"))
                .with_system(gen_terrain.after("queue_terrain_gen")),
        )
        .init_resource::<TerrainGenQueue>()
        .insert_resource(WorldTerrainGenFrameBudget { gen_per_frame: 16 });
    }
}

#[derive(Default)]
struct TerrainGenQueue(VecDeque<ChunkKey>);
