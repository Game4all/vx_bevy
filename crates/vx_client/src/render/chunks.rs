use super::Visibility;
use crate::utils::{ChunkMeshBuilder, ThreadLocalRes};
use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
        render_graph::base::MainPass,
        shader::ShaderStages,
    },
    utils::Instant,
};
use building_blocks::{
    core::Extent3i,
    mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG},
    prelude::Get,
    storage::{copy_extent, Array3x1},
};
use std::cell::RefCell;
use vx_core::{
    voxel::Voxel,
    world::{
        chunk_extent, ChunkInfo, ChunkMapReader, ChunkMeshingRequest, ChunkReadyEvent,
        ChunkUpdateEvent, WorldTaskPool, WorldUpdateStage,
    },
};

pub const CHUNK_MESHING_TIME: DiagnosticId = DiagnosticId::from_u128(489617772449846);

const TERRAIN_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 541458694767869);

const FLUID_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 494984949444979);

const SHARED_STANDARD_MATERIAL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 9734486248927);

// A system stage used exclusively for meshing systems to allow using change detection.
#[derive(StageLabel, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct ChunkRenderStage;

pub struct ChunkMeshInfo {
    pub fluid_mesh: Handle<Mesh>,
    pub chunk_mesh: Handle<Mesh>,
    pub is_empty: bool,
}

#[derive(Bundle)]
pub struct ChunkRenderBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub render_visibility: Visible,
    pub render_pipelines: RenderPipelines,
    pub visibility: Visibility,
}

/// Attach to the newly created chunk entities, the render components.
fn attach_chunk_render_bundle(
    chunks: Query<Entity, Added<ChunkInfo>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    chunks.for_each(|ent| {
        let chunk_mesh = meshes.add(Mesh::new(PrimitiveTopology::TriangleList));
        let fluid_mesh = meshes.add(Mesh::new(PrimitiveTopology::TriangleList));

        commands
            .entity(ent)
            .insert_bundle(ChunkRenderBundle {
                mesh: chunk_mesh.clone(),
                material: SHARED_STANDARD_MATERIAL_HANDLE.typed(),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    TERRAIN_PIPELINE_HANDLE.typed(),
                )]),
                draw: Default::default(),
                main_pass: Default::default(),
                render_visibility: Visible {
                    is_visible: false,
                    is_transparent: false,
                },
                visibility: Default::default(),
            })
            .insert(ChunkMeshInfo {
                chunk_mesh,
                fluid_mesh: fluid_mesh.clone(),
                is_empty: true,
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(ChunkRenderBundle {
                        mesh: fluid_mesh.clone(),
                        material: SHARED_STANDARD_MATERIAL_HANDLE.typed(),
                        render_pipelines: RenderPipelines::from_pipelines(vec![
                            RenderPipeline::new(FLUID_PIPELINE_HANDLE.typed()),
                        ]),
                        draw: Default::default(),
                        main_pass: Default::default(),
                        render_visibility: Visible {
                            is_visible: false,
                            is_transparent: true,
                        },
                        visibility: Default::default(),
                    })
                    .insert(GlobalTransform::default())
                    .insert(Transform::default());
            });
    });
}

struct ChunkTransformAnimation {
    pub start_time: f32,
}

fn attach_animation_components(
    mut ready_events: EventReader<ChunkReadyEvent>,
    mut chunks: QuerySet<(Query<(&Children, &ChunkMeshInfo)>, Query<&mut Transform>)>,
    mut entities: bevy::ecs::system::Local<Vec<Entity>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for ready_event in ready_events.iter() {
        if let Ok((children, mesh_info)) = chunks.q0().get(ready_event.1) {
            if mesh_info.is_empty {
                //don't animate empty chunks.
                continue;
            }

            entities.push(ready_event.1);
            entities.push(children.first().unwrap().clone());

            commands
                .entity(ready_event.1)
                .insert(ChunkTransformAnimation {
                    start_time: time.time_since_startup().as_secs_f32(),
                });

            commands
                .entity(children.first().unwrap().clone())
                .insert(ChunkTransformAnimation {
                    start_time: time.time_since_startup().as_secs_f32(),
                });
        }
    }

    for entity in entities.drain(..) {
        if let Ok(mut transform) = chunks.q1_mut().get_mut(entity) {
            transform.translation.y = -ANIMATION_HEIGHT;
        }
    }
}

const ANIMATION_DURATION: f32 = 0.8;
const ANIMATION_HEIGHT: f32 = 128.;

fn step_chunk_ready_animation(
    mut chunks: Query<(Entity, &mut Transform, &ChunkTransformAnimation)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    chunks.for_each_mut(|(entity, mut transform, animation)| {
        let delta = (time.time_since_startup().as_secs_f32() - animation.start_time)
            .min(ANIMATION_DURATION);
        let ytransform = -ANIMATION_HEIGHT
            + (1. - (1. - (delta / ANIMATION_DURATION)).powi(5)) * ANIMATION_HEIGHT;

        transform.translation.y = ytransform;

        if delta == ANIMATION_DURATION {
            commands.entity(entity).remove::<ChunkTransformAnimation>();
        }
    });
}

fn update_meshes_visibility(
    mut meshes: QuerySet<(
        Query<(&Children, &ChunkMeshInfo, Entity), Changed<ChunkMeshInfo>>,
        Query<&mut Visibility>,
    )>,
    mut entities: bevy::ecs::system::Local<Vec<Entity>>,
) {
    meshes.q0().for_each(|(child, mesh_info, entity)| {
        if !mesh_info.is_empty {
            entities.extend(&[child.first().unwrap().clone(), entity]);
        }
    });

    for entity in entities.drain(..) {
        if let Ok(mut visibility) = meshes.q1_mut().get_mut(entity) {
            visibility.visible = true;
        }
    }
}

#[inline]
fn padded_chunk_extent() -> Extent3i {
    chunk_extent().padded(1)
}

fn queue_meshing(
    mut ready_entities: EventReader<ChunkReadyEvent>,
    mut update_events: EventReader<ChunkUpdateEvent>,
    mut meshing_events: EventWriter<ChunkMeshingRequest>,
) {
    meshing_events.send_batch(
        ready_entities
            .iter()
            .map(|event| ChunkMeshingRequest(event.1)),
    );

    meshing_events.send_batch(
        update_events
            .iter()
            .map(|event| ChunkMeshingRequest(event.0)),
    );
}

struct ReusableMeshBuffer {
    greedy_buffer: GreedyQuadsBuffer,
    padded_buffer: Array3x1<Voxel>,
}

impl Default for ReusableMeshBuffer {
    fn default() -> Self {
        Self {
            greedy_buffer: GreedyQuadsBuffer::new(
                padded_chunk_extent(),
                RIGHT_HANDED_Y_UP_CONFIG.quad_groups(),
            ),
            padded_buffer: Array3x1::fill(padded_chunk_extent(), Voxel::Empty),
        }
    }
}

fn mesh_chunks(
    mut chunks: Query<(&ChunkInfo, &mut ChunkMeshInfo)>,
    mut ready_entities: EventReader<ChunkMeshingRequest>,
    mut meshes: ResMut<Assets<Mesh>>,
    local_buffers: bevy::ecs::system::Local<ThreadLocalRes<RefCell<ReusableMeshBuffer>>>,
    chunk_map: ChunkMapReader,
    task_pool: Res<WorldTaskPool>,
    mut diagnostics: ResMut<Diagnostics>,
) {
    let before_meshing_time = Instant::now();

    let mesh_results = task_pool.scope(|scope| {
        for meshing_event in ready_entities.iter() {
            match chunks.get_component::<ChunkInfo>(meshing_event.0) {
                Ok(chunk_info) => {
                    if let Some(chunk_data) = chunk_map.get_chunk_data(chunk_info.pos) {
                        let buffer_handle = local_buffers.get_handle();
                        scope.spawn(async move {
                            let buffer: &mut ReusableMeshBuffer =
                                &mut buffer_handle.get().borrow_mut();

                            let extent = chunk_data.extent();
                            let padded_extent = chunk_data.extent().padded(1);

                            buffer.padded_buffer.set_minimum(padded_extent.minimum);

                            copy_extent(extent, chunk_data, &mut buffer.padded_buffer);

                            buffer.greedy_buffer.reset(padded_extent);
                            greedy_quads(
                                &buffer.padded_buffer,
                                &padded_extent,
                                &mut buffer.greedy_buffer,
                            );

                            if buffer.greedy_buffer.num_quads() != 0 {
                                let mut chunk_mesh_builder = ChunkMeshBuilder::default();

                                for group in buffer.greedy_buffer.quad_groups.iter() {
                                    for quad in group.quads.iter() {
                                        chunk_mesh_builder.add_quad_to_mesh(
                                            &group.face,
                                            quad,
                                            &buffer.padded_buffer.get(quad.minimum),
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

                                (meshing_event.0, Some((terrain_mesh, fluid_mesh)))
                            } else {
                                (meshing_event.0, None)
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

    for meshing_results in mesh_results {
        if let Ok(mut mesh_info) = chunks.get_component_mut::<ChunkMeshInfo>(meshing_results.0) {
            if let Some((terrain_mesh, fluid_mesh)) = meshing_results.1 {
                *meshes.get_mut(&mesh_info.chunk_mesh).unwrap() = terrain_mesh;
                *meshes.get_mut(&mesh_info.fluid_mesh).unwrap() = fluid_mesh;

                mesh_info.is_empty = false;
            } else {
                mesh_info.is_empty = true;
            }
        }
    }

    let after_chunk_meshing = Instant::now() - before_meshing_time;
    diagnostics.add_measurement(CHUNK_MESHING_TIME, after_chunk_meshing.as_secs_f64());
}

/// Setups all the required resources for rendering (ie: shader pipelines) and debug diagnostics
fn setup(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut diagnostics: ResMut<Diagnostics>,
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

    diagnostics.add(Diagnostic::new(
        CHUNK_MESHING_TIME,
        "Avg. chunk meshing time (s)",
        3,
    ));

    materials.set_untracked(SHARED_STANDARD_MATERIAL_HANDLE, Default::default());
}

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ClearColor(Color::hex("87CEEB").unwrap()))
            .add_startup_system(setup.system())
            .add_stage_after(
                WorldUpdateStage::PostUpdate,
                ChunkRenderStage,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                WorldUpdateStage::PostUpdate,
                attach_chunk_render_bundle
                    .system()
                    .label("attach_chunk_render_bundle"),
            )
            .add_system_to_stage(
                ChunkRenderStage,
                queue_meshing
                    .system()
                    .label("queue_meshing_for_ready_chunks"),
            )
            .add_system_to_stage(
                ChunkRenderStage,
                mesh_chunks
                    .system()
                    .label("mesh_chunks")
                    .after("queue_meshing_for_ready_chunks"),
            )
            .add_system_to_stage(
                ChunkRenderStage,
                attach_animation_components
                    .system()
                    .label("attach_animation_components")
                    .after("mesh_chunks"),
            )
            .add_system_to_stage(
                ChunkRenderStage,
                update_meshes_visibility
                    .system()
                    .label("update_meshes_visibility")
                    .after("attach_animation_components"),
            )
            .add_system_to_stage(
                ChunkRenderStage,
                step_chunk_ready_animation
                    .system()
                    .label("step_chunk_ready_animation")
                    .after("update_meshes_visibility"),
            );
    }
}
