use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use vx_core::utils::ConfigurationPlugin;

pub mod debug;
pub mod input;
pub mod player;
pub mod render;
pub mod utils;

pub struct ClientPlugins;

impl PluginGroup for ClientPlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(FrameTimeDiagnosticsPlugin::default())
            .add(ConfigurationPlugin::<input::Keybindings>::default())
            .add(input::PlayerInputPlugin)
            .add(player::PlayerControllerPlugin)
            .add(render::WorldRenderPlugin)
            .add(render::MeshCullingPlugin)
            .add(render::EnvLightingPlugin)
            .add(debug::DebugUIPlugin);
    }
}
