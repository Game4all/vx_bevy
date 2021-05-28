use bevy::prelude::*;

pub mod platform;
pub mod voxel;
pub mod world;
pub mod config;

pub struct Player;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(platform::PlatformPlugin)
            .add(world::WorldSimulationPlugin);
    }
}
