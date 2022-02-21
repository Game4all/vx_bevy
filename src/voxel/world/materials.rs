use bevy::{math::{UVec4, UVec2}, prelude::Plugin};

use crate::voxel::material::{MaterialRegistryInfo, VoxelMaterialRegistry};

pub struct Dirt;

impl Dirt {
    pub const ID: u8 = 1;
    pub const NAME: &'static str = "Dirt";
}

pub struct Sand;
impl Sand {
    pub const ID: u8 = 2;
    pub const NAME: &'static str = "Sand";
}

pub struct Grass;
impl Grass {
    pub const ID: u8 = 3;
    pub const NAME: &'static str = "Grass";
}

pub struct Rock;
impl Rock {
    pub const ID: u8 = 4;
    pub const NAME: &'static str = "Rock";
}

pub struct VoxelWorldBaseMaterialsPlugin;

impl Plugin for VoxelWorldBaseMaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut registry = app
            .world
            .get_resource_mut::<VoxelMaterialRegistry>()
            .unwrap();

        registry.register_material::<Dirt>(MaterialRegistryInfo {
            base_color: UVec4::ZERO,
            name: Dirt::NAME,
        });

        registry.register_material::<Sand>(MaterialRegistryInfo {
            base_color: UVec4::ZERO,
            name: Sand::NAME,
        });

        registry.register_material::<Grass>(MaterialRegistryInfo {
            base_color: UVec4::new(104, 188, 68, 0),
            name: Grass::NAME,
        });

        registry.register_material::<Rock>(MaterialRegistryInfo {
            base_color: UVec4::ZERO,
            name: Rock::NAME,
        });
    }
}
