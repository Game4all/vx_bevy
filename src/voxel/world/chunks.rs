use bevy::{
    math::IVec3,
    prelude::{
        Changed, Commands, CoreSet, DetectChanges, Entity, GlobalTransform, IntoSystemConfig,
        IntoSystemSetConfig, IntoSystemSetConfigs, Plugin, Query, Res, ResMut, Resource, SystemSet,
        With,
    },
    utils::{HashMap, HashSet},
};
use float_ord::FloatOrd;

use super::{player::PlayerController, Chunk, ChunkShape, CHUNK_LENGTH};
use crate::voxel::storage::ChunkMap;
use crate::voxel::Voxel;

/// Updates the current chunk position for the current player.
fn update_player_pos(
    player: Query<&GlobalTransform, (With<PlayerController>, Changed<GlobalTransform>)>,
    mut chunk_pos: ResMut<CurrentLocalPlayerChunk>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation().as_ivec3();
        let nearest_chunk_origin = !IVec3::splat((CHUNK_LENGTH - 1) as i32) & player_coords;

        chunk_pos.world_pos = player_coords;

        if chunk_pos.chunk_min != nearest_chunk_origin {
            chunk_pos.chunk_min = nearest_chunk_origin;
        }
    }
}

/// Run criteria for the [`update_view_chunks`] system
fn update_view_chunks_criteria(
    chunk_pos: Res<CurrentLocalPlayerChunk>,
    view_distance: Res<ChunkLoadRadius>,
) -> bool {
    chunk_pos.is_changed() || view_distance.is_changed()
}

/// Checks for the loaded chunks around the player and schedules loading of new chunks in sight
fn update_view_chunks(
    player_pos: Res<CurrentLocalPlayerChunk>,
    chunk_entities: Res<ChunkEntities>,
    view_radius: Res<ChunkLoadRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
) {
    // quick n dirty circular chunk loading.
    //perf: optimize this.
    for x in -view_radius.horizontal..view_radius.horizontal {
        for z in -view_radius.horizontal..view_radius.horizontal {
            for y in -view_radius.vertical..view_radius.vertical {
                if x.pow(2) + z.pow(2) >= view_radius.horizontal.pow(2) {
                    continue;
                }

                let chunk_key = {
                    let mut pos: IVec3 = player_pos.chunk_min
                        + IVec3::new(
                            x * CHUNK_LENGTH as i32,
                            y * CHUNK_LENGTH as i32,
                            z * CHUNK_LENGTH as i32,
                        );

                    pos.y = pos.y.max(0);

                    pos
                };

                if chunk_entities.entity(chunk_key).is_none() {
                    chunk_command_queue.create.push(chunk_key);
                }
            }
        }
    }

    // quick n dirty circular chunk !loading.
    for loaded_chunk in chunk_entities.0.keys() {
        let delta: IVec3 = *loaded_chunk - player_pos.chunk_min;

        // Compiler complains that this is a bug
        #[allow(clippy::suspicious_operation_groupings)]
        if delta.x.pow(2) + delta.z.pow(2)
            > view_radius.horizontal.pow(2) * (CHUNK_LENGTH as i32).pow(2)
            || delta.y.pow(2) > view_radius.vertical.pow(2) * (CHUNK_LENGTH as i32).pow(2)
        {
            chunk_command_queue.destroy.push(*loaded_chunk);
        }
    }

    // load chunks starting from the player position
    chunk_command_queue.create.sort_unstable_by_key(|key| {
        FloatOrd(key.as_vec3().distance(player_pos.chunk_min.as_vec3()))
    });
}

/// Creates the requested chunks and attach them an ECS entity.
fn create_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    chunks_command_queue
        .create
        .drain(..)
        .for_each(|request| chunk_entities.attach_entity(request, cmds.spawn(Chunk(request)).id()));
}

fn destroy_chunks(
    mut chunks_command_queue: ResMut<ChunkCommandQueue>,
    mut chunks: ResMut<ChunkMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    for command in chunks_command_queue.destroy.drain(..) {
        cmds.entity(chunk_entities.detach_entity(command).unwrap())
            .despawn();
        chunks.remove(command);
    }
}

fn clear_dirty_chunks(mut dirty_chunks: ResMut<DirtyChunks>) {
    dirty_chunks.0.clear();
}

/// Label for the stage housing the chunk loading systems.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
#[system_set(base)]
pub struct ChunkLoadingStage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemSet)]
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
#[derive(Default, Resource)]
pub struct ChunkEntities(HashMap<IVec3, Entity>);

impl ChunkEntities {
    /// Returns the entity attached to the chunk.
    pub fn entity(&self, pos: IVec3) -> Option<Entity> {
        self.0.get(&pos).copied()
    }

    /// Attaches the specified entity to the chunk data.
    pub fn attach_entity(&mut self, pos: IVec3, entity: Entity) {
        self.0.insert(pos, entity);
    }

    /// Detaches the specified entity to the chunk data.
    pub fn detach_entity(&mut self, pos: IVec3) -> Option<Entity> {
        self.0.remove(&pos)
    }

    /// Returns an iterator iterating over the loaded chunk keys.
    pub fn iter_keys(&self) -> impl Iterator<Item = &IVec3> {
        self.0.keys()
    }

    /// Return the number of loaded chunks.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Holds the dirty chunk for the current frame.
#[derive(Default, Resource)]
pub struct DirtyChunks(HashSet<IVec3>);

#[allow(dead_code)]
impl DirtyChunks {
    pub fn mark_dirty(&mut self, chunk: IVec3) {
        self.0.insert(chunk);
    }

    pub fn iter_dirty(&self) -> impl Iterator<Item = &IVec3> {
        self.0.iter()
    }

    pub fn num_dirty(&self) -> usize {
        self.0.len()
    }
}

/// Resource storing the current chunk the player is in as well as its current coords.
#[derive(Resource)]
pub struct CurrentLocalPlayerChunk {
    pub chunk_min: IVec3,
    pub world_pos: IVec3,
}

// Resource holding the view distance.
#[derive(Resource)]
pub struct ChunkLoadRadius {
    pub horizontal: i32,
    pub vertical: i32,
}

/// A queue tracking the creation / destroy commands for chunks.
#[derive(Default, Resource)]
pub struct ChunkCommandQueue {
    create: Vec<IVec3>,
    destroy: Vec<IVec3>,
}

impl ChunkCommandQueue {
    pub fn queue_unload<'a>(&mut self, region: impl Iterator<Item = &'a IVec3>) {
        self.destroy.extend(region);
    }
}

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource::<ChunkLoadRadius>(ChunkLoadRadius {
            horizontal: 16,
            vertical: 6,
        })
        .init_resource::<ChunkEntities>()
        .insert_resource(CurrentLocalPlayerChunk {
            chunk_min: IVec3::ZERO,
            world_pos: IVec3::ZERO,
        })
        .init_resource::<ChunkCommandQueue>()
        .init_resource::<DirtyChunks>()
        .configure_set(
            ChunkLoadingStage
                .after(CoreSet::Update)
                .before(CoreSet::Last),
        )
        .configure_set(ChunkLoadingSystem::ClearDirtyChunks.in_base_set(CoreSet::Last))
        .configure_sets(
            (
                ChunkLoadingSystem::UpdatePlayerPos,
                ChunkLoadingSystem::UpdateViewChunks,
                ChunkLoadingSystem::CreateChunks,
            )
                .in_base_set(ChunkLoadingStage),
        )
        .add_systems((
            update_player_pos.in_set(ChunkLoadingSystem::UpdatePlayerPos),
            update_view_chunks
                .in_set(ChunkLoadingSystem::UpdateViewChunks)
                .run_if(update_view_chunks_criteria),
            create_chunks.in_set(ChunkLoadingSystem::CreateChunks),
            destroy_chunks.in_base_set(CoreSet::Last),
            clear_dirty_chunks.in_set(ChunkLoadingSystem::ClearDirtyChunks),
        ));
    }
}
