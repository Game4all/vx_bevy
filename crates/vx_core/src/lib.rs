use bevy::prelude::*;

pub mod world;
pub mod meshing;

pub struct Player;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(world::WorldSimulationPlugin);
    }
}
