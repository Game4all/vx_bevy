use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, VertexAttributeValues},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::Shape;

use crate::voxel::storage::VoxelBuffer;

// Processes the voxel data buffer specified as a parameter and generate.
//todo: don't populate mesh directly, introduce a meshbuilding system.
pub fn mesh_buffer<T, S>(
    buffer: &VoxelBuffer<T, S>,
    mesh_buffers: &mut GreedyQuadsBuffer,
    render_mesh: &mut Mesh,
    scale: f32,
) where
    T: Copy + Default + Voxel + MergeVoxel,
    S: Shape<u32, 3>,
{
    mesh_buffers.reset(buffer.shape().size() as usize);

    greedy_quads(
        buffer.slice(),
        buffer.shape(),
        [0; 3],
        buffer.shape().as_array().map(|x| x - 1),
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        mesh_buffers,
    );

    let num_indices = mesh_buffers.quads.num_quads() * 6;
    let num_vertices = mesh_buffers.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);

    for (group, face) in mesh_buffers
        .quads
        .groups
        .as_ref()
        .into_iter()
        .zip(RIGHT_HANDED_Y_UP_CONFIG.faces.into_iter())
    {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, scale));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    render_mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    render_mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.set_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));
}
