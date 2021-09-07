use building_blocks::{core::Point3i, storage::Array3x1};

use crate::voxel::Voxel;

mod noise;
pub use noise::*;

pub trait TerrainGenerator {
    fn generate(&self, chunk_pos: Point3i, data: &mut Array3x1<Voxel>);

    fn set_seed(&mut self, seed: i32);
}
