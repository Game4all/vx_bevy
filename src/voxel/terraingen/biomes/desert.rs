use bevy::math::{IVec3, UVec3, Vec2, Vec3, Vec3Swizzles};
use ilattice::prelude::Extent;

use crate::voxel::{
    material::VoxelMaterial,
    materials::{Cactus, Sand, Sandstone},
    sdf,
    storage::VoxelBuffer,
    terraingen::noise,
    ChunkShape, Voxel, CHUNK_LENGTH,
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
        key: IVec3,
        pos: UVec3,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        let cacti_spawn_chance = noise::rand2to1(
            (pos.xz().as_vec2() + key.xz().as_vec2()) * 0.1,
            Vec2::new(12.989, 78.233),
        );

        if cacti_spawn_chance > 0.992 {
            let size = ((cacti_spawn_chance - 0.992) * 2000.0) as u32 + 2;
            make_cacti(buffer, pos, size);
        }
    }
}

fn make_cacti(buffer: &mut VoxelBuffer<Voxel, ChunkShape>, pos: UVec3, size: u32) {
    Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH))
        .iter3()
        .filter(|x| {
            sdf::sdf_v_capsule(x.as_vec3() - pos.as_vec3() - Vec3::Y, size as f32, 1.5) < 0.0
        })
        .for_each(|x| *buffer.voxel_at_mut(x) = Cactus::into_voxel());
}
