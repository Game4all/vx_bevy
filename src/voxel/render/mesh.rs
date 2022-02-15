use std::marker::PhantomData;

use crate::voxel::storage::VoxelBuffer;
use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, VertexAttributeValues},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, RIGHT_HANDED_Y_UP_CONFIG};
use ndcopy::copy3;
use ndshape::{Shape, Shape3u32};

use super::VoxelMesh;

/// Intermediate buffers for greedy meshing of voxel data which are reusable between frames to not allocate.
pub struct MeshBuffers<T, S: Shape<u32, 3>>
where
    T: Copy + Default + Voxel + MergeVoxel,
{
    // A padded buffer to run greedy meshing algorithm on
    scratch_buffer: VoxelBuffer<T, Shape3u32>,
    greedy_buffer: GreedyQuadsBuffer,
    _phantom: PhantomData<S>,
}

impl<T, S: Shape<u32, 3>> MeshBuffers<T, S>
where
    T: Copy + Default + Voxel + MergeVoxel,
{
    pub fn new(shape: S) -> Self {
        let padded_shape = Shape3u32::new(shape.as_array().map(|x| x + 2));

        Self {
            greedy_buffer: GreedyQuadsBuffer::new(padded_shape.size() as usize),
            scratch_buffer: VoxelBuffer::<T, Shape3u32>::new_empty(padded_shape),
            _phantom: Default::default(),
        }
    }
}

// Processes the voxel data buffer specified as a parameter and generate.
//todo: don't populate mesh directly, introduce a meshbuilding system.
pub fn mesh_buffer<T, S>(
    buffer: &VoxelBuffer<T, S>,
    mesh_buffers: &mut MeshBuffers<T, S>,
    render_mesh: &mut Mesh,
    scale: f32,
) where
    T: Copy + Default + Voxel + MergeVoxel,
    S: Shape<u32, 3>,
{
    mesh_buffers
        .greedy_buffer
        .reset(buffer.shape().size() as usize);

    let dst_shape = mesh_buffers.scratch_buffer.shape().clone();

    copy3(
        buffer.shape().as_array(),
        buffer.slice(),
        buffer.shape(),
        [0; 3],
        mesh_buffers.scratch_buffer.slice_mut(),
        &dst_shape,
        [1; 3],
    );

    greedy_quads(
        mesh_buffers.scratch_buffer.slice(),
        mesh_buffers.scratch_buffer.shape(),
        [0; 3],
        mesh_buffers
            .scratch_buffer
            .shape()
            .as_array()
            .map(|axis| axis - 1),
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut mesh_buffers.greedy_buffer,
    );

    let num_indices = mesh_buffers.greedy_buffer.quads.num_quads() * 6;
    let num_vertices = mesh_buffers.greedy_buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);

    for (group, face) in mesh_buffers
        .greedy_buffer
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

    //todo: encode mesh normal and material index into this.
    render_mesh.set_attribute(
        VoxelMesh::ATTRIBUTE_DATA,
        VertexAttributeValues::Sint32(vec![0x001i32; num_vertices]),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));
}
