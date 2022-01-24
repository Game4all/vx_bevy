use bevy::prelude::{Component, Plugin};
use ndshape::ConstShape3u32;

use std::hash::Hash;

use super::storage::{VoxelMap, VoxelMapKey};

/// Systems for dynamically loading / unloading regions (aka chunks) of the world according to camera position.
mod chunks;

/// Registers all resources and systems for simulating and rendering an editable and interactive voxel world.
pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(VoxelMap::<Voxel, ChunkShape>::new(ChunkShape {}))
            .add_plugin(chunks::VoxelWorldChunkingPlugin);
        //insert chunk loading systems
        //insert chunk voxel meshing systems
        //insert voxel pipeline
    }
}

/// Component tagging a player.
#[derive(Component)]
pub struct Player;

pub type ChunkKey = VoxelMapKey<Voxel>;
pub const CHUNK_LENGTH: u32 = 32;
pub type ChunkShape = ConstShape3u32<CHUNK_LENGTH, CHUNK_LENGTH, CHUNK_LENGTH>;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Voxel(u8);

pub const EMPTY_VOXEL: Voxel = Voxel(0);

impl Default for Voxel {
    fn default() -> Self {
        EMPTY_VOXEL
    }
}
