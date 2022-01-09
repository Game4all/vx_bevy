use bevy::prelude::*;

pub mod config;
pub mod platform;
pub mod utils;
pub mod voxel;
pub mod world;
pub mod worldgen;

use utils::ConfigurationPlugin;

#[derive(Component)]
pub struct Player;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(platform::PlatformPlugin)
            .add(ConfigurationPlugin::<config::GlobalConfig>::default())
            .add(world::WorldSimulationPlugin)
            .add(worldgen::WorldGenerationPlugin);
    }
}
