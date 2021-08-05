use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::base::MainPass,
        shader::ShaderStages,
    },
};

use vx_core::world::{ChunkInfo, ChunkMeshInfo, ChunkReadyEvent, WorldUpdateStage, CHUNK_HEIGHT};

const TERRAIN_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 541458694767869);

const FLUID_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 494984949444979);

const SHARED_STANDARD_MATERIAL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 9734486248927);

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
    chunks: Query<(&ChunkMeshInfo, Entity), Added<ChunkInfo>>,
    mut commands: Commands,
) {
    for (mesh_info, ent) in chunks.iter() {
        commands
            .entity(ent)
            .insert_bundle(ChunkRenderBundle {
                mesh: mesh_info.chunk_mesh.clone(),
                material: SHARED_STANDARD_MATERIAL_HANDLE.typed(),
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
                        mesh: mesh_info.fluid_mesh.clone(),
                        material: SHARED_STANDARD_MATERIAL_HANDLE.typed(),
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

struct ChunkTransformAnimation {
    pub start_time: f32,
    pub final_y: f32,
}

fn update_meshes_visibility(
    mut ready_events: EventReader<ChunkReadyEvent>,
    mut chunks: QuerySet<(
        Query<(&Children, &ChunkInfo)>,
        Query<(&mut Visible, &mut Transform)>,
    )>,
    mut entities: bevy::ecs::system::Local<Vec<Entity>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for ready_event in ready_events.iter() {
        if let Ok((children, chunk_info)) = chunks.q0().get(ready_event.1) {
            entities.push(ready_event.1);
            entities.push(children.first().unwrap().clone());

            commands
                .entity(ready_event.1)
                .insert(ChunkTransformAnimation {
                    start_time: time.time_since_startup().as_secs_f32(),
                    final_y: (chunk_info.pos.y * CHUNK_HEIGHT) as f32,
                });

            commands
                .entity(children.first().unwrap().clone())
                .insert(ChunkTransformAnimation {
                    start_time: time.time_since_startup().as_secs_f32(),
                    final_y: (chunk_info.pos.y * CHUNK_HEIGHT) as f32,
                });
        }
    }

    for entity in entities.drain(..) {
        if let Ok((mut visibility, mut transform)) = chunks.q1_mut().get_mut(entity) {
            visibility.is_visible = true;
            transform.translation.y = -128.0
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
    for (entity, mut transform, animation) in chunks.iter_mut() {
        let delta = (time.time_since_startup().as_secs_f32() - animation.start_time)
            .min(ANIMATION_DURATION);
        let ytransform = -ANIMATION_HEIGHT
            + (1. - (1. - (delta / ANIMATION_DURATION)).powi(5))
                * (ANIMATION_HEIGHT + animation.final_y);

        transform.translation.y = ytransform;

        if delta == ANIMATION_DURATION {
            commands.entity(entity).remove::<ChunkTransformAnimation>();
        }
    }
}

/// Setups all the required resources for rendering (ie: shader pipelines)
fn setup_render_resources(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    materials.set_untracked(SHARED_STANDARD_MATERIAL_HANDLE, Default::default());
}

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ClearColor(Color::hex("87CEEB").unwrap()))
            .add_startup_system(setup_render_resources.system())
            .add_system_to_stage(
                WorldUpdateStage::PostUpdate,
                attach_chunk_render_bundle.system(),
            )
            .add_system_to_stage(
                WorldUpdateStage::PostUpdate,
                update_meshes_visibility.system(),
            )
            .add_system_to_stage(
                WorldUpdateStage::PostUpdate,
                step_chunk_ready_animation.system(),
            );
    }
}
