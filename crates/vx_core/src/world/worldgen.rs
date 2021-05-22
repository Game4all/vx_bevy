use bevy::prelude::Mut;
use building_blocks::core::{ExtentN, PointN};

use super::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::voxel::Voxel;

pub(crate) fn generate_chunk(mut chunk: Mut<Chunk>) {
    let noise = simdnoise::NoiseBuilder::fbm_2d_offset(
        (chunk.pos.x * CHUNK_WIDTH) as f32,
        CHUNK_WIDTH as usize,
        (chunk.pos.y * CHUNK_DEPTH) as f32,
        CHUNK_DEPTH as usize,
    )
    .generate()
    .0;

    for x in 0..CHUNK_WIDTH {
        for z in 0..CHUNK_DEPTH {
            let height =
                noise.get((z * CHUNK_WIDTH + x) as usize).unwrap() * CHUNK_HEIGHT as f32 * 4.0;

            let block_height = (height.round() as i32).max(0).min(CHUNK_HEIGHT - 1);

            chunk.block_data.fill_extent(
                &ExtentN::from_min_and_max(PointN([0; 3]), PointN([CHUNK_WIDTH, 0, CHUNK_DEPTH])),
                Voxel {
                    attributes: [194, 178, 128, 255],
                },
            );

            if block_height == 0 {
                chunk.block_data.fill_extent(
                    &ExtentN::from_min_and_max(
                        PointN([0; 3]),
                        PointN([CHUNK_WIDTH, 1, CHUNK_DEPTH]),
                    ),
                    Voxel {
                        attributes: [0, 10, 128, 255],
                    },
                );
            }

            if block_height > 8 {
                let extent = ExtentN::from_min_and_max(PointN([x, 0, z]), PointN([x, 8, z]));
                chunk.block_data.fill_extent(
                    &extent,
                    Voxel {
                        attributes: [194, 178, 128, 255],
                    },
                );
            }

            chunk.block_data.fill_extent(
                &ExtentN::from_min_and_max(PointN([x, 9, z]), PointN([x, block_height, z])),
                Voxel {
                    attributes: [99, 146, 103, 255],
                },
            );
        }
    }
}
