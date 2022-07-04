use ilattice::{glam::UVec3, prelude::Extent};

use crate::voxel::{
    terraingen::{noise::Heightmap, BiomeTerrainGenerator},
    ChunkKey, Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

pub struct HeightmapBiomeTerrainGenerator {
    pub voxel: Voxel,
    pub biome_ord: f32,
}

impl HeightmapBiomeTerrainGenerator {
    const DEFAULT_TERRAIN_HEIGHT: i32 = 64;

    pub const fn new(voxel: Voxel, biome_ord: f32) -> Self {
        Self { voxel, biome_ord }
    }

    #[inline]
    fn heightmap_scale_func(x: f32, chunk_key: ChunkKey) -> u32 {
        ((Self::DEFAULT_TERRAIN_HEIGHT as i32 + ((x * 3.0).round() as i32))
            - chunk_key.location().y as i32)
            .max(0)
            .min((CHUNK_LENGTH) as i32) as u32
    }
}

impl BiomeTerrainGenerator for HeightmapBiomeTerrainGenerator {
    fn generate_terrain(
        &self,
        chunk_key: crate::voxel::ChunkKey,
        heightmap: Heightmap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut crate::voxel::storage::VoxelBuffer<Voxel, crate::voxel::ChunkShape>,
    ) {
        Extent::from_min_and_shape(UVec3::ZERO, UVec3::new(CHUNK_LENGTH, 1, CHUNK_LENGTH))
            .iter3()
            .for_each(|vec| {
                let height = heightmap.map([vec.x as u32, vec.z as u32], |x| {
                    Self::heightmap_scale_func(x, chunk_key)
                });

                for h in 0..height {
                    *buffer.voxel_at_mut([vec.x, h, vec.z].into()) = self.voxel;
                }
            });
    }

    fn biome_temp_humidity(&self) -> float_ord::FloatOrd<f32> {
        float_ord::FloatOrd(self.biome_ord)
    }
}
