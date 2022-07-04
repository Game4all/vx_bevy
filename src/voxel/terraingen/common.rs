use ilattice::{glam::UVec3, prelude::Extent};

use crate::voxel::{
    material::VoxelMaterial, materials::Bedrock, storage::VoxelBuffer, ChunkShape, Voxel,
    CHUNK_LENGTH,
};

/// Generate the world bottom border for a chunk.
pub fn terrain_generate_world_bottom_border(buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
    buffer.fill_extent(
        Extent::from_min_and_shape(UVec3::ZERO, UVec3::new(CHUNK_LENGTH, 2, CHUNK_LENGTH)),
        Bedrock::into_voxel(),
    )
}
