use bevy::math::{IVec2, IVec3, Vec3};

mod world;
pub use world::*;

mod worldgen;

pub const CHUNK_HEIGHT: i32 = 128;
pub const CHUNK_WIDTH: i32 = 16;
pub const CHUNK_DEPTH: i32 = 16;

pub const DEFAULT_VIEW_DISTANCE: i32 = 24;

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
