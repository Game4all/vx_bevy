use super::{chunks::ChunkLoadingSystem, Chunk};
use bevy::prelude::*;

/// Attaches to the newly inserted chunk entities components required for rendering.
pub fn prepare_chunks(
    chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cmds: Commands,
) {
    for (chunk, chunk_key) in chunks.iter() {
        cmds.entity(chunk).insert_bundle(PbrBundle {
            mesh: meshes.add(shape::Box::new(16.0, 16.0, 16.0).into()),
            transform: Transform::from_translation(chunk_key.0.location().as_vec3()),
            ..Default::default()
        });
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
pub enum ChunkRenderingSystem {
    /// Attaches to the newly inserted chunk entities components required for rendering.
    Prepare,
}

/// Handles the rendering of the chunks.
pub struct VoxelWorldRenderingPlugin;

impl Plugin for VoxelWorldRenderingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(
            prepare_chunks
                .label(ChunkRenderingSystem::Prepare)
                .after(ChunkLoadingSystem::CreateChunks),
        );
    }
}
