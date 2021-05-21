use bevy::prelude::PluginGroup;

pub mod player;
pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group.add(player::PlayerControllerPlugin);
    }
}
