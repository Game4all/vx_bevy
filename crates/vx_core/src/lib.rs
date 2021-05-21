use bevy::prelude::PluginGroup;

pub mod player;
pub mod render;
pub mod world;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(player::PlayerControllerPlugin)
            .add(world::WorldSimulationPlugin)
            .add(render::WorldRenderPlugin);
    }
}
