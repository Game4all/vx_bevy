use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

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

use vx_core::world::{chunk_extent, Chunk, ChunkReadyEvent};
use vx_core::{config::GlobalConfig, utils::ChunkMeshBuilder};

struct ChunkMeshingEvent(Entity);

const TERRAIN_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 541458694767869);

const FLUID_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 494984949444979);

#[derive(Bundle)]
pub struct ChunkRenderBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
}

#[inline]
fn padded_chunk_extent() -> Extent3i {
    chunk_extent().padded(1)
}

struct ReusableGreedyQuadsBuffer(GreedyQuadsBuffer);

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
        Self(GreedyQuadsBuffer::new_with_y_up(padded_chunk_extent()))
    }
}

/// Attach to the newly created chunk entities, the render components.
fn attach_chunk_render_bundle(
    chunks: Query<Entity, Added<Chunk>>,
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ent in chunks.iter() {
        commands
            .entity(ent)
            .insert_bundle(ChunkRenderBundle {
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
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(ChunkRenderBundle {
                        mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                        material: mats.add(Default::default()),
                        render_pipelines: RenderPipelines::from_pipelines(vec![
                            RenderPipeline::new(FLUID_PIPELINE_HANDLE.typed()),
                        ]),
                        draw: Default::default(),
                        main_pass: Default::default(),
                        visible: Visible {
                            is_visible: false,
                            is_transparent: true,
                        },
                    })
                    .insert(GlobalTransform::default())
                    .insert(Transform::default());
            });
    }
}

//todo: run this asynchronously
//todo: limit concurrency
fn mesh_chunks_async(
    mut chunks: QuerySet<(
        Query<(&Chunk, &mut Visible, &Handle<Mesh>, &Children)>,
        Query<(&mut Visible, &Handle<Mesh>)>,
    )>,
    mut meshing_events: ResMut<VecDeque<ChunkMeshingEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut greedy_buffer: bevy::ecs::system::Local<ReusableGreedyQuadsBuffer>,
    config: Res<GlobalConfig>,
) {
    for _ in 0..(config.render_distance / 2) {
        if let Some(meshing_event) = meshing_events.pop_back() {
            if let Ok((chunk, mut terrain_visibility, mesh_handle, children)) =
                chunks.q0_mut().get_mut(meshing_event.0)
            {
                let extent = padded_chunk_extent();
                let fluid_mesh_entity = children.first().unwrap().clone();

                greedy_buffer.reset(extent);
                greedy_quads(&chunk.block_data, &extent, &mut greedy_buffer);

                let mut chunk_mesh = ChunkMeshBuilder::default();

                for group in greedy_buffer.quad_groups.iter() {
                    for quad in group.quads.iter() {
                        chunk_mesh.add_quad_to_mesh(
                            &group.face,
                            quad,
                            &chunk.block_data.get(quad.minimum),
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
                } = chunk_mesh;

                {
                    let terrain_mesh = meshes.get_mut(mesh_handle).unwrap();

                    terrain_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    terrain_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    terrain_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                    terrain_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    terrain_mesh.set_indices(Some(Indices::U32(indices)));

                    terrain_visibility.is_visible = true;
                }

                if let Ok((mut fluid_visibility, mesh_handle)) =
                    chunks.q1_mut().get_mut(fluid_mesh_entity)
                {
                    let fluid_mesh = meshes.get_mut(mesh_handle).unwrap();

                    fluid_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, fluid_positions);
                    fluid_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, fluid_normals);
                    fluid_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, fluid_uv);
                    fluid_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, fluid_colors);
                    fluid_mesh.set_indices(Some(Indices::U32(fluid_indices)));

                    fluid_visibility.is_visible = true;
                }
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

    let _ = pipelines.set_untracked(
        FLUID_PIPELINE_HANDLE,
        PipelineDescriptor::default_config(ShaderStages {
            vertex: asset_server.load("shaders/fluid_pbr.vert"),
            fragment: Some(asset_server.load("shaders/fluid_pbr.frag")),
        }),
    );
}

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<ChunkMeshingEvent>()
            .init_resource::<VecDeque<ChunkMeshingEvent>>()
            .insert_resource(ClearColor(Color::hex("87CEEB").unwrap()))
            .add_startup_system(setup_render_resources.system())
            .add_system(attach_chunk_render_bundle.system())
            .add_system(handle_chunk_ready_events.system())
            .add_system(mesh_chunks_async.system());
    }
}
