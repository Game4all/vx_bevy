#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Voxel(u8);

pub const EMPTY_VOXEL: Voxel = Voxel(0);

impl Default for Voxel {
    fn default() -> Self {
        EMPTY_VOXEL
    }
}
