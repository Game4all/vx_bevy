use super::{
    chunks::{ChunkEntities, ChunkLoadingStage, DirtyChunks},
    Chunk, ChunkShape, Voxel, CHUNK_LENGTH,
};
use crate::voxel::{
    render::{mesh_buffer, MeshBuffers},
    storage::VoxelMap,
};
use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

/// Attaches to the newly inserted chunk entities components required for rendering.
pub fn prepare_chunks(
    chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cmds: Commands,
) {
    for (chunk, chunk_key) in chunks.iter() {
        cmds.entity(chunk)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                transform: Transform::from_translation(chunk_key.0.location().as_vec3()),
                visibility: Visibility { is_visible: false },
                ..Default::default()
            })
            .insert(Aabb::from_min_max(
                Vec3::ZERO,
                Vec3::new(
                    CHUNK_LENGTH as f32,
                    CHUNK_LENGTH as f32,
                    CHUNK_LENGTH as f32,
                ),
            ));
    }
}

//perf: reuse mesh buffers
/// Queues meshing tasks for the chunks in need of a remesh.
fn queue_mesh_tasks(
    mut commands: Commands,
    dirty_chunks: Res<DirtyChunks>,
    chunk_entities: Res<ChunkEntities>,
    chunks: Res<VoxelMap<Voxel, ChunkShape>>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    dirty_chunks
        .iter_dirty()
        .filter_map(|key| {
            chunk_entities
                .entity(*key)
                .and_then(|entity| Some((key, entity)))
        })
        .filter_map(|(key, entity)| {
            chunks
                .buffer_at(*key)
                .and_then(|buffer| Some((buffer.clone(), entity)))
        })
        .map(|(buffer, entity)| {
            (
                entity,
                ChunkMeshTask(task_pool.spawn(async move {
                    let mut mesh_buffers = MeshBuffers::<Voxel, ChunkShape>::new(ChunkShape {});
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    mesh_buffer(&buffer, &mut mesh_buffers, &mut mesh, 1.0);

                    mesh
                })),
            )
        })
        .for_each(|(entity, task)| {
            commands.entity(entity).insert(task);
        });
}

/// Polls and process the generated chunk meshes
fn process_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_query: Query<
        (Entity, &Handle<Mesh>, &mut ChunkMeshTask, &mut Visibility),
        With<Chunk>,
    >,
    mut commands: Commands,
) {
    chunk_query.for_each_mut(|(entity, handle, mut mesh_task, mut visibility)| {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            *meshes.get_mut(handle).unwrap() = mesh;
            visibility.is_visible = true;
            commands.entity(entity).remove::<ChunkMeshTask>();
        }
    });
}

/// A stage existing solely for enabling the use of change detection.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkRenderingPrepareStage;

/// Label for the stage housing the chunk rendering systems.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkRenderingStage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
pub enum ChunkRenderingSystem {
    /// Queues meshing tasks for the chunks in need of a remesh.
    QueueMeshTasks,

    /// Polls and process the generated chunk meshes.
    ProcessMeshTasks,
}

/// Handles the rendering of the chunks.
pub struct VoxelWorldRenderingPlugin;

impl Plugin for VoxelWorldRenderingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_stage_after(
            ChunkLoadingStage,
            ChunkRenderingPrepareStage,
            SystemStage::single(prepare_chunks),
        )
        .add_stage_after(
            ChunkRenderingPrepareStage,
            ChunkRenderingStage,
            SystemStage::parallel()
                .with_system(queue_mesh_tasks.label(ChunkRenderingSystem::QueueMeshTasks))
                .with_system(
                    process_mesh_tasks
                        .label(ChunkRenderingSystem::ProcessMeshTasks)
                        .after(ChunkRenderingSystem::QueueMeshTasks),
                ),
        );
    }
}

#[derive(Component)]
struct ChunkMeshTask(Task<Mesh>);
