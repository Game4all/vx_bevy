use bevy::{
    math::IVec3,
    prelude::{
        Changed, Commands, Entity, EventReader, EventWriter, GlobalTransform,
        ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut, StageLabel, SystemLabel,
        SystemStage, With,
    },
    utils::HashMap,
};

use super::{Chunk, ChunkKey, ChunkShape, Player, Voxel, CHUNK_LENGTH};
use crate::voxel::storage::{VoxelBuffer, VoxelMap};

// Stores the Entity <-> Chunk voxel data buffer mapping
#[derive(Default)]
pub struct ChunkEntities(HashMap<ChunkKey, Entity>);

#[allow(dead_code)]
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

        chunk_pos.0 = ChunkKey::from_ivec3(nearest_chunk_origin);
    }
}

/// Checks for the loaded chunks around the player and schedules loading of new chunks in sight
fn update_view_chunks(
    player_pos: Res<CurrentLocalPlayerChunk>,
    chunks: Res<VoxelMap<Voxel, ChunkShape>>,
    view_radius: Res<ChunkLoadingRadius>,
    mut loads: EventWriter<ChunkCreateKey>,
    mut unloads: EventWriter<ChunkDestroyKey>,
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
                loads.send(ChunkCreateKey(chunk_key));
            }
        }
    }

    // quick n dirty circular chunk !loading.
    for loaded_chunk in chunks.chunks.keys() {
        let delta = loaded_chunk.location() - player_pos.0.location();
        if delta.x.pow(2) + delta.y.pow(2) + delta.z.pow(2)
            > view_radius.0.pow(2) * (CHUNK_LENGTH as i32).pow(2)
        {
            unloads.send(ChunkDestroyKey(*loaded_chunk));
        }
    }
}

/// Creates the requested chunks and attach them an ECS entity.
fn create_chunks(
    mut requests: EventReader<ChunkCreateKey>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    //perf: the spawning should be split between multiple frames so it doesn't freeze when spawning all the chunk entities.
    for request in requests.iter() {
        //todo: at some point we may want to split the buffer and entity creation into two separate systems for handling procgen and stuff like loading data from disk.
        chunks.insert(
            request.0,
            VoxelBuffer::<Voxel, ChunkShape>::new(ChunkShape {}, Voxel(1)),
        );
        chunk_entities.attach_entity(request.0, cmds.spawn().insert(Chunk(request.0)).id());
    }
}

fn destroy_chunks(
    mut requests: EventReader<ChunkDestroyKey>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    //perf: the despawning should be split between multiple frames so it doesn't freeze when despawning all the chunk entities.
    for request in requests.iter() {
        //todo: at some point we may want to split the buffer and entity creation into two separate systems for handling procgen and stuff like loading data from disk.
        cmds.entity(chunk_entities.detach_entity(request.0).unwrap())
            .despawn();
        chunks.remove(request.0);
    }
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
    /// Destroy the ECS entities and their buffer data.
    DestroyChunks,
}

/// Handles dynamically loading / unloading regions (aka chunks) of the world according to camera position.
pub struct VoxelWorldChunkingPlugin;

/// Resource storing the current chunk the player is in.
pub struct CurrentLocalPlayerChunk(pub ChunkKey);

// Resource holding the view distance.
pub struct ChunkLoadingRadius(pub i32);

/// An event indicating a chunk received an update.
pub struct ChunkUpdateEvent(pub ChunkKey);

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ChunkEntities::default())
            .add_event::<ChunkCreateKey>()
            .add_event::<ChunkDestroyKey>()
            .add_event::<ChunkUpdateEvent>()
            .insert_resource::<ChunkLoadingRadius>(ChunkLoadingRadius(16))
            .insert_resource(CurrentLocalPlayerChunk(ChunkKey::from_ivec3(IVec3::ZERO)))
            .add_stage(
                ChunkLoadingStage,
                SystemStage::parallel()
                    .with_system(update_player_pos.label(ChunkLoadingSystem::UpdatePlayerPos))
                    .with_system(
                        update_view_chunks
                            .label(ChunkLoadingSystem::UpdateViewChunks)
                            .after(ChunkLoadingSystem::UpdatePlayerPos),
                    )
                    .with_system(
                        create_chunks
                            .label(ChunkLoadingSystem::CreateChunks)
                            .after(ChunkLoadingSystem::UpdateViewChunks),
                    )
                    .with_system(
                        destroy_chunks
                            .label(ChunkLoadingSystem::DestroyChunks)
                            .after(ChunkLoadingSystem::CreateChunks),
                    ),
            );
    }
}

/// An internal event requesting the creation of a chunk at the specified origin
struct ChunkCreateKey(ChunkKey);

/// An internal requesting the deletion of a chunk at the specified origin
struct ChunkDestroyKey(ChunkKey);
