use float_ord::FloatOrd;
use std::sync::RwLock;

use bevy::prelude::Plugin;
use once_cell::sync::Lazy;

use self::{
    common::terrain_generate_world_bottom_border,
    noise::{generate_heightmap_data, Heightmap},
};

use super::{storage::VoxelBuffer, ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH_U};

mod generators;

/// noise functions ported over from C / GLSL code
pub mod noise;

/// common functions used by all terrain generators
pub mod common;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(|| Default::default());

/// A trait representing a terrain generator for a biome.
pub trait BiomeTerrainGenerator: 'static + Sync + Send {
    /// Carve the terrain using the materials for the biome.
    fn carve_terrain(
        &self,
        chunk_key: ChunkKey,
        heightmap: Heightmap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    );

    //fixme: rename this as it is misleading (this won't use temperature or humidity stats for biome placement but I haven't been able to think of a better name for now)
    fn biome_temp_humidity(&self) -> FloatOrd<f32>;
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

#[derive(Default)]
pub struct TerrainGenerator {
    // biomes_map: BTreeMap<FloatOrd<f32>, Box<dyn BiomeTerrainGenerator>>,
}

impl TerrainGenerator {
    // pub fn register_biome(&mut self, biome: Box<dyn BiomeTerrainGenerator>) -> &mut Self {
    //     self.biomes_map.insert(biome.biome_temp_humidity(), biome);
    //     self
    // }

    // returns the biome with the closest temp / humidity
    // fn biome_at(&self, chunk_key: ChunkKey) -> &Box<dyn BiomeTerrainGenerator> {
    //     const BIOME_INVSCALE: f32 = 0.001;

    //     let coords =
    //         noise::voronoi(chunk_key.location().xzy().truncate().as_vec2() * BIOME_INVSCALE);
    //     let p = FloatOrd(noise::rand2to1i(coords));

    //     self.biomes_map
    //         .range(..=p)
    //         .last()
    //         .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    // }

    pub fn generate(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        // let biome = self.biome_at(chunk_key);
        let noise = generate_heightmap_data(chunk_key, CHUNK_LENGTH_U);

        let noise_map = Heightmap::<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&noise);

        common::terrain_carve_heightmap(buffer, chunk_key, &noise_map);

        // biome.generate_terrain(chunk_key, noise_map, buffer);

        if chunk_key.location().y == 0 {
            terrain_generate_world_bottom_border(buffer);
        }
    }
}

pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {}
}
