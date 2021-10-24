use building_blocks::{
    core::{ExtentN, Point, Point3i, PointN},
    storage::{Array3x1, FillExtent, GetMut},
};

use super::TerrainGenerator;
use crate::{utils::ValueMap2D, voxel::Voxel, world::CHUNK_LENGTH};

pub struct NoiseTerrainGenerator;

impl TerrainGenerator for NoiseTerrainGenerator {
    fn fill(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>) -> bool {
        if chunk_min.y() < 0 {
            data.fill_extent(
                &ExtentN::from_min_and_shape(chunk_min, PointN::fill(CHUNK_LENGTH)),
                Voxel::Solid {
                    attributes: [236, 230, 214, 255],
                },
            );
            return true;
        } else if chunk_min.y() == 0 {
            data.fill_extent(
                &ExtentN::from_min_and_max(
                    chunk_min + PointN([0, 1, 0]),
                    PointN([CHUNK_LENGTH, 8, CHUNK_LENGTH]),
                ),
                Voxel::Fluid {
                    attributes: [102, 133, 254, 255],
                },
            );
            return false;
        } else {
            return false;
        }
    }

    fn shape_terrain(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>) {
        let heightmap = ValueMap2D::new(
            CHUNK_LENGTH,
            CHUNK_LENGTH,
            simdnoise::NoiseBuilder::fbm_2d_offset(
                chunk_min.x() as f32,
                CHUNK_LENGTH as usize,
                chunk_min.z() as f32,
                CHUNK_LENGTH as usize,
            )
            .with_octaves(5)
            .generate()
            .0,
        );

        let base_height = chunk_min.y();

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                let block_height = (heightmap.value_at(x, z).abs()
                    * NoiseTerrainGenerator::MAX_TERRAIN_HEIGHT as f32)
                    as i32;

                let local_height = (block_height - base_height).max(0).min(CHUNK_LENGTH);

                for y in 0..local_height {
                    *data.get_mut(chunk_min + PointN([x, y, z])) = Voxel::Solid {
                        attributes: self.get_color_for_height(base_height + y),
                    }
                }
            }
        }
    }
}

impl NoiseTerrainGenerator {
    const MAX_TERRAIN_HEIGHT: i32 = 256;

    #[inline]
    fn get_color_for_height(&self, height: i32) -> [u8; 4] {
        if height < 12 {
            [236, 230, 214, 255]
        } else if height < 24 {
            [96, 200, 102, 255]
        } else if height < 83 {
            [64, 152, 72, 255]
        } else if height < 102 {
            [122, 121, 87, 255]
        } else if height < 115 {
            [99, 99, 88, 255]
        } else {
            [255; 4]
        }
    }
}
