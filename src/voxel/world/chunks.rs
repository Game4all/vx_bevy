use bevy::{
    math::IVec3,
    pbr::PbrBundle,
    prelude::{
        shape, Assets, Changed, Commands, Entity, GlobalTransform, Mesh, Plugin, Query, ResMut,
        StageLabel, Transform, With,
    },
    utils::HashMap,
};

use crate::voxel::storage::VoxelMap;

use super::{ChunkKey, ChunkShape, Player, Voxel, CHUNK_LENGTH};

// Stores the Entity <-> Chunk voxel data buffer mapping
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
    pub fn detach_entity(&mut self, pos: ChunkKey, entity: Entity) -> Option<Entity> {
        self.0.remove(&pos)
    }
}

/// Checks for the loaded chunks around the player
fn check_visible_regions(
    player: Query<&GlobalTransform, (With<Player>, Changed<GlobalTransform>)>,
    mut chunks: ResMut<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation.floor();

        let nearest_chunk_origin = IVec3::new(
            (player_coords.x as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
            (player_coords.y as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
            (player_coords.z as i32 / CHUNK_LENGTH as i32) * CHUNK_LENGTH as i32,
        );

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
                    println!("Chunk at {:?} doesn't exist yet.", &chunk_key);

                    chunks.insert_default(chunk_key);
                    chunk_entities.attach_entity(
                        chunk_key,
                        cmds.spawn_bundle(PbrBundle {
                            mesh: meshes.add(shape::Box::new(16.0, 16.0, 16.0).into()),
                            transform: Transform::from_translation(chunk_key.location().as_vec3()),
                            ..Default::default()
                        })
                        .id(),
                    );
                }
            }
        }
    }
}

/// Handles dynamically loading / unloading regions (aka chunks) of the world according to camera position.
pub struct VoxelWorldChunkingPlugin;

impl Plugin for VoxelWorldChunkingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ChunkEntities::default())
            .add_system(check_visible_regions);
    }
}
