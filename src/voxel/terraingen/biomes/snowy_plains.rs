use bevy::math::{UVec3, Vec2, Vec3Swizzles};

use crate::voxel::{
    materials::{Dirt, Grass, Snow, Wood},
    storage::VoxelBuffer,
    terraingen::{common::make_tree, noise},
    ChunkKey, ChunkShape, Voxel, material::VoxelMaterial,
};

use super::LayeredBiomeTerrainGenerator;

pub struct BasicSnowyPlainsBiomeTerrainGenerator;

impl LayeredBiomeTerrainGenerator for BasicSnowyPlainsBiomeTerrainGenerator {
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0 => Snow::into_voxel(),
            1..=2 => Grass::into_voxel(),
            _ => Dirt::into_voxel(),
        }
    }

    fn place_decoration(
        &self,
        key: ChunkKey,
        pos: UVec3,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        let spawn_chance = noise::rand2to1(
            (pos.xz().as_vec2() + key.xz().as_vec2()) * 0.1,
            Vec2::new(12.989, 78.233),
        );

        if spawn_chance > 0.981 {
            if pos.y <= 13 {
                // this is a stupid hack but a real fix would be to allow terrain decoration to work vertically
                make_tree::<Wood, Snow>(buffer, pos);
            }
        }
    }
}
