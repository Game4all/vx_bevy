use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::*;
use building_blocks::{
    core::{ExtentN, PointN},
    storage::{Array3x1, FillExtent},
};

use super::{
    chunk_extent, ChunkInfo, ChunkLoadRequest, ChunkLoadState, ChunkMapWriter, WorldTaskPool,
    CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, MAX_FRAME_CHUNK_GEN_COUNT,
};
use crate::voxel::Voxel;

pub trait TerrainGenerator {
    fn generate(&self, chunk_pos: IVec2, data: &mut Array3x1<Voxel>);

    fn set_seed(&mut self, seed: i32);
}

#[derive(Default)]
pub struct NoiseTerrainGenerator {
    seed: i32,
}

impl TerrainGenerator for NoiseTerrainGenerator {
    fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    fn generate(&self, chunk_pos: IVec2, data: &mut Array3x1<Voxel>) {
        let heightmap = simdnoise::NoiseBuilder::fbm_2d_offset(
            (chunk_pos.x * CHUNK_WIDTH) as f32,
            CHUNK_WIDTH as usize,
            (chunk_pos.y * CHUNK_DEPTH) as f32,
            CHUNK_DEPTH as usize,
        )
        .with_seed(self.seed)
        .with_octaves(5)
        .generate()
        .0;

        data.fill_extent(
            &ExtentN::from_min_and_max(PointN([0; 3]), PointN([CHUNK_WIDTH, 4, CHUNK_DEPTH])),
            Voxel::Fluid {
                attributes: [102, 133, 254, 255],
            },
        );

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let original_height = heightmap.get((z * CHUNK_WIDTH + x) as usize).unwrap().abs();

                let height = original_height * 8.0;
                let block_height = (original_height * CHUNK_HEIGHT as f32) as i32;
                let color = self.get_color_for_height(height);

                let extent =
                    ExtentN::from_min_and_max(PointN([x, 0, z]), PointN([x, block_height, z]));
                data.fill_extent(&extent, Voxel::Solid { attributes: color })
            }
        }
    }
}

impl NoiseTerrainGenerator {
    fn get_color_for_height(&self, height: f32) -> [u8; 4] {
        if height < 0.30 {
            [236, 230, 214, 255]
        } else if height < 0.45 {
            [96, 200, 102, 255]
        } else if height < 0.65 {
            [64, 152, 72, 255]
        } else if height < 0.8 {
            [122, 121, 87, 255]
        } else if height < 0.9 {
            [99, 99, 88, 255]
        } else {
            [255; 4]
        }
    }
}

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
