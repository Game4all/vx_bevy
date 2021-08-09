use bevy::prelude::*;

pub mod config;
pub mod platform;
pub mod utils;
pub mod voxel;
pub mod world;
pub mod worldgen;

pub struct Player;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(platform::PlatformPlugin)
            .add(config::ConfigurationPlugin)
            .add(world::WorldSimulationPlugin);
    }
}
