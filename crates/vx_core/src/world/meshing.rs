use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
    tasks::ComputeTaskPool,
};
use building_blocks::{
    core::Extent3i,
    mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG},
    storage::Get,
};
use std::ops::{Deref, DerefMut};

use crate::utils::ChunkMeshBuilder;

use super::{chunk_extent, ChunkInfo, ChunkLoadState, ChunkMap, ChunkMeshInfo};

pub(crate) struct ChunkMeshingRequest(Entity);

pub(crate) struct ReusableGreedyQuadsBuffer(GreedyQuadsBuffer);

impl Deref for ReusableGreedyQuadsBuffer {
    type Target = GreedyQuadsBuffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReusableGreedyQuadsBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromWorld for ReusableGreedyQuadsBuffer {
    fn from_world(_: &mut World) -> Self {
        Self(GreedyQuadsBuffer::new(
            padded_chunk_extent(),
            RIGHT_HANDED_Y_UP_CONFIG.quad_groups(),
        ))
    }
}

#[inline]
fn padded_chunk_extent() -> Extent3i {
    chunk_extent().padded(1)
}

pub(crate) fn mesh_chunks(
    mut chunks: Query<(&ChunkInfo, &ChunkMeshInfo, &mut ChunkLoadState)>,
    mut meshing_requests: EventReader<ChunkMeshingRequest>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_map: ResMut<ChunkMap>,
    task_pool: Res<ComputeTaskPool>,
) {
    let mesh_results = task_pool.scope(|scope| {
        for meshing_event in meshing_requests.iter() {
            if let Ok(chunk_info) = chunks.get_component::<ChunkInfo>(meshing_event.0) {
                if let Some(chunk_data) = chunk_map.chunks.get(&chunk_info.pos) {
                    scope.spawn(async move {
                        let mut greedy_buffer = GreedyQuadsBuffer::new(
                            padded_chunk_extent(),
                            RIGHT_HANDED_Y_UP_CONFIG.quad_groups(),
                        );
                        let extent = padded_chunk_extent();

                        greedy_buffer.reset(extent);
                        greedy_quads(chunk_data, &extent, &mut greedy_buffer);

                        let mut chunk_mesh_builder = ChunkMeshBuilder::default();

                        for group in greedy_buffer.quad_groups.iter() {
                            for quad in group.quads.iter() {
                                chunk_mesh_builder.add_quad_to_mesh(
                                    &group.face,
                                    quad,
                                    &chunk_data.get(quad.minimum),
                                );
                            }
                        }

                        let ChunkMeshBuilder {
                            positions,
                            normals,
                            indices,
                            colors,
                            uv,
                            fluid_positions,
                            fluid_normals,
                            fluid_indices,
                            fluid_colors,
                            fluid_uv,
                        } = chunk_mesh_builder;

                        let mut terrain_mesh = Mesh::new(PrimitiveTopology::TriangleList);

                        terrain_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                        terrain_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                        terrain_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                        terrain_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                        terrain_mesh.set_indices(Some(Indices::U32(indices)));

                        let mut fluid_mesh = Mesh::new(PrimitiveTopology::TriangleList);

                        fluid_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, fluid_positions);
                        fluid_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, fluid_normals);
                        fluid_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, fluid_uv);
                        fluid_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, fluid_colors);
                        fluid_mesh.set_indices(Some(Indices::U32(fluid_indices)));

                        (meshing_event.0, terrain_mesh, fluid_mesh)
                    });
                }
            }
        }
    });

    for (chunk, terrain_mesh, fluid_mesh) in mesh_results {
        if let Ok((___, mesh_info, mut load_state)) = chunks.get_mut(chunk) {
            *meshes.get_mut(&mesh_info.chunk_mesh).unwrap() = terrain_mesh;
            *meshes.get_mut(&mesh_info.fluid_mesh).unwrap() = fluid_mesh;
            if *load_state < ChunkLoadState::Idle {
                *load_state = ChunkLoadState::Idle;
            }
        }
    }
}

pub(crate) fn handle_chunk_loading_events(
    mut meshing_events: EventWriter<ChunkMeshingRequest>,
    query: Query<(Entity, &ChunkLoadState), Changed<ChunkLoadState>>,
) {
    for (entity, load_state) in query.iter() {
        if matches!(load_state, &ChunkLoadState::Loading) {
            meshing_events.send(ChunkMeshingRequest(entity));
        }
    }
}