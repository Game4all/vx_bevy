use bevy::prelude::{Component, Plugin};
use ndshape::ConstShape3u32;

use super::{
    storage::{VoxelMap, VoxelMapKey},
    terraingen, Voxel,
};

/// Systems for dynamically loading / unloading regions (aka chunks) of the world according to camera position.
mod chunks;
pub use chunks::{
    ChunkCommandQueue, ChunkEntities, ChunkLoadRadius, CurrentLocalPlayerChunk, DirtyChunks,
};

mod chunks_anim;
pub mod materials;
mod meshing;
pub mod player;
mod terrain;

/// Registers all resources and systems for simulating and rendering an editable and interactive voxel world.
pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(VoxelMap::<Voxel, ChunkShape>::new(ChunkShape {}))
            .add_plugin(chunks::VoxelWorldChunkingPlugin)
            .add_plugin(meshing::VoxelWorldMeshingPlugin)
            // ordering of plugin insertion matters here.
            .add_plugin(terraingen::TerrainGeneratorPlugin)
            .add_plugin(terrain::VoxelWorldTerrainGenPlugin)
            .add_plugin(super::render::VoxelMeshRenderPipelinePlugin)
            .add_plugin(super::material::VoxelMaterialPlugin)
            .add_plugin(materials::VoxelWorldBaseMaterialsPlugin)
            .add_plugin(chunks_anim::ChunkAppearanceAnimatorPlugin)
            .add_plugin(bevy_atmosphere::AtmospherePlugin::default())
            .add_plugin(player::VoxelWorldPlayerControllerPlugin);
    }
}

pub type ChunkKey = VoxelMapKey<Voxel>;
pub const CHUNK_LENGTH: u32 = 32;
pub const CHUNK_LENGTH_U: usize = CHUNK_LENGTH as usize;
pub type ChunkShape = ConstShape3u32<CHUNK_LENGTH, CHUNK_LENGTH, CHUNK_LENGTH>;

// A component tagging an entity as a chunk.
#[derive(Component)]
pub struct Chunk(pub ChunkKey);
