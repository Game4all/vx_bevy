use block_mesh::{MergeVoxel, Voxel as MeshableVoxel};

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Voxel(pub u8);

pub const EMPTY_VOXEL: Voxel = Voxel(0);

impl Default for Voxel {
    fn default() -> Self {
        EMPTY_VOXEL
    }
}

impl MeshableVoxel for Voxel {
    #[inline]
    fn get_visibility(&self) -> block_mesh::VoxelVisibility {
        match self.0 {
            0 => block_mesh::VoxelVisibility::Empty,
            _ => block_mesh::VoxelVisibility::Opaque,
        }
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = u8;

    #[inline]
    fn merge_value(&self) -> Self::MergeValue {
        self.0
    }
}

pub trait MaterialVoxel: MergeVoxel + MeshableVoxel {
    fn as_mat_id(&self) -> u8;
}

impl MaterialVoxel for Voxel {
    fn as_mat_id(&self) -> u8 {
        self.0
    }
}
