use std::cell::RefCell;

use super::{
    chunks::{ChunkEntities, ChunkLoadingStage, DirtyChunks},
    Chunk, ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH,
};
use crate::{
    utils::ThreadLocalRes,
    voxel::{
        render::{mesh_buffer, MeshBuffers},
        storage::VoxelMap,
    },
};
use bevy::{
    prelude::*,
    render::{primitives::Aabb, render_resource::PrimitiveTopology},
    tasks::ComputeTaskPool,
};

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

/// Marks chunk entities that need meshing by attaching them a [`NeedsMeshing`] marker component.
fn queue_meshing(dirty_chunks: Res<DirtyChunks>, mut queue_mesh: ResMut<ChunkMeshQueue>) {
    queue_mesh.0.extend(dirty_chunks.iter_dirty());
}

/// Meshes the specified chunks.
fn mesh_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_queue: ResMut<ChunkMeshQueue>,
    chunk_query: Query<&Handle<Mesh>, With<Chunk>>,
    chunk_entities: Res<ChunkEntities>,
    mesh_buffers: Local<ThreadLocalRes<RefCell<MeshBuffers<Voxel, ChunkShape>>>>,
    chunks: Res<VoxelMap<Voxel, ChunkShape>>,
    mesh_budget: Res<WorldChunksMeshingFrameBudget>,
    task_pool: Res<ComputeTaskPool>,
) {
    let drain_size = if mesh_queue.0.len() < mesh_budget.meshes_per_frame {
        mesh_queue.0.len()
    } else {
        mesh_budget.meshes_per_frame
    };

    let generated_meshes = task_pool.scope(|scope| {
        mesh_queue
            .0
            .drain(..drain_size)
            .filter_map(|key| {
                chunk_entities
                    .entity(key)
                    .and_then(|entity| Some((entity, chunks.buffer_at(key).unwrap())))
            })
            .map(|(entity, buffer)| {
                let mesh_buffers_handle = mesh_buffers.get_handle();
                scope.spawn_local(async move {
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    let mut mesh_buffers = &mut mesh_buffers_handle
                        .get_or(|| {
                            RefCell::new(MeshBuffers::<Voxel, ChunkShape>::new(ChunkShape {}))
                        })
                        .borrow_mut();

                    mesh_buffer(buffer, &mut mesh_buffers, &mut mesh, 1.0);

                    (entity, mesh)
                })
            })
            .collect()
    });

    // meshes

    for (entity, mesh) in generated_meshes {
        *meshes.get_mut(chunk_query.get(entity).unwrap()).unwrap() = mesh;
    }
}

/// A stage existing solely for enabling the use of change detection.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkRenderingPrepareStage;

/// Label for the stage housing the chunk rendering systems.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkRenderingStage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
pub enum ChunkRenderingSystem {
    /// Marks chunk entities that need meshing.
    QueueMeshing,

    /// Mesh actual chunks
    MeshChunks,
}

/// Handles the rendering of the chunks.
pub struct VoxelWorldRenderingPlugin;

pub struct WorldChunksMeshingFrameBudget {
    pub meshes_per_frame: usize,
}

impl Plugin for VoxelWorldRenderingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<ChunkMeshQueue>()
            .add_stage_after(
                ChunkLoadingStage,
                ChunkRenderingPrepareStage,
                SystemStage::single(prepare_chunks),
            )
            .add_stage_after(
                ChunkRenderingPrepareStage,
                ChunkRenderingStage,
                SystemStage::parallel()
                    .with_system(queue_meshing.label(ChunkRenderingSystem::QueueMeshing))
                    .with_system(
                        mesh_chunks
                            .label(ChunkRenderingSystem::MeshChunks)
                            .after(ChunkRenderingSystem::QueueMeshing),
                    ),
            )
            .insert_resource(WorldChunksMeshingFrameBudget {
                meshes_per_frame: 16,
            });
    }
}

#[derive(Default)]
struct ChunkMeshQueue(Vec<ChunkKey>);
