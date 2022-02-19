use bevy::{
    math::UVec4,
    prelude::{info, Plugin},
    utils::HashMap,
};
use std::{any::type_name, any::TypeId};

// Registry info about a voxel material
pub struct MaterialRegistryInfo {
    pub name: &'static str,
    pub base_color: UVec4,
}

/// A registry for voxel material types.
/// This stores the voxel materials along their material id used to refer them in voxel data
pub struct VoxelMaterialRegistry {
    materials: Vec<MaterialRegistryInfo>,
    mat_ids: HashMap<TypeId, usize>,
}

impl VoxelMaterialRegistry {
    #[inline]
    pub fn get_by_id(&self, id: u8) -> Option<&MaterialRegistryInfo> {
        self.materials.get(id as usize)
    }

    pub fn get_by_type<M: 'static>(&self) -> Option<&MaterialRegistryInfo> {
        self.mat_ids
            .get(&TypeId::of::<M>())
            .map(|x| self.materials.get(*x).unwrap())
    }

    pub fn get_id_for_type<M: 'static>(&self) -> Option<u8> {
        self.mat_ids.get(&TypeId::of::<M>()).map(|x| *x as u8)
    }

    pub fn register_material<M: 'static>(&mut self, mat: MaterialRegistryInfo) {
        self.materials.push(mat);
        info!(
            "Registered material {:?} (ID: {})",
            type_name::<M>(),
            self.materials.len() - 1
        );
        self.mat_ids.insert(TypeId::of::<M>(), self.materials.len());
    }
}

impl Default for VoxelMaterialRegistry {
    fn default() -> Self {
        let mut registry = Self {
            materials: Default::default(),
            mat_ids: Default::default(),
        };

        registry.register_material::<Void>(MaterialRegistryInfo {
            base_color: UVec4::ZERO,
            name: "Void",
        });

        registry
    }
}

// The material with ID #0;
pub struct Void;

pub struct VoxelMaterialPlugin;
impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<VoxelMaterialRegistry>();
    }
}
