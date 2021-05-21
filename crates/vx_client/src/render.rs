use std::collections::VecDeque;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
        render_graph::base::MainPass,
        shader::ShaderStages,
    },
};
use building_blocks::{
    mesh::{greedy_quads, GreedyQuadsBuffer},
    prelude::*,
};

use vx_core::meshing::ChunkMesh;
use vx_core::world::{chunk_extent, Chunk, ChunkReadyEvent, DEFAULT_VIEW_DISTANCE};

struct ChunkMeshingEvent(Entity);

pub const TERRAIN_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 541458694767869);

#[derive(Bundle)]
pub struct ChunkRenderBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
}

/// Attach to the newly created chunk entities, the render components.
fn attach_chunk_render_bundle(
    chunks: Query<Entity, Added<Chunk>>,
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ent in chunks.iter() {
        commands.entity(ent).insert_bundle(ChunkRenderBundle {
            mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
            material: mats.add(Default::default()),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                TERRAIN_PIPELINE_HANDLE.typed(),
            )]),
            draw: Default::default(),
            main_pass: Default::default(),
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
        });
    }
}

//todo: run this asynchronously
//todo: limit concurrency
fn mesh_chunks_async(
    mut chunks: Query<(&Chunk, &mut Visible, &Handle<Mesh>)>,
    mut meshing_events: ResMut<VecDeque<ChunkMeshingEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in 0..(DEFAULT_VIEW_DISTANCE / 2) {
        if let Some(meshing_event) = meshing_events.pop_back() {
            if let Ok((chunk, mut visibility, mesh_handle)) = chunks.get_mut(meshing_event.0) {
                let mesh = meshes.get_mut(mesh_handle).unwrap();
                let extent = chunk_extent();

                let mut greedy_buffer = GreedyQuadsBuffer::new_with_y_up(extent.padded(1));
                greedy_quads(&chunk.block_data, &extent.padded(1), &mut greedy_buffer);

                let mut chunk_mesh = ChunkMesh::default();

                for group in greedy_buffer.quad_groups.iter() {
                    for quad in group.quads.iter() {
                        chunk_mesh.add_quad_to_mesh(
                            &group.face,
                            quad,
                            &chunk.block_data.get(quad.minimum),
                        );
                    }
                }

                let ChunkMesh {
                    positions,
                    normals,
                    indices,
                    colors,
                    uv,
                } = chunk_mesh;

                mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                mesh.set_indices(Some(Indices::U32(indices)));

                visibility.is_visible = true;
            }
        }
    }
}

fn handle_chunk_ready_events(
    mut ready_events: EventReader<ChunkReadyEvent>,
    mut meshing_events: ResMut<VecDeque<ChunkMeshingEvent>>,
) {
    for ready_event in ready_events.iter() {
        meshing_events.push_front(ChunkMeshingEvent(ready_event.1));
    }
}

/// Setups all the required resources for rendering (ie: shader pipelines)
fn setup_render_resources(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    asset_server: Res<AssetServer>,
) {
    let _ = pipelines.set_untracked(
        TERRAIN_PIPELINE_HANDLE,
        PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server.load("shaders/terrain_pbr.vert"),
            fragment: Some(asset_server.load("shaders/terrain_pbr.frag")),
        }),
    );
}

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ChunkMeshingEvent>()
            .init_resource::<VecDeque<ChunkMeshingEvent>>()
            .add_startup_system(setup_render_resources.system())
            .add_system(attach_chunk_render_bundle.system())
            .add_system(handle_chunk_ready_events.system())
            .add_system(mesh_chunks_async.system());
    }
}
