use bevy::{
    math::IVec3,
    pbr::PbrBundle,
    prelude::{
        shape, Assets, Changed, Commands, Entity, EventReader, EventWriter, GlobalTransform, Mesh,
        ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut, SystemLabel, Transform, With,
    },
    utils::HashMap,
};

use crate::voxel::storage::VoxelMap;

use super::{ChunkKey, ChunkShape, Player, Voxel, CHUNK_LENGTH};

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

/// Checks for the loaded chunks around the player and schedules loading of new chunks in sight
fn update_view_chunks(
    player: Query<&GlobalTransform, (With<Player>, Changed<GlobalTransform>)>,
    chunks: Res<VoxelMap<Voxel, ChunkShape>>,
    mut loads: EventWriter<ChunkCreateKey>,
    mut unloads: EventWriter<ChunkDestroyKey>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation.floor();

        let nearest_chunk_origin = IVec3::new(
            (player_coords.x as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
            (player_coords.y as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
            (player_coords.z as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
        );

        // quick n dirty circular chunk loading.
        for x in -8i32..8i32 {
            for z in -8i32..8i32 {
                if x.pow(2) + z.pow(2) >= 8i32.pow(2) {
                    continue;
                }

                let chunk_key = ChunkKey::from_ivec3(
                    nearest_chunk_origin
                        + IVec3::new(x * CHUNK_LENGTH as i32, 0, z * CHUNK_LENGTH as i32),
                );

                if !chunks.exists(chunk_key) {
                    loads.send(ChunkCreateKey(chunk_key));
                }
            }
        }

        // quick n dirty circular chunk !loading.
        for loaded_chunk in chunks.chunks.keys() {
            let delta = loaded_chunk.location() - nearest_chunk_origin;
            if delta.x.pow(2) + delta.y.pow(2) + delta.z.pow(2) > 512i32.pow(2) {
                unloads.send(ChunkDestroyKey(*loaded_chunk));
            }
        }
    }
}

/// Creates the requested chunks and attach them an ECS entity.
fn create_chunks(
    mut requests: EventReader<ChunkCreateKey>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cmds: Commands,
) {
    for request in requests.iter() {
        //todo: at some point we may want to split the buffer and entity creation into two separate systems for handling procgen and stuff like loading data from disk.
        chunks.insert_default(request.0);
        chunk_entities.attach_entity(
            request.0,
            cmds.spawn_bundle(PbrBundle {
                mesh: meshes.add(shape::Box::new(16.0, 16.0, 16.0).into()),
                transform: Transform::from_translation(request.0.location().as_vec3()),
                ..Default::default()
            })
            .id(),
        );
    }
}

fn destroy_chunks(
    mut requests: EventReader<ChunkDestroyKey>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
) {
    for request in requests.iter() {
        //todo: at some point we may want to split the buffer and entity creation into two separate systems for handling procgen and stuff like loading data from disk.
        cmds.entity(chunk_entities.detach_entity(request.0).unwrap())
            .despawn();
        chunks.remove(request.0);
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
pub enum ChunkLoadingSystem {
    /// Runs chunk view distance calculations and queue events for chunk creations and deletions.
    UpdateViewChunks,
    /// Creates the voxel buffers to hold chunk data and attach them a chunk entity in the ECS world.
    CreateChunks,
    /// Destroy the ECS entities and their buffer data.
    DestroyChunks,
}

/// Handles dynamically loading / unloading regions (aka chunks) of the world according to camera position.
pub struct VoxelWorldChunkingPlugin;

struct ChunkCreateKey(ChunkKey);
struct ChunkDestroyKey(ChunkKey);

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ChunkEntities::default())
            .add_event::<ChunkCreateKey>()
            .add_event::<ChunkDestroyKey>()
            .add_system(update_view_chunks.label(ChunkLoadingSystem::UpdateViewChunks))
            .add_system(
                create_chunks
                    .label(ChunkLoadingSystem::CreateChunks)
                    .after(ChunkLoadingSystem::UpdateViewChunks),
            )
            .add_system(
                destroy_chunks
                    .label(ChunkLoadingSystem::DestroyChunks)
                    .after(ChunkLoadingSystem::CreateChunks),
            );
    }
}
