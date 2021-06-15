use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::base::MainPass,
        shader::ShaderStages,
    },
};

use vx_core::world::{ChunkInfo, ChunkMeshInfo};

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

/// Attach to the newly created chunk entities, the render components.
fn attach_chunk_render_bundle(
    chunks: Query<(&ChunkMeshInfo, Entity), Added<ChunkInfo>>,
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for (mesh_info, ent) in chunks.iter() {
        commands
            .entity(ent)
            .insert_bundle(ChunkRenderBundle {
                mesh: mesh_info.chunk_mesh.clone(),
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
                        mesh: mesh_info.fluid_mesh.clone(),
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

fn update_meshes_visibility(
    mut chunks: QuerySet<(
        Query<(Entity, &Children), Changed<ChunkMeshInfo>>,
        Query<&mut Visible>,
    )>,
    mut entities: bevy::ecs::system::Local<Vec<Entity>>,
) {
    for (entity, children) in chunks.q0().iter() {
        entities.push(entity);
        entities.push(children.first().unwrap().clone());
    }

    for entity in entities.drain(..) {
        if let Ok(mut visibility) = chunks.q1_mut().get_mut(entity) {
            visibility.is_visible = true;
        }
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
        app.insert_resource(ClearColor(Color::hex("87CEEB").unwrap()))
            .add_startup_system(setup_render_resources.system())
            .add_system(attach_chunk_render_bundle.system())
            .add_system(update_meshes_visibility.system());
    }
}
