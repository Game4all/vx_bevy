use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::*;
use building_blocks::storage::Array3x1;

use super::{
    chunk_extent, ChunkInfo, ChunkLoadRequest, ChunkLoadState, ChunkMapWriter, WorldTaskPool,
    MAX_FRAME_CHUNK_GEN_COUNT,
};
use crate::worldgen::{NoiseTerrainGenerator, TerrainGenerator};

pub(crate) fn generate_terrain_data(
    mut query: Query<(&ChunkInfo, &mut ChunkLoadState)>,
    mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
    mut chunk_map: ChunkMapWriter,
    gen: Res<Arc<NoiseTerrainGenerator>>,
    task_pool: Res<WorldTaskPool>,
) {
    let chunks = task_pool.scope(|scope| {
        let gen_req_count = gen_requests.len().min(MAX_FRAME_CHUNK_GEN_COUNT);
        for req in gen_requests.drain(..gen_req_count) {
            if let Ok(info) = query.get_component::<ChunkInfo>(req.0) {
                let generator = gen.clone();
                scope.spawn(async move {
                    let mut data = Array3x1::fill(chunk_extent().padded(1), Default::default());
                    generator.generate(info.pos, &mut data);
                    (req.0, data)
                });
            }
        }
    });

    for (entity, chunk_data) in chunks {
        if let Ok((info, mut load_state)) = query.get_mut(entity) {
            chunk_map.chunk_data.insert(info.pos, chunk_data);
            *load_state = ChunkLoadState::Loading;
        }
    }
}
