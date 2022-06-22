use float_ord::FloatOrd;
use std::{collections::BTreeMap, sync::RwLock};

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::Plugin,
};
use once_cell::sync::Lazy;

use super::{
    materials::{Grass, Sand, Water},
    storage::VoxelBuffer,
    ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH,
};

mod generators;
pub mod noise;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(|| Default::default());

pub trait BiomeTerrainGenerator: 'static + Sync + Send {
    fn generate_terrain(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>);

    //fixme: rename this as it is misleading (this won't use temperature or humidity stats for biome placement but I haven't been able to think of a better name for now)
    fn biome_temp_humidity(&self) -> FloatOrd<f32>;
}

#[derive(Default)]
pub struct TerrainGenerator {
    biomes_map: BTreeMap<FloatOrd<f32>, Box<dyn BiomeTerrainGenerator>>,

    // fixme: this is temporary for visualizing the noise.
    pub(crate) voxel_maps: BTreeMap<FloatOrd<f32>, Voxel>,
}

impl TerrainGenerator {
    pub fn register_biome(&mut self, biome: Box<dyn BiomeTerrainGenerator>) {
        self.biomes_map.insert(biome.biome_temp_humidity(), biome);
    }

    /// returns the biome with the closest temp / humidity
    // fn biome_at(&self, chunk_key: ChunkKey) -> &Box<dyn BiomeTerrainGenerator> {
    //     const BIOME_INVSCALE: f32 = 0.0024;

    //     let coords =
    //         noise::voronoi(chunk_key.location().xzy().truncate().as_vec2() * BIOME_INVSCALE);
    //     let p = FloatOrd(noise::rand2to1i(coords));

    //     self.biomes_map
    //         .range(..=p)
    //         .last()
    //         .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    // }

    pub fn generate(&self, chunk_key: ChunkKey, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        const BIOME_INVSCALE: f32 = 0.001;

        if chunk_key.location().y != 0 {
            return;
        }

        let v = noise::voronoi(chunk_key.location().xzy().truncate().as_vec2() * BIOME_INVSCALE);

        let p = FloatOrd(noise::rand2to1i(v));

        let voxel = self
            .voxel_maps
            .range(..=p)
            .last()
            .map_or(Voxel(Grass::ID), |x| *x.1);

        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                *buffer.voxel_at_mut([x, 0, z].into()) = voxel;
            }
        }
    }
}
pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {
        TERRAIN_GENERATOR
            .write()
            .expect("Failed to acquire terrain generator singleton.")
            .register_biome(Box::new(generators::DefaultTerrainGenerator));

        // this is hacked in to visualize the noise.
        {
            let mut reg = &mut TERRAIN_GENERATOR.write().expect("D").voxel_maps;
            reg.insert(FloatOrd(1.419f32), Voxel(Water::ID));
            reg.insert(FloatOrd(0.8), Voxel(Sand::ID));
        }
    }
}
