use building_blocks::{
    mesh::{IsOpaque, MergeVoxel},
    storage::IsEmpty,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Solid { attributes: [u8; 4] },
    Fluid { attributes: [u8; 4] },
    Empty,
}

impl Default for Voxel {
    fn default() -> Self {
        Self::Empty
    }
}

impl MergeVoxel for Voxel {
    type VoxelValue = Voxel;

    fn voxel_merge_value(&self) -> Self::VoxelValue {
        *self
    }
}

impl IsOpaque for Voxel {
    fn is_opaque(&self) -> bool {
        matches!(self, &Voxel::Solid { .. })
    }
}

impl IsEmpty for Voxel {
    fn is_empty(&self) -> bool {
        matches!(self, &Voxel::Empty)
    }
}
