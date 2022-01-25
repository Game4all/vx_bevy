use std::f32::consts::PI;

use bevy::prelude::*;

mod input;
mod voxel;

use input::PlayerController;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugin(input::PlayerControllerPlugin)
        .add_plugin(voxel::VoxelWorldPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut cmds: Commands) {
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
