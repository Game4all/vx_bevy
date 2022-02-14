use super::{storage::VoxelBuffer, ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH};

pub fn generate_terrain(key: ChunkKey, data: &mut VoxelBuffer<Voxel, ChunkShape>) {
    let heightmap: Vec<u32> = simdnoise::NoiseBuilder::fbm_2d_offset(
        key.location().x as f32,
        CHUNK_LENGTH as usize,
        key.location().z as f32,
        CHUNK_LENGTH as usize,
    )
    .with_octaves(5)
    .generate()
    .0
    .iter()
    .map(|x| (x * 256.0).round() as i32)
    .map(|x| x - key.location().y)
    .map(|x| x.max(0).min((CHUNK_LENGTH - 1) as i32))
    .map(|x| x as u32)
    .collect();

    for x in 0..CHUNK_LENGTH {
        for z in 0..CHUNK_LENGTH {
            for h in 0..heightmap[(z * CHUNK_LENGTH + x) as usize] {
                *data.voxel_at_mut([x, h, z].into()) = Voxel(1);
            }
        }
    }
}
