use std::sync::RwLock;

use bevy::prelude::Plugin;
use once_cell::sync::Lazy;

use super::{storage::VoxelBuffer, ChunkKey, ChunkShape, Voxel};

mod generators;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(|| Default::default());

pub trait BiomeTerrainGenerator: 'static + Sync + Send {
    fn generate_terrain(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>);
}

#[derive(Default)]
pub struct TerrainGenerator {
    biomes: Vec<Box<dyn BiomeTerrainGenerator>>, // should use a btree map for faster lookup in the future when a biome map is generated.
}

impl TerrainGenerator {
    pub fn register_biome(&mut self, biome: Box<dyn BiomeTerrainGenerator>) {
        self.biomes.push(biome);
    }

    pub fn generate(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        self.biomes
            .first()
            .unwrap()
            .generate_terrain(chunk_key, buffer);
    }
}

pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {
        TERRAIN_GENERATOR
            .write()
            .expect("Failed to acquire terrain generator singleton.")
            .register_biome(Box::new(generators::DefaultTerrainGenerator));
    }
}
