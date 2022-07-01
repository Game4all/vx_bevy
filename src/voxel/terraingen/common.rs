use ilattice::{glam::UVec3, prelude::Extent};

use crate::voxel::{materials::Rock, storage::VoxelBuffer, ChunkShape, Voxel, CHUNK_LENGTH};

/// Generate the world bottom border for a chunk.
pub fn terrain_generate_world_bottom_border(buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
    Extent::from_min_and_shape(UVec3::ZERO, UVec3::new(CHUNK_LENGTH, 1, CHUNK_LENGTH))
        .iter3()
        .for_each(|vec| {
            *buffer.voxel_at_mut(vec) = Voxel(Rock::ID);
        });
}
