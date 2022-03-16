use bevy::prelude::{Color, Plugin};

use crate::{
    define_voxel_material,
    voxel::material::{MaterialRegistryInfo, VoxelMaterialFlags, VoxelMaterialRegistry},
};

define_voxel_material!(Dirt, "Dirt", 1);
define_voxel_material!(Sand, "Sand", 2);
define_voxel_material!(Grass, "Grass", 3);
define_voxel_material!(Rock, "Rock", 4);
define_voxel_material!(Snow, "Snow", 5);
define_voxel_material!(Water, "Water", 6);

pub struct VoxelWorldBaseMaterialsPlugin;

impl Plugin for VoxelWorldBaseMaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut registry = app
            .world
            .get_resource_mut::<VoxelMaterialRegistry>()
            .unwrap();

        registry.register_material::<Dirt>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(112, 97, 92),
            name: Dirt::NAME,
            flags: VoxelMaterialFlags::SOLID,
        });

        registry.register_material::<Sand>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(228, 219, 148),
            name: Sand::NAME,
            flags: VoxelMaterialFlags::SOLID,
        });

        registry.register_material::<Grass>(MaterialRegistryInfo {
            base_color: Color::LIME_GREEN,
            name: Grass::NAME,
            flags: VoxelMaterialFlags::SOLID,
        });

        registry.register_material::<Rock>(MaterialRegistryInfo {
            base_color: Color::GRAY,
            name: Rock::NAME,
            flags: VoxelMaterialFlags::SOLID,
        });

        registry.register_material::<Snow>(MaterialRegistryInfo {
            base_color: Color::WHITE,
            name: Snow::NAME,
            flags: VoxelMaterialFlags::SOLID,
        });

        registry.register_material::<Water>(MaterialRegistryInfo {
            base_color: *Color::rgb_u8(106, 235, 187).set_a(0.4),
            name: Water::NAME,
            flags: VoxelMaterialFlags::LIQUID,
        });
    }
}
