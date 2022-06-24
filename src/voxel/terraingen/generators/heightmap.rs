use crate::voxel::{terraingen::BiomeTerrainGenerator, Voxel, CHUNK_LENGTH};

pub struct HeightmapBiomeTerrainGenerator {
    pub voxel: Voxel,
    pub biome_ord: f32,
}

impl HeightmapBiomeTerrainGenerator {
    const DEFAULT_TERRAIN_HEIGHT: i32 = 64;

    pub const fn new(voxel: Voxel, biome_ord: f32) -> Self {
        Self { voxel, biome_ord }
    }
}

impl BiomeTerrainGenerator for HeightmapBiomeTerrainGenerator {
    fn generate_terrain(
        &self,
        chunk_key: crate::voxel::ChunkKey,
        buffer: &mut crate::voxel::storage::VoxelBuffer<Voxel, crate::voxel::ChunkShape>,
    ) {
        let heightmap: Vec<u32> = simdnoise::NoiseBuilder::fbm_2d_offset(
            chunk_key.location().x as f32,
            CHUNK_LENGTH as usize,
            chunk_key.location().z as f32,
            CHUNK_LENGTH as usize,
        )
        .with_octaves(4)
        .generate()
        .0
        .iter()
        .map(|x| Self::DEFAULT_TERRAIN_HEIGHT as i32 + ((x * 1.0).round() as i32)) //todo: add a default 128 default height
        .map(|x| x - chunk_key.location().y)
        .map(|x| x.max(0).min((CHUNK_LENGTH) as i32))
        .map(|x| x as u32)
        .collect();

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                for h in 0..heightmap[(z * CHUNK_LENGTH + x) as usize] {
                    *buffer.voxel_at_mut([x, h, z].into()) = self.voxel;
                }
            }
        }
    }

    fn biome_temp_humidity(&self) -> float_ord::FloatOrd<f32> {
        float_ord::FloatOrd(self.biome_ord)
    }
}
