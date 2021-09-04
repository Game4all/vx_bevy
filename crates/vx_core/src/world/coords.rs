use bevy::math::{IVec2, IVec3, Vec3};
use building_blocks::core::{Point3i, PointN};

use super::CHUNK_LENGTH;

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunk(position: Vec3) -> IVec3 {
    IVec3::new(
        position.x.floor() as i32 / CHUNK_LENGTH,
        position.y.floor() as i32 / CHUNK_LENGTH,
        position.z.floor() as i32 / CHUNK_LENGTH,
    )
}

/// Transform a point in global space to a point in chunk space.
pub fn global2locali(pos: IVec3) -> IVec3 {
    IVec3::new(pos.x % CHUNK_LENGTH, pos.y, pos.z % CHUNK_LENGTH)
}

/// Gets the corresponding chunks coordinates from a point in global space.
pub fn global2chunki(position: IVec3) -> IVec2 {
    IVec2::new(position.x / CHUNK_LENGTH, position.z / CHUNK_LENGTH)
}

/// Gets the origin of a chunk in global space from its chunk coordinates.
pub fn chunk2global(chunk_coords: IVec3) -> Vec3 {
    Vec3::new(
        (chunk_coords.x * CHUNK_LENGTH) as f32,
        (chunk_coords.y * CHUNK_LENGTH) as f32,
        (chunk_coords.z * CHUNK_LENGTH) as f32,
    )
}

/// Returns a [`Point3i`] pointing to a chunk origin from its chunk coordinates.
pub fn chunk2point(chunk_coords: IVec3) -> Point3i {
    PointN([
        chunk_coords.x * CHUNK_LENGTH,
        chunk_coords.y * CHUNK_LENGTH,
        chunk_coords.z * CHUNK_LENGTH,
    ])
}
