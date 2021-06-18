use bevy::math::{IVec2, IVec3, Vec3};
use building_blocks::core::{Point3i, PointN};

use super::{CHUNK_DEPTH, CHUNK_WIDTH};

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunk(position: Vec3) -> IVec2 {
    IVec2::new(
        position.x.floor() as i32 / CHUNK_WIDTH,
        position.z.floor() as i32 / CHUNK_DEPTH,
    )
}

/// Transform a point in global space to a point in chunk space.
pub fn global2locali(pos: IVec3) -> IVec3 {
    IVec3::new(pos.x % CHUNK_WIDTH, pos.y, pos.z % CHUNK_DEPTH)
}

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunki(position: IVec3) -> IVec2 {
    IVec2::new(position.x / CHUNK_WIDTH, position.z / CHUNK_DEPTH)
}

/// Gets the origin of a chunk in global space from its chunk coordinates.
pub fn chunk2global(chunk_coords: IVec2) -> Vec3 {
    Vec3::new(
        (chunk_coords.x * CHUNK_WIDTH) as f32,
        0.,
        (chunk_coords.y * CHUNK_DEPTH) as f32,
    )
}

/// Returns a [`Point3i`] pointing to a chunk origin from its chunk coordinates.
pub fn chunk2point(chunk_coords: IVec2) -> Point3i {
    PointN([
        chunk_coords.x * CHUNK_WIDTH,
        0,
        chunk_coords.y * CHUNK_DEPTH,
    ])
}
