use crate::voxel::{
    materials::{Dirt, Grass, Rock, Sand, Snow, Water},
    storage::VoxelBuffer,
    terraingen::BiomeTerrainGenerator,
    ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH,
};

pub struct DefaultTerrainGenerator;

impl BiomeTerrainGenerator for DefaultTerrainGenerator {
    fn generate_terrain(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        generate_terrain(chunk_key, buffer);
    }
}

const DEFAULT_TERRAIN_HEIGHT: u32 = 128; // equals to 4 vertical chunks

pub fn generate_terrain(key: ChunkKey, data: &mut VoxelBuffer<Voxel, ChunkShape>) {
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
    .map(|x| DEFAULT_TERRAIN_HEIGHT as i32 + ((x * 6.0).round() as i32)) //todo: add a default 128 default height
    .map(|x| x - key.location().y)
    .map(|x| x.max(0).min((CHUNK_LENGTH) as i32))
    .map(|x| x as u32)
    .collect();

    if key.location().y == 0 {
        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                for y in 1..16 {
                    *data.voxel_at_mut([x, y, z].into()) = Voxel(Water::ID);
                }
            }
        }
    }

    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_LENGTH {
            for h in 0..heightmap[(z * CHUNK_LENGTH + x) as usize] {
                *data.voxel_at_mut([x, h, z].into()) =
                    get_mat_by_height(key.location().y as u32 + h);
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
