use std::f32::consts::PI;

use bevy::prelude::*;

mod input;

use input::{PlayerController, PlayerControllerPlugin};

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::PlayerControllerPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut cmds: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    cmds.spawn_bundle(PerspectiveCameraBundle {
        perspective_projection: PerspectiveProjection {
            fov: PI / 2.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(PlayerController::default());

    cmds.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Box::new(0.5, 0.5, 0.5).into()),
        transform: Transform::default(),
        ..Default::default()
    });
}
