mod features;
mod gen;

use std::sync::Arc;

use bevy::prelude::*;
use building_blocks::{core::Point3i, storage::Array3x1};

pub use features::*;
pub use gen::*;

use crate::voxel::Voxel;

pub struct ChunkGenerator {
    pub terrain_generator: Box<dyn TerrainGenerator>,
    pub terrain_features: Vec<Box<dyn TerrainFeature>>,
}

impl ChunkGenerator {
    pub fn with_terrain_generator(terrain_generator: Box<dyn TerrainGenerator>) -> Self {
        ChunkGenerator {
            terrain_generator,
            terrain_features: Default::default(),
        }
    }

    pub fn register_terrain_feature(&mut self, feature: Box<dyn TerrainFeature>) {
        self.terrain_features.push(feature);
    }

    pub fn generate_chunk_terrain(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>) {
        if !self.terrain_generator.fill(chunk_min, data) {
            self.terrain_generator.shape_terrain(chunk_min, data);
        }
        self.terrain_generator.carve_terrain(chunk_min, data);
    }
}

/// Base trait for world generator implementations.
pub trait TerrainGenerator: Send + Sync {
    // The initial fill to do in the chunk.
    // Returns whether the terrain shaping should be skipped (ie: in case the chunk is full of air / stone).
    fn fill(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>) -> bool;

    // Builds the base terrain on which the terrain features are lated applied.
    fn shape_terrain(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>);

    // Carve the terrain.
    fn carve_terrain(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>);

    // fn apply_terrain_features(
    //     &self,
    //     chunk_min: Point3i,
    //     data: &mut Array3x1<Voxel>,
    //     features: &[Box<dyn TerrainFeature>],
    // );
}

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Arc::new(ChunkGenerator::with_terrain_generator(Box::new(
            NoiseTerrainGenerator,
        ))));
    }
}
