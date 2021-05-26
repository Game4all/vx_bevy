use bevy::prelude::*;

pub mod input;
pub mod player;
pub mod render;

pub struct ClientPlugins;

impl PluginGroup for ClientPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(input::PlayerInputPlugin)
            .add(player::PlayerControllerPlugin)
            .add(render::WorldRenderPlugin);
    }
}
