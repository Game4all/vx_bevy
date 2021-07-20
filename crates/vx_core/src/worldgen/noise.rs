use bevy::math::IVec2;
use building_blocks::{
    core::{ExtentN, PointN},
    storage::{Array3x1, FillExtent},
};

use crate::{
    voxel::Voxel,
    world::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH},
};

use super::TerrainGenerator;

#[derive(Default)]
pub struct NoiseTerrainGenerator {
    seed: i32,
}

impl TerrainGenerator for NoiseTerrainGenerator {
    fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    fn generate(&self, chunk_pos: IVec2, data: &mut Array3x1<Voxel>) {
        let heightmap = NoiseMap::new(chunk_pos, self.seed, 5);

        data.fill_extent(
            &ExtentN::from_min_and_max(PointN([0; 3]), PointN([CHUNK_WIDTH, 4, CHUNK_DEPTH])),
            Voxel::Fluid {
                attributes: [102, 133, 254, 255],
            },
        );

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                let original_height = heightmap.value_at(x, z).abs();

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

pub(crate) struct NoiseMap {
    noise: Vec<f32>,
}

impl NoiseMap {
    pub fn new(chunk_pos: IVec2, seed: i32, octave_nb: u8) -> Self {
        Self {
            noise: {
                simdnoise::NoiseBuilder::fbm_2d_offset(
                    (chunk_pos.x * CHUNK_WIDTH) as f32,
                    CHUNK_WIDTH as usize,
                    (chunk_pos.y * CHUNK_DEPTH) as f32,
                    CHUNK_DEPTH as usize,
                )
                .with_seed(seed)
                .with_octaves(octave_nb)
                .generate()
                .0
            },
        }
    }

    #[inline]
    pub fn value_at(&self, x: i32, z: i32) -> f32 {
        unsafe { *self.noise.get_unchecked((z * CHUNK_WIDTH + x) as usize) }
    }
}
