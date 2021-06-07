use bevy::prelude::*;
use heron::prelude::*;

pub mod config;
pub mod platform;
pub mod utils;
pub mod voxel;
pub mod world;

pub struct Player;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(platform::PlatformPlugin)
            .add(config::ConfigurationPlugin)
            .add(PhysicsPlugin::default())
            .add(world::WorldSimulationPlugin);
    }
}
