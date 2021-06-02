use building_blocks::mesh::{OrientedCubeFace, UnorientedQuad};

use crate::voxel::Voxel;

#[derive(Default)]
pub struct ChunkMeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
}

impl ChunkMeshBuilder {
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

    pub fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.indices.clear();
        self.uv.clear();
        self.colors.clear();
    }
}
