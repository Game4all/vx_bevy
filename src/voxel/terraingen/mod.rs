use float_ord::FloatOrd;
use std::{collections::BTreeMap, sync::RwLock};

use bevy::{math::Vec3Swizzles, prelude::Plugin};
use once_cell::sync::Lazy;

use self::generators::{FlatBiomeTerrainGenerator, HeightmapBiomeTerrainGenerator};

use super::{
    materials::{Grass, Sand, Water},
    storage::VoxelBuffer,
    ChunkKey, ChunkShape, Voxel,
};

mod generators;
pub mod noise;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(|| Default::default());

/// A trait representing terrain generation for a specific biome type.
pub trait BiomeTerrainGenerator: 'static + Sync + Send {
    /// Generate the general terrain shape for a chunk.
    fn generate_terrain(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>);

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
    biomes_map: BTreeMap<FloatOrd<f32>, Box<dyn BiomeTerrainGenerator>>,
}

impl TerrainGenerator {
    pub fn register_biome(&mut self, biome: Box<dyn BiomeTerrainGenerator>) -> &mut Self {
        self.biomes_map.insert(biome.biome_temp_humidity(), biome);
        self
    }

    // returns the biome with the closest temp / humidity
    fn biome_at(&self, chunk_key: ChunkKey) -> &Box<dyn BiomeTerrainGenerator> {
        const BIOME_INVSCALE: f32 = 0.001;

        let coords =
            noise::voronoi(chunk_key.location().xzy().truncate().as_vec2() * BIOME_INVSCALE);
        let p = FloatOrd(noise::rand2to1i(coords));

        self.biomes_map
            .range(..=p)
            .last()
            .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    }

    pub fn generate(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        let biome = self.biome_at(chunk_key);
        biome.generate_terrain(chunk_key, buffer);
    }
}

pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {
        TERRAIN_GENERATOR
            .write()
            .expect("Failed to acquire terrain generator singleton.")
            // .register_biome(generators::DefaultTerrainGenerator.into_boxed_generator())
            .register_biome(
                HeightmapBiomeTerrainGenerator::new(Voxel(Grass::ID), 0.0f32)
                    .into_boxed_generator(),
            )
            .register_biome(
                FlatBiomeTerrainGenerator::new(Voxel(Water::ID), 1.419f32).into_boxed_generator(),
            )
            .register_biome(
                HeightmapBiomeTerrainGenerator::new(Voxel(Sand::ID), 0.8f32).into_boxed_generator(),
            );
    }
}
