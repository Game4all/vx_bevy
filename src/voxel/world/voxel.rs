use block_mesh::Voxel as MeshableVoxel;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Voxel(u8);

pub const EMPTY_VOXEL: Voxel = Voxel(0);

impl Default for Voxel {
    fn default() -> Self {
        EMPTY_VOXEL
    }
}

impl MeshableVoxel for Voxel {
    #[inline]
    fn is_empty(&self) -> bool {
        self.0 == EMPTY_VOXEL.0
    }

    #[inline]
    fn is_opaque(&self) -> bool {
        true
    }
}
