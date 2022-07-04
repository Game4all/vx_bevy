use crate::voxel::{
    terraingen::{noise::NoiseMap, BiomeTerrainGenerator},
    Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

pub struct FlatBiomeTerrainGenerator {
    voxel: Voxel,
    biome_range: f32,
}

impl FlatBiomeTerrainGenerator {
    pub const fn new(voxel: Voxel, biome_range: f32) -> Self {
        FlatBiomeTerrainGenerator { voxel, biome_range }
    }
}

impl BiomeTerrainGenerator for FlatBiomeTerrainGenerator {
    fn generate_terrain(
        &self,
        chunk_key: crate::voxel::ChunkKey,
        _heightmap: NoiseMap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut crate::voxel::storage::VoxelBuffer<
            crate::voxel::Voxel,
            crate::voxel::ChunkShape,
        >,
    ) {
        if chunk_key.location().y != 0 {
            return;
        }

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                *buffer.voxel_at_mut([x, 2, z].into()) = self.voxel;
            }
        }
    }

    fn biome_temp_humidity(&self) -> float_ord::FloatOrd<f32> {
        float_ord::FloatOrd(self.biome_range)
    }
}
