use std::cell::RefCell;

use super::{
    chunks::{ChunkEntities, ChunkLoadingSet, DirtyChunks},
    terrain::AsyncTerrainGenSet,
    Chunk, ChunkShape, Voxel, CHUNK_LENGTH,
};
use crate::voxel::{
    render::{mesh_buffer, MeshBuffers, VoxelTerrainMeshBundle},
    storage::ChunkMap,
};
use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use once_cell::sync::Lazy;
use thread_local::ThreadLocal;

/// Attaches to the newly inserted chunk entities components required for rendering.
pub fn prepare_chunks(
    chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cmds: Commands,
) {
    for (chunk, chunk_key) in chunks.iter() {
        cmds.entity(chunk).insert(VoxelTerrainMeshBundle {
            mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
            transform: Transform::from_translation(chunk_key.0.as_vec3()),
            visibility: Visibility::Hidden,
            aabb: Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_LENGTH as f32)),
            ..Default::default()
        });
    }
}

// a pool of mesh buffers shared between meshing tasks.
static SHARED_MESH_BUFFERS: Lazy<ThreadLocal<RefCell<MeshBuffers<Voxel, ChunkShape>>>> =
    Lazy::new(ThreadLocal::default);

/// Queues meshing tasks for the chunks in need of a remesh.
fn queue_mesh_tasks(
    mut commands: Commands,
    dirty_chunks: Res<DirtyChunks>,
    chunk_entities: Res<ChunkEntities>,
    chunks: Res<ChunkMap<Voxel, ChunkShape>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    dirty_chunks
        .iter_dirty()
        .filter_map(|key| chunk_entities.entity(*key).map(|entity| (key, entity)))
        .filter_map(|(key, entity)| {
            chunks
                .buffer_at(*key)
                .map(|buffer| (buffer.clone(), entity))
        })
        .map(|(buffer, entity)| {
            (
                entity,
                ChunkMeshingTask(task_pool.spawn(async move {
                    let mut mesh_buffers = SHARED_MESH_BUFFERS
                        .get_or(|| {
                            RefCell::new(MeshBuffers::<Voxel, ChunkShape>::new(ChunkShape {}))
                        })
                        .borrow_mut();

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
        (
            Entity,
            &Handle<Mesh>,
            &mut ChunkMeshingTask,
            &mut Visibility,
        ),
        With<Chunk>,
    >,
    mut commands: Commands,
) {
    chunk_query.for_each_mut(|(entity, handle, mut mesh_task, mut visibility)| {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            *meshes.get_mut(handle).unwrap() = mesh;
            *visibility = Visibility::Visible;
            commands.entity(entity).remove::<ChunkMeshingTask>();
        }
    });
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, SystemSet)]
pub struct AsyncChunkMeshSet;

/// Handles the meshing of the chunks.
pub struct VoxelWorldMeshingPlugin;

impl Plugin for VoxelWorldMeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            (
                prepare_chunks,
                queue_mesh_tasks,
                process_mesh_tasks
                    .after(prepare_chunks)
                    .after(queue_mesh_tasks),
            )
                .in_set(AsyncChunkMeshSet),
        )
        .configure_set(
            AsyncChunkMeshSet
                .in_base_set(CoreSet::Update)
                .after(AsyncTerrainGenSet)
                .after(bevy::scene::scene_spawner_system),
        );
    }
}

#[derive(Component)]
pub struct ChunkMeshingTask(Task<Mesh>);
