use super::{
    chunk_extent, ChunkDataBundle, ChunkDespawnRequest, ChunkInfo, ChunkLoadRequest,
    ChunkLoadState, ChunkMapReader, ChunkMapWriter, ChunkReadyEvent, ChunkSpawnRequest,
    WorldChunkIndexer, WorldTaskPool, CHUNK_DATA_GEN_TIME, CHUNK_LENGTH, MAX_FRAME_CHUNK_GEN_COUNT,
};
use crate::{config::GlobalConfig, worldgen, Player};
use bevy::{diagnostic::Diagnostics, ecs::schedule::ShouldRun, prelude::*, utils::Instant};
use building_blocks::{
    core::{IntoIntegerPoint, Point3i, PointN},
    storage::{Array3x1, ChunkKey3},
};
use std::collections::VecDeque;

/// Handles the visibility checking of the currently loaded chunks around the player.
/// This will accordingly emit [`ChunkSpawnRequest`] events for chunks that need to be loaded since they entered the player's view distance and [`ChunkDespawnRequest`] for
/// chunks out of the player's view distance.
pub(crate) fn update_visible_chunks(
    player: Query<&GlobalTransform, (Changed<GlobalTransform>, With<Player>)>,
    chunk_map: ChunkMapReader,
    config: Res<GlobalConfig>,
    indexer: Res<WorldChunkIndexer>,
    mut load_radius_chunks: bevy::ecs::system::Local<Vec<Point3i>>,
    mut spawn_requests: EventWriter<ChunkSpawnRequest>,
    mut despawn_requests: EventWriter<ChunkDespawnRequest>,
) {
    if let Ok(transform) = player.single() {
        let pos = indexer
            .min_of_chunk_containing_point(PointN(transform.translation.to_array()).into_int());

        for dx in -config.render_distance..=config.render_distance {
            for dz in -config.render_distance..=config.render_distance {
                for dy in -config.render_distance..=config.render_distance {
                    if dx.pow(2) + dy.pow(2) + dz.pow(2) >= config.render_distance.pow(2) {
                        continue;
                    };

                    let chunk_pos =
                        pos + PointN([dx * CHUNK_LENGTH, dy * CHUNK_LENGTH, dz * CHUNK_LENGTH]);

                    if !chunk_map.chunk_exists(chunk_pos) {
                        load_radius_chunks.push(chunk_pos);
                    }
                }
            }
        }

        load_radius_chunks.sort_by_key(|a| (a.x().pow(2) + a.z().pow(2)));

        spawn_requests.send_batch(
            load_radius_chunks
                .drain(..)
                .map(|c| ChunkSpawnRequest(c.clone())),
        );

        for key in chunk_map.chunk_entities.keys() {
            let delta = *key - pos;
            let entity = chunk_map.get_entity(*key).unwrap();
            if delta.x().abs().pow(2) + delta.y().abs().pow(2) + delta.z().abs().pow(2)
                > (config.render_distance * CHUNK_LENGTH).pow(2)
            {
                despawn_requests.send(ChunkDespawnRequest(entity));
            }
        }
    }
}

pub(crate) fn update_visible_chunks_run_criteria(
    indexer: Res<WorldChunkIndexer>,
    player: Query<&GlobalTransform, (Changed<GlobalTransform>, With<Player>)>,
    mut previous_pos: bevy::ecs::system::Local<Point3i>,
) -> ShouldRun {
    for transform in player.iter() {
        let new_pos = indexer
            .min_of_chunk_containing_point(PointN(transform.translation.to_array()).into_int());

        if *previous_pos != new_pos {
            *previous_pos = new_pos;
            return ShouldRun::Yes;
        }
    }
    ShouldRun::No
}

pub(crate) fn create_chunks(
    mut commands: Commands,
    mut spawn_events: EventReader<ChunkSpawnRequest>,
    mut chunk_map: ChunkMapWriter,
) {
    for creation_request in spawn_events.iter() {
        let entity = commands
            .spawn_bundle(ChunkDataBundle {
                transform: Transform::from_xyz(
                    creation_request.0.x() as f32,
                    creation_request.0.y() as f32,
                    creation_request.0.z() as f32,
                ),
                chunk_info: ChunkInfo {
                    pos: creation_request.0,
                },
                global_transform: Default::default(),
            })
            .insert(ChunkLoadState::LoadRequested)
            .id();

        chunk_map.chunk_entities.insert(creation_request.0, entity);
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
    chunks.for_each_mut(|(mut load_state, entity)| match *load_state {
        ChunkLoadState::LoadRequested => {
            *load_state = ChunkLoadState::Generate;
            gen_requests.push_back(ChunkLoadRequest(entity));
        }
        _ => {}
    });
}

/// Marks the load state of all chunk that are queued to be unloaded as [`ChunkLoadState::Unload`]
pub(crate) fn prepare_for_unload(
    mut despawn_events: EventReader<ChunkDespawnRequest>,
    mut chunks: Query<&mut ChunkLoadState>,
) {
    for despawn_event in despawn_events.iter() {
        if let Ok(mut load_state) = chunks.get_mut(despawn_event.0) {
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
    chunks.for_each(|(chunk, load_state)| match load_state {
        ChunkLoadState::Unload => {
            let entity = chunk_map
                .chunk_entities
                .remove(&chunk.pos)
                .expect("Expected valid chunk");
            chunk_map.chunk_data.pop_chunk(ChunkKey3::new(0, chunk.pos));
            commands.entity(entity).despawn_recursive();
        }
        _ => {}
    });
}

pub(crate) fn mark_chunks_ready(
    mut ready_events: EventWriter<ChunkReadyEvent>,
    chunks: Query<(&ChunkInfo, &ChunkLoadState, Entity), Changed<ChunkLoadState>>,
) {
    chunks.for_each(|(chunk, load_state, entity)| match load_state {
        ChunkLoadState::Idle => ready_events.send(ChunkReadyEvent(chunk.pos, entity)),
        _ => {}
    });
}

pub(crate) fn generate_terrain_data(
    mut query: Query<(&ChunkInfo, &mut ChunkLoadState)>,
    mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
    mut chunk_map: ChunkMapWriter,
    task_pool: Res<WorldTaskPool>,
    mut diagnostics: ResMut<Diagnostics>,
) {
    let time_before_loading = Instant::now();

    let chunks = task_pool.scope(|scope| {
        let gen_req_count = gen_requests.len().min(MAX_FRAME_CHUNK_GEN_COUNT);
        for req in gen_requests.drain(..gen_req_count) {
            if let Ok(info) = query.get_component::<ChunkInfo>(req.0) {
                scope.spawn(async move {
                    let mut data = Array3x1::fill(chunk_extent(), Default::default());
                    worldgen::gen_terrain_for_chunk(info.pos, &mut data);
                    (req.0, data)
                });
            }
        }
    });

    for (entity, chunk_data) in chunks {
        if let Ok((info, mut load_state)) = query.get_mut(entity) {
            chunk_map
                .chunk_data
                .write_chunk(ChunkKey3::new(0, info.pos), chunk_data);
            *load_state = ChunkLoadState::Idle;
        }
    }

    let time_after_loading = Instant::now() - time_before_loading;
    diagnostics.add_measurement(CHUNK_DATA_GEN_TIME, time_after_loading.as_secs_f64());
}
