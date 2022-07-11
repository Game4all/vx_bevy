use ilattice::{glam::UVec2, glam::UVec3, prelude::Extent};

use crate::voxel::{
    material::VoxelMaterial,
    materials::{Bedrock, Rock, Water},
    storage::VoxelBuffer,
    ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

use super::noise::Heightmap;

/// Generate the world bottom border for a chunk.
pub fn terrain_generate_world_bottom_border(buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
    buffer.fill_extent(
        Extent::from_min_and_shape(UVec3::ZERO, UVec3::new(CHUNK_LENGTH, 2, CHUNK_LENGTH)),
        Bedrock::into_voxel(),
    )
}

/// Carve the general terrain shape for a chunk.
pub fn terrain_carve_heightmap(
    buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    key: ChunkKey,
    heighmap: &Heightmap<f32, CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
) {

    // drown the terrain under sea level.
    if key.location().y <= 96 {
        buffer.fill_extent(
            Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH)),
            Water::into_voxel(),
        );
    }

    // carve the terrain.
    Extent::from_min_and_shape(UVec2::ZERO, UVec2::new(CHUNK_LENGTH, CHUNK_LENGTH))
        .iter2()
        .for_each(|pos| {
            let local_height = ((heighmap.get(pos.into()).round() - key.location().y as f32) as u32)
                .min(CHUNK_LENGTH)
                .max(0);

            for h in 0..local_height {
                *buffer.voxel_at_mut([pos.x, h, pos.y].into()) = Rock::into_voxel();
            }
        });
}
