use crate::voxel::{
    materials::{Dirt, Grass, Rock, Sand, Snow, Water},
    storage::VoxelBuffer,
    terraingen::{noise::NoiseMap, BiomeTerrainGenerator},
    ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

pub struct DefaultTerrainGenerator;

impl DefaultTerrainGenerator {
    const DEFAULT_TERRAIN_HEIGHT: i32 = 64;

    #[inline]
    fn heightmap_scale_func(x: f32, chunk_key: ChunkKey) -> u32 {
        ((Self::DEFAULT_TERRAIN_HEIGHT as i32 + ((x * 4.0).round() as i32))
            - chunk_key.location().y as i32)
            .max(0)
            .min((CHUNK_LENGTH) as i32) as u32
    }
}

impl BiomeTerrainGenerator for DefaultTerrainGenerator {
    fn generate_terrain(
        &self,
        key: ChunkKey,
        heightmap: NoiseMap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        if key.location().y == 0 {
            for x in 0..CHUNK_LENGTH {
                for z in 0..CHUNK_LENGTH {
                    for y in 1..16 {
                        *buffer.voxel_at_mut([x, y, z].into()) = Voxel(Water::ID);
                    }
                }
            }
        }

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                let h = heightmap.map(x as usize, z as usize, |x| {
                    Self::heightmap_scale_func(x, key)
                });

                for h in 0..h {
                    *buffer.voxel_at_mut([x, h, z].into()) =
                        get_mat_by_height(key.location().y as u32 + h);
                }
            }
        }

        if key.location().y == 0 {
            for x in 0..CHUNK_LENGTH {
                for z in 0..CHUNK_LENGTH {
                    *buffer.voxel_at_mut([x, 0, z].into()) = Voxel(Rock::ID);
                }
            }
        }
    }

    fn biome_temp_humidity(&self) -> float_ord::FloatOrd<f32> {
        float_ord::FloatOrd(0.0)
    }
}

#[inline]
fn get_mat_by_height(h: u32) -> Voxel {
    match h {
        0..=29 => Voxel(Sand::ID),
        188..=192 => Voxel(Dirt::ID),
        193..=224 => Voxel(Rock::ID),
        225..=384 => Voxel(Snow::ID),
        _ => Voxel(Grass::ID),
    }
}
