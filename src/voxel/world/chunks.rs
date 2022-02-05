use bevy::{
    ecs::schedule::ShouldRun,
    math::IVec3,
    prelude::{
        Changed, Commands, CoreStage, Entity, GlobalTransform, ParallelSystemDescriptorCoercion,
        Plugin, Query, Res, ResMut, StageLabel, SystemLabel, SystemStage, With,
    },
    utils::{HashMap, HashSet},
};

use super::{Chunk, ChunkKey, ChunkShape, Player, Voxel, CHUNK_LENGTH};
use crate::voxel::storage::VoxelMap;

/// Updates the current chunk position for the current player.
fn update_player_pos(
    player: Query<&GlobalTransform, (With<Player>, Changed<GlobalTransform>)>,
    mut chunk_pos: ResMut<CurrentLocalPlayerChunk>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation.floor();
        let nearest_chunk_origin = IVec3::new(
            (player_coords.x as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
            0,
            (player_coords.z as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
        );

        if chunk_pos.0.location() != nearest_chunk_origin {
            chunk_pos.0 = ChunkKey::from_ivec3(nearest_chunk_origin);
        }
    }
}

/// Run criteria for the [`update_view_chunks`] system
fn update_view_chunks_criteria(
    chunk_pos: Res<CurrentLocalPlayerChunk>,
    view_distance: Res<ChunkLoadingRadius>,
) -> ShouldRun {
    if chunk_pos.is_changed() || view_distance.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

/// Checks for the loaded chunks around the player and schedules loading of new chunks in sight
fn update_view_chunks(
    player_pos: Res<CurrentLocalPlayerChunk>,
    chunks: Res<VoxelMap<Voxel, ChunkShape>>,
    view_radius: Res<ChunkLoadingRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
) {
    // quick n dirty circular chunk loading.
    //perf: optimize this.
    for x in -view_radius.0..view_radius.0 {
        for z in -view_radius.0..view_radius.0 {
            if x.pow(2) + z.pow(2) >= view_radius.0.pow(2) {
                continue;
            }

            let chunk_key = ChunkKey::from_ivec3(
                player_pos.0.location()
                    + IVec3::new(x * CHUNK_LENGTH as i32, 0, z * CHUNK_LENGTH as i32),
            );

            if !chunks.exists(chunk_key) {
                chunk_command_queue.create.push(chunk_key);
            }
        }
    }

    // quick n dirty circular chunk !loading.
    for loaded_chunk in chunks.chunks.keys() {
        let delta = loaded_chunk.location() - player_pos.0.location();
        if delta.x.pow(2) + delta.y.pow(2) + delta.z.pow(2)
            > view_radius.0.pow(2) * (CHUNK_LENGTH as i32).pow(2)
        {
            chunk_command_queue.destroy.push(*loaded_chunk);
        }
    }

    // load chunks starting from the player position
    chunk_command_queue
        .create
        .sort_unstable_by_key(|key| key.distance(&player_pos.0));
}

/// Creates the requested chunks and attach them an ECS entity.
fn create_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    for request in chunks_command_queue.create.drain(..) {
        chunks.insert_empty(request);
        chunk_entities.attach_entity(request, cmds.spawn().insert(Chunk(request)).id());
    }
}

fn destroy_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    //perf: the despawning should be split between multiple frames so it doesn't freeze when despawning all the chunk entities.
    for command in chunks_command_queue.destroy.drain(..) {
        cmds.entity(chunk_entities.detach_entity(command).unwrap())
            .despawn();
        chunks.remove(&command);
    }
}

fn clear_dirty_chunks(mut dirty_chunks: ResMut<DirtyChunks>) {
    dirty_chunks.0.clear();
}

/// Label for the stage housing the chunk loading systems.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkLoadingStage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
/// Labels for the systems added by [`VoxelWorldChunkingPlugin`]
pub enum ChunkLoadingSystem {
    /// Updates the player current chunk.
    /// The computed position is used for loading / meshing priority systems.
    UpdatePlayerPos,
    /// Runs chunk view distance calculations and queue events for chunk creations and deletions.
    UpdateViewChunks,
    /// Creates the voxel buffers to hold chunk data and attach them a chunk entity in the ECS world.
    CreateChunks,
    /// Clears the dirty chunks list.
    ClearDirtyChunks,
}

/// Handles dynamically loading / unloading regions (aka chunks) of the world according to camera position.
pub struct VoxelWorldChunkingPlugin;

/// Stores the Entity <-> Chunk voxel data buffer mapping
#[derive(Default)]
pub struct ChunkEntities(HashMap<ChunkKey, Entity>);

impl ChunkEntities {
    /// Returns the entity attached to the chunk.
    pub fn entity(&self, pos: ChunkKey) -> Option<Entity> {
        self.0.get(&pos).map(|x| x.clone())
    }

    /// Attaches the specified entity to the chunk data.
    pub fn attach_entity(&mut self, pos: ChunkKey, entity: Entity) {
        self.0.insert(pos, entity);
    }

    /// Detaches the specified entity to the chunk data.
    pub fn detach_entity(&mut self, pos: ChunkKey) -> Option<Entity> {
        self.0.remove(&pos)
    }
}

/// Holds the dirty chunk for the current frame.
#[derive(Default)]
pub struct DirtyChunks(HashSet<ChunkKey>);

#[allow(dead_code)]
impl DirtyChunks {
    pub fn mark_dirty(&mut self, chunk: ChunkKey) {
        self.0.insert(chunk);
    }

    pub fn iter_dirty(&self) -> impl Iterator<Item = &ChunkKey> {
        self.0.iter()
    }

    pub fn num_dirty(&self) -> usize {
        self.0.len()
    }
}

/// Resource storing the current chunk the player is in.
pub struct CurrentLocalPlayerChunk(pub ChunkKey);

// Resource holding the view distance.
pub struct ChunkLoadingRadius(pub i32);

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource::<ChunkLoadingRadius>(ChunkLoadingRadius(16))
            .init_resource::<ChunkEntities>()
            .insert_resource(CurrentLocalPlayerChunk(ChunkKey::from_ivec3(IVec3::ZERO)))
            .init_resource::<ChunkCommandQueue>()
            .init_resource::<DirtyChunks>()
            .add_stage_after(
                CoreStage::Update,
                ChunkLoadingStage,
                SystemStage::parallel()
                    .with_system(update_player_pos.label(ChunkLoadingSystem::UpdatePlayerPos))
                    .with_system(
                        update_view_chunks
                            .label(ChunkLoadingSystem::UpdateViewChunks)
                            .after(ChunkLoadingSystem::UpdatePlayerPos)
                            .with_run_criteria(update_view_chunks_criteria),
                    )
                    .with_system(
                        create_chunks
                            .label(ChunkLoadingSystem::CreateChunks)
                            .after(ChunkLoadingSystem::UpdateViewChunks),
                    ),
            )
            .add_system_to_stage(CoreStage::Last, destroy_chunks)
            .add_system_to_stage(
                CoreStage::Last,
                clear_dirty_chunks.label(ChunkLoadingSystem::ClearDirtyChunks),
            );
    }
}

/// An internal queue tracking the creation / destroy commands for chunks.
#[derive(Default)]
struct ChunkCommandQueue {
    create: Vec<ChunkKey>,
    destroy: Vec<ChunkKey>,
}
