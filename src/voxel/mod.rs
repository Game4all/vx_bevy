///! Storage primitives for storing voxel data
mod storage;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Voxel(u8);

pub const EMPTY_VOXEL: Voxel = Voxel(0);
