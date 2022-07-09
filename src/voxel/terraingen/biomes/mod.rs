use crate::voxel::{storage::VoxelBuffer, ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH_U};

use super::noise::Heightmap;

/// A trait representing a terrain generator for a biome.
pub trait BiomeTerrainGenerator: 'static + Sync + Send {
    /// Carve the terrain using the materials for the biome.
    fn carve_terrain(
        &self,
        chunk_key: ChunkKey,
        heightmap: Heightmap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    );
}

/// Utility trait for boxing biome generators.
pub trait IntoBoxedTerrainGenerator: BiomeTerrainGenerator + Sized {
    fn into_boxed_generator(self) -> Box<Self>;
}

impl<T: BiomeTerrainGenerator> IntoBoxedTerrainGenerator for T {
    fn into_boxed_generator(self) -> Box<Self> {
        Box::new(self)
    }
}
