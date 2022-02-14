use bevy::{
    core::Time,
    prelude::{
        Added, Changed, Commands, Component, Entity, Plugin, Query, Res, Transform, Visibility,
        With,
    },
};

use crate::voxel::render::VoxelMesh;

use super::Chunk;

const ANIMATION_DURATION: f32 = 0.8;
const ANIMATION_HEIGHT: f32 = 128.;

#[derive(Component)]
pub struct ChunkSpawnAnimation {
    start_time: f32,
}

fn attach_chunk_animation(
    ready_chunks: Query<Entity, (Added<VoxelMesh>, With<Chunk>, Changed<Visibility>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    ready_chunks.for_each(|entity| {
        commands.entity(entity).insert(ChunkSpawnAnimation {
            start_time: time.time_since_startup().as_secs_f32(),
        });
    });
}

/// Steps the chunk animation by one frame.
fn step_chunk_animation(
    mut chunks: Query<(Entity, &mut Transform, &Chunk, &ChunkSpawnAnimation)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    chunks.for_each_mut(|(entity, mut transform, _chunk, animation)| {
        let delta = (time.time_since_startup().as_secs_f32() - animation.start_time)
            .min(ANIMATION_DURATION);

        let ytransform = _chunk.0.location().y as f32 - ANIMATION_HEIGHT
            + (1. - (1. - (delta / ANIMATION_DURATION)).powi(5)) * ANIMATION_HEIGHT;

        transform.translation.y = ytransform;

        if delta == ANIMATION_DURATION {
            commands.entity(entity).remove::<ChunkSpawnAnimation>();
        }
    });
}

/// Animates the spawning of chunk entities that come into sight.
pub struct ChunkAppearanceAnimatorPlugin;

impl Plugin for ChunkAppearanceAnimatorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(step_chunk_animation)
            .add_system(attach_chunk_animation);
    }
}
