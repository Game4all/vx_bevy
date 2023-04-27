#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]

use std::f32::consts::PI;

use bevy::{
    core_pipeline::fxaa::Fxaa,
    prelude::*, ecs::schedule::{ScheduleBuildSettings, LogLevel},
};

mod debug;
mod voxel;

fn main() {
    let mut app = App::default();

    app.edit_schedule(CoreSchedule::Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            report_sets: true,
            hierarchy_detection: LogLevel::Warn,
            use_shortnames: true,
        });
    });

    app.add_plugins(DefaultPlugins)
        .add_plugin(voxel::VoxelWorldPlugin)
        .add_plugin(debug::DebugUIPlugins)
        .add_startup_system(setup);

    app.run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera3dBundle {
        projection: bevy::render::camera::Projection::Perspective(PerspectiveProjection {
            fov: PI / 2.,
            far: 2048.0,
            ..Default::default()
        }),
        transform: Transform::from_xyz(2.0, 160.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(voxel::player::PlayerController::default())
    .insert(Fxaa::default())
    .insert(bevy_atmosphere::plugin::AtmosphereCamera::default());

    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
}
