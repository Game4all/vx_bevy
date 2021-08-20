use bevy::{
    diagnostic::Diagnostics,
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
    utils::Instant,
};
use building_blocks::{
    core::Extent3i,
    mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG},
    storage::Get,
};

use crate::utils::ChunkMeshBuilder;

use super::{
    chunk_extent, ChunkInfo, ChunkLoadState, ChunkMapReader, ChunkMeshInfo, ChunkMeshingRequest,
    ChunkUpdateEvent, WorldTaskPool, CHUNK_MESHING_TIME,
};

#[inline]
fn padded_chunk_extent() -> Extent3i {
    chunk_extent().padded(1)
}

pub(crate) fn mesh_chunks(
    mut chunks: Query<(&ChunkInfo, &ChunkMeshInfo, &mut ChunkLoadState)>,
    mut meshing_requests: EventReader<ChunkMeshingRequest>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_map: ChunkMapReader,
    task_pool: Res<WorldTaskPool>,
    mut diagnostics: ResMut<Diagnostics>,
) {
    let before_meshing_time = Instant::now();

    let mesh_results = task_pool.scope(|scope| {
        for meshing_event in meshing_requests.iter() {
            match chunks.get_component::<ChunkInfo>(meshing_event.0) {
                Ok(chunk_info) => {
                    if let Some(chunk_data) = chunk_map.get_chunk_data(&chunk_info.pos) {
                        scope.spawn(async move {
                            let mut greedy_buffer = GreedyQuadsBuffer::new(
                                padded_chunk_extent(),
                                RIGHT_HANDED_Y_UP_CONFIG.quad_groups(),
                            );
                            let extent = padded_chunk_extent();

                            greedy_buffer.reset(extent);
                            greedy_quads(chunk_data, &extent, &mut greedy_buffer);

                            if greedy_buffer.num_quads() != 0 {
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

                                Some((meshing_event.0, terrain_mesh, fluid_mesh))
                            } else {
                                None
                            }
                        });
                    }
                }
                Err(err) => warn!(
                    "Mesh data generation failed for chunk entity {:?}: {:?}",
                    meshing_event.0, err
                ),
            }
        }
    });

    for meshing_result in mesh_results {
        if let Some((chunk, terrain_mesh, fluid_mesh)) = meshing_result {
            if let Ok((___, mesh_info, mut load_state)) = chunks.get_mut(chunk) {
                *meshes.get_mut(&mesh_info.chunk_mesh).unwrap() = terrain_mesh;
                *meshes.get_mut(&mesh_info.fluid_mesh).unwrap() = fluid_mesh;
                if *load_state < ChunkLoadState::Idle {
                    *load_state = ChunkLoadState::Idle;
                }
            }
        }
    }

    let after_chunk_meshing = Instant::now() - before_meshing_time;
    diagnostics.add_measurement(CHUNK_MESHING_TIME, after_chunk_meshing.as_secs_f64());
}

pub(crate) fn queue_chunk_meshing(
    mut meshing_events: EventWriter<ChunkMeshingRequest>,
    query: Query<(Entity, &ChunkLoadState), Changed<ChunkLoadState>>,
) {
    for (entity, load_state) in query.iter() {
        if matches!(load_state, &ChunkLoadState::Loading) {
            meshing_events.send(ChunkMeshingRequest(entity));
        }
    }
}

pub(crate) fn handle_chunk_update_events(
    mut update_events: EventReader<ChunkUpdateEvent>,
    mut meshing_events: EventWriter<ChunkMeshingRequest>,
) {
    meshing_events.send_batch(update_events.iter().map(|x| ChunkMeshingRequest(x.0)));
}
