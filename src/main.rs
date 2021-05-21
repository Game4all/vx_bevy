use bevy::prelude::*;
use vx_core::{player::PlayerController, CorePlugins};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugins(CorePlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands
) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PlayerController::default());
}
