use building_blocks::{
    mesh::{IsOpaque, MergeVoxel, OrientedCubeFace, UnorientedQuad},
    storage::IsEmpty,
};

use crate::world::Voxel;

impl MergeVoxel for Voxel {
    type VoxelValue = u8;

    fn voxel_merge_value(&self) -> Self::VoxelValue {
        self.attributes[0]
    }
}

impl IsOpaque for Voxel {
    fn is_opaque(&self) -> bool {
        true
    }
}

impl IsEmpty for Voxel {
    fn is_empty(&self) -> bool {
        self.attributes[3] == 0
    }
}

#[derive(Default)]
pub struct ChunkMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
}

impl ChunkMesh {
    pub fn add_quad_to_mesh(
        &mut self,
        face: &OrientedCubeFace,
        quad: &UnorientedQuad,
        voxel: &Voxel,
    ) {
        let start_index = self.positions.len() as u32;

        self.positions
            .extend_from_slice(&face.quad_mesh_positions(quad));

        self.normals.extend_from_slice(&face.quad_mesh_normals());

        self.uv
            .extend_from_slice(&face.simple_tex_coords(false, quad));

        self.colors.extend_from_slice(&[voxel.attributes; 4]);

        self.indices
            .extend_from_slice(&face.quad_mesh_indices(start_index));
    }
}
