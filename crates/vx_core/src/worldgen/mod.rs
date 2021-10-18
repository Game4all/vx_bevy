mod gen;
use std::sync::Arc;

use building_blocks::{
    core::{Extent3i, Point3i},
    storage::Array3x1,
};
pub use gen::*;

use crate::voxel::Voxel;

pub type BoxedTerrainGenerator = Arc<Box<dyn TerrainGenerator>>;

// Base trait for world generator implementations
pub trait TerrainGenerator: Send + Sync {
    // Builds the base terrain on which the terrain features are lated applied.
    fn build_terrain_base(&self, chunk_min: Point3i, data: &mut Array3x1<Voxel>);
}

// Terrain features such as vegetation, structures, etc ...
pub trait TerrainFeature {
    fn extent(&self) -> &Extent3i;

    fn apply(chunk_min: Point3i, chunk_data: &mut Array3x1<Voxel>, base: Point3i);
}
