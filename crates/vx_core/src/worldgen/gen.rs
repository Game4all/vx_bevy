use building_blocks::{
    core::{ConstZero, ExtentN, Point, Point3i, PointN},
    storage::{Array3x1, FillExtent, GetMut},
};

use crate::{utils::ValueMap2D, voxel::Voxel, world::CHUNK_LENGTH};

const MAX_TERRAIN_HEIGHT: i32 = 256;

pub fn gen_terrain_for_chunk(chunk_pos: Point3i, data: &mut Array3x1<Voxel>) {
    let heightmap = ValueMap2D::new(
        CHUNK_LENGTH,
        CHUNK_LENGTH,
        simdnoise::NoiseBuilder::fbm_2d_offset(
            chunk_pos.x() as f32,
            CHUNK_LENGTH as usize,
            chunk_pos.z() as f32,
            CHUNK_LENGTH as usize,
        )
        .with_octaves(5)
        .generate()
        .0,
    );

    let base_height = chunk_pos.y();

    if base_height < 0 {
        data.fill_extent(
            &ExtentN::from_min_and_shape(PointN::ZERO, PointN::fill(CHUNK_LENGTH)),
            Voxel::Solid {
                attributes: [236, 230, 214, 255],
            },
        );
        return;
    }

    //todo: fix thiss
    // gen water only for first vertical chunk.
    if base_height == 0 {
        data.fill_extent(
            &ExtentN::from_min_and_max(PointN([0, 1, 0]), PointN([CHUNK_LENGTH, 8, CHUNK_LENGTH])),
            Voxel::Fluid {
                attributes: [102, 133, 254, 255],
            },
        );
    }

    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_LENGTH {
            let block_height = (heightmap.value_at(x, z).abs() * MAX_TERRAIN_HEIGHT as f32) as i32;

            let local_height = (block_height - base_height).max(0).min(CHUNK_LENGTH);

            for y in 0..local_height {
                *data.get_mut(PointN([x, y, z])) = Voxel::Solid {
                    attributes: get_color_for_height(base_height + y),
                }
            }
        }
    }
}

#[inline]
fn get_color_for_height(height: i32) -> [u8; 4] {
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
