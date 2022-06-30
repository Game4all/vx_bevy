use crate::voxel::{materials::Rock, storage::VoxelBuffer, ChunkShape, Voxel, CHUNK_LENGTH};

/// Generate the world bottom border for a chunk.
pub fn terrain_generate_world_bottom_border(buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_LENGTH {
            *buffer.voxel_at_mut([x, 0, z].into()) = Voxel(Rock::ID);
        }
    }
}
