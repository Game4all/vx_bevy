use std::f32::consts::PI;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

mod input;
mod voxel;

use input::PlayerController;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::PlayerControllerPlugin)
        .add_plugin(voxel::VoxelWorldPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin::filtered(vec![
            FrameTimeDiagnosticsPlugin::FPS,
        ]))
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
    .insert(PlayerController::default())
    .insert(voxel::Player);
}
