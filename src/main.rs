use bevy::prelude::*;
use vx_client::{player::PlayerController, ClientPlugins};
use vx_core::{CorePlugins, Player};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugins)
        .add_plugins(ClientPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0., 150.0, 0.0),
            ..Default::default()
        })
        .insert(Player)
        .insert(PlayerController::default());
}
