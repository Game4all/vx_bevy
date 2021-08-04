use bevy::math::IVec3;
use building_blocks::storage::Array3x1;

use crate::voxel::Voxel;

mod noise;
pub use noise::*;

pub trait TerrainGenerator {
    fn generate(&self, chunk_pos: IVec3, data: &mut Array3x1<Voxel>);

    fn set_seed(&mut self, seed: i32);
}
