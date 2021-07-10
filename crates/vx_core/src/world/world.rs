use std::{collections::VecDeque, sync::Arc};

use bevy::{math::IVec2, prelude::*, render::pipeline::PrimitiveTopology};
use building_blocks::storage::Array3x1;
use heron::prelude::*;

use super::{
    chunk2global, chunk_extent, global2chunk,
    worldgen::{NoiseTerrainGenerator, TerrainGenerator},
    ChunkDataBundle, ChunkDespawnRequest, ChunkInfo, ChunkLoadRequest, ChunkLoadState,
    ChunkMapReader, ChunkMapWriter, ChunkMeshInfo, ChunkReadyEvent, ChunkSpawnRequest,
    WorldTaskPool,
};
use crate::{config::GlobalConfig, Player};

/// Handles the visibility checking of the currently loaded chunks around the player.
/// This will accordingly emit [`ChunkSpawnRequest`] events for chunks that need to be loaded since they entered the player's view distance and [`ChunkDespawnRequest`] for
/// chunks out of the player's view distance.
pub(crate) fn update_visible_chunks(
    player: Query<(&Transform, &Player)>,
    chunk_map: ChunkMapReader,
    config: Res<GlobalConfig>,
    mut load_radius_chunks: bevy::ecs::system::Local<Vec<IVec2>>,
    mut spawn_requests: EventWriter<ChunkSpawnRequest>,
    mut despawn_requests: EventWriter<ChunkDespawnRequest>,
) {
    if let Ok((transform, _)) = player.single() {
        let pos = global2chunk(transform.translation);

        for dx in -config.render_distance..=config.render_distance {
            for dy in -config.render_distance..=config.render_distance {
                if dx.pow(2) + dy.pow(2) >= config.render_distance.pow(2) {
                    continue;
                };

                let chunk_pos = pos + (dx, dy).into();
                if !chunk_map.chunk_exists(&chunk_pos) {
                    load_radius_chunks.push(chunk_pos);
                }
            }
        }

        load_radius_chunks.sort_by_key(|a| (a.x.pow(2) + a.y.pow(2)));

        spawn_requests.send_batch(
            load_radius_chunks
                .drain(..)
                .map(|c| ChunkSpawnRequest(c.clone())),
        );

        for key in chunk_map.chunk_entities.keys() {
            let delta = *key - pos;
            let entity = chunk_map.get_entity(key).unwrap();
            if delta.x.abs().pow(2) + delta.y.abs().pow(2) > config.render_distance.pow(2) {
                despawn_requests.send(ChunkDespawnRequest(key.clone(), entity));
            }
        }
    }
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    mut spawn_events: EventReader<ChunkSpawnRequest>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_map: ChunkMapWriter,
) {
    for creation_request in spawn_events.iter() {
        let entity = commands
            .spawn_bundle(ChunkDataBundle {
                transform: Transform::from_translation(chunk2global(creation_request.0)),
                chunk_info: ChunkInfo {
                    pos: creation_request.0,
                },
                mesh_info: ChunkMeshInfo {
                    fluid_mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                    chunk_mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                },
                global_transform: Default::default(),
                rigid_body: RigidBody::Static,
                collision_shape: CollisionShape::Sphere { radius: 16.0 },
            })
            .insert(ChunkLoadState::LoadRequested)
            .id();

        chunk_map.insert_entity(creation_request.0, entity);
        chunk_map.chunk_data.insert(
            creation_request.0,
            Array3x1::fill(chunk_extent().padded(1), Default::default()),
        );
    }
}

//todo: parallelize this.
//todo: run this on the IOTaskPool
/// Loads from disk the chunk data of chunks with a current load state of [`ChunkLoadState::Load`].
/// If the chunk wasn't generated, the [`ChunkLoadState`] of the chunk is set to [`ChunkLoadState::Generate`].
pub(crate) fn load_chunk_data(
    mut chunks: Query<(&mut ChunkLoadState, Entity), Added<ChunkInfo>>,
    mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
) {
    for (mut load_state, entity) in chunks.iter_mut() {
        match *load_state {
            ChunkLoadState::LoadRequested => {
                *load_state = ChunkLoadState::Generate;
                gen_requests.push_front(ChunkLoadRequest(entity));
            }
            _ => continue,
        }
    }
}

/// Marks the load state of all chunk that are queued to be unloaded as [`ChunkLoadState::Unload`]
pub(crate) fn prepare_for_unload(
    mut despawn_events: EventReader<ChunkDespawnRequest>,
    mut chunks: Query<&mut ChunkLoadState>,
) {
    for despawn_event in despawn_events.iter() {
        if let Ok(mut load_state) = chunks.get_mut(despawn_event.1) {
            *load_state = ChunkLoadState::Unload;
        }
    }
}

/// Destroys all the chunks that have a load state of [`ChunkLoadState::Unload`]
pub(crate) fn destroy_chunks(
    mut commands: Commands,
    mut chunk_map: ChunkMapWriter,
    chunks: Query<(&ChunkInfo, &ChunkLoadState), Changed<ChunkLoadState>>,
) {
    for (chunk, load_state) in chunks.iter() {
        match load_state {
            ChunkLoadState::Unload => {
                let entity = chunk_map.remove_entity(&chunk.pos);
                chunk_map.chunk_data.remove(&chunk.pos);
                commands.entity(entity).despawn_recursive();
            }
            _ => {}
        }
    }
}

pub(crate) fn generate_chunks(
    mut query: Query<(&ChunkInfo, &mut ChunkLoadState)>,
    mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
    mut chunk_map: ChunkMapWriter,
    gen: Res<Arc<NoiseTerrainGenerator>>,
    task_pool: Res<WorldTaskPool>,
) {
    let chunks = task_pool.scope(|scope| {
        for req in gen_requests.drain(..) {
            if let Ok(info) = query.get_component::<ChunkInfo>(req.0) {
                let generator = gen.clone();
                scope.spawn(async move {
                    let mut data = Array3x1::fill(chunk_extent().padded(1), Default::default());
                    generator.generate(info.pos, &mut data);
                    (req.0, data)
                });
            }
        }
    });

    for (entity, chunk_data) in chunks {
        if let Ok((info, mut load_state)) = query.get_mut(entity) {
            chunk_map.chunk_data.insert(info.pos, chunk_data);
            *load_state = ChunkLoadState::Loading;
        }
    }
}

pub(crate) fn mark_chunks_ready(
    mut ready_events: EventWriter<ChunkReadyEvent>,
    chunks: Query<(&ChunkInfo, &ChunkLoadState, Entity), Changed<ChunkLoadState>>,
) {
    for (chunk, load_state, entity) in chunks.iter() {
        match load_state {
            ChunkLoadState::Idle => ready_events.send(ChunkReadyEvent(chunk.pos, entity)),
            _ => {}
        }
    }
}
