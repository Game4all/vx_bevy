use building_blocks::{
    mesh::{IsOpaque, MergeVoxel, OrientedCubeFace, UnorientedQuad},
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

        let attribute = match voxel {
            &Voxel::Fluid { attributes } => attributes,
            &Voxel::Solid { attributes } => attributes,
            &Voxel::Empty => unreachable!(),
        };

        self.colors.extend_from_slice(&[attribute; 4]);

        self.indices
            .extend_from_slice(&face.quad_mesh_indices(start_index));
    }
}
