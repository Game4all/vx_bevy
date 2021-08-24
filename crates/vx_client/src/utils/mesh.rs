use building_blocks::{
    core::Axis3,
    mesh::{OrientedCubeFace, UnorientedQuad},
};

use vx_core::voxel::Voxel;

const VOXEL_MESH_SIZE: f32 = 1.0;

#[derive(Default)]
pub struct ChunkMeshBuilder {
    //normal mesh data
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,

    //fluid mesh data
    pub fluid_positions: Vec<[f32; 3]>,
    pub fluid_normals: Vec<[f32; 3]>,
    pub fluid_indices: Vec<u32>,
    pub fluid_uv: Vec<[f32; 2]>,
    pub fluid_colors: Vec<[u8; 4]>,
}

impl ChunkMeshBuilder {
    pub fn add_quad_to_mesh(
        &mut self,
        face: &OrientedCubeFace,
        quad: &UnorientedQuad,
        voxel: &Voxel,
    ) {
        match voxel {
            &Voxel::Fluid { attributes } => {
                let start_index = self.fluid_positions.len() as u32;
                self.fluid_positions
                    .extend_from_slice(&face.quad_mesh_positions(quad, VOXEL_MESH_SIZE));

                self.fluid_normals
                    .extend_from_slice(&face.quad_mesh_normals());

                self.fluid_uv
                    .extend_from_slice(&face.tex_coords(Axis3::X, false, quad));

                self.fluid_colors.extend_from_slice(&[attributes; 4]);

                self.fluid_indices
                    .extend_from_slice(&face.quad_mesh_indices(start_index));
            }
            &Voxel::Solid { attributes } => {
                let start_index = self.positions.len() as u32;
                self.positions
                    .extend_from_slice(&face.quad_mesh_positions(quad, VOXEL_MESH_SIZE));

                self.normals.extend_from_slice(&face.quad_mesh_normals());

                self.uv
                    .extend_from_slice(&face.tex_coords(Axis3::X, false, quad));

                self.colors.extend_from_slice(&[attributes; 4]);

                self.indices
                    .extend_from_slice(&face.quad_mesh_indices(start_index));
            }
            &Voxel::Empty => unreachable!(),
        };
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.indices.clear();
        self.uv.clear();
        self.colors.clear();
    }
}
