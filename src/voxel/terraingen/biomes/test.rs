use bevy::math::{UVec3, Vec2, Vec3Swizzles};

use crate::voxel::{
    material::VoxelMaterial,
    materials::{Cactus, Dirt, Grass, Sand, Sandstone, Snow},
    storage::VoxelBuffer,
    terraingen::{common::make_tree, noise},
    ChunkKey, ChunkShape, Voxel, CHUNK_LENGTH,
};

use super::LayeredBiomeTerrainGenerator;

pub struct BasicDesertBiomeTerrainGenerator;

impl LayeredBiomeTerrainGenerator for BasicDesertBiomeTerrainGenerator {
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0..=5 => Sand::into_voxel(),
            _ => Sandstone::into_voxel(),
        }
    }

    fn place_decoration(
        &self,
        key: ChunkKey,
        pos: UVec3,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        let cacti_spawn_chance = noise::rand2to1(
            (pos.xz().as_vec2() + key.location().xz().as_vec2()) * 0.1,
            Vec2::new(12.989, 78.233),
        );

        if cacti_spawn_chance > 0.992 {
            let size = ((cacti_spawn_chance - 0.992) * 2000.0) as u32;
            for h in pos.y..(pos.y + size).min(CHUNK_LENGTH) {
                *buffer.voxel_at_mut([pos.x, h, pos.z].into()) = Cactus::into_voxel();
            }
        }
    }
}

pub struct BasicPlainsBiomeTerrainGenerator;

impl LayeredBiomeTerrainGenerator for BasicPlainsBiomeTerrainGenerator {
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0..=1 => Grass::into_voxel(),
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
            (pos.xz().as_vec2() + key.location().xz().as_vec2()) * 0.1,
            Vec2::new(12.989, 78.233),
        );

        if spawn_chance > 0.981 {
            if pos.y <= 16 || pos.x > 8 && pos.x < 23 {
                make_tree(buffer, pos);
            }
        }
    }
}

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
        _key: ChunkKey,
        _pos: UVec3,
        _buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
    }
}
