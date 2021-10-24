use building_blocks::{
    core::{Extent3i, Point3i},
    storage::Array3x1,
};

use crate::voxel::Voxel;

// Base trait for terrain features such as vegetation, structures, etc ...
pub trait TerrainFeature: Send + Sync {
    fn extent(&self) -> &Extent3i;

    fn apply(&self, chunk_min: Point3i, chunk_data: &mut Array3x1<Voxel>, base: Point3i);
}
