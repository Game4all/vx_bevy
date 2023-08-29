#![allow(
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::module_inception
)]

use std::f32::consts::PI;

use bevy::{core_pipeline::fxaa::Fxaa, prelude::*};
use voxel::player::{self, CameraMode};

mod debug;
mod voxel;

fn main() {
    let mut app = App::default();
    app.add_plugins(DefaultPlugins)
        .add_plugin(voxel::VoxelWorldPlugin)
        .add_plugin(debug::DebugUIPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*
    Lowercase denotes an entity, and uppercase denotes a component:

    player
    ├── Player
    ├── TransformBundle
    ├── VisibilityBundle
    ├── body
    │   └── BodyBundle
    └── head
        ├── Head (marker)
        ├── TransformBundle (offsets the head from the player)
        ├── VisibilityBundle (camera needs to be child of a visible entity)
        └── camera
            ├── AtmosphereCamera (cancels atmosphere translation)
            ├── Camera3dBundle
            ├── CameraMode (first or third person)
            ├── Fxaa
            └── TransformBundle (moves camera w.r.t. head orientation)
    */

    let visibility_bundle = || VisibilityBundle {
        visibility: Visibility::Visible,
        ..default()
    };

    let player_bundle = (
        player::Player,
        visibility_bundle(),
        TransformBundle {
            local: Transform::from_xyz(2.0, 170.0, 2.0).looking_to(Vec3::Z, Vec3::Y),
            ..default()
        },
    );

    let body_bundle = player::BodyBundle {
        material_mesh_bundle: MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.5, 1.8, 0.5))),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::IDENTITY.looking_to(Vec3::Z, Vec3::Y),
            ..default()
        },
        ..default()
    };

    let head_bundle = (
        player::Head,
        visibility_bundle(),
        TransformBundle {
            // head is 1.8m above feet or 0.9m above the center
            local: Transform::from_translation(Vec3::new(0.0, 0.9, 0.0))
                .looking_to(Vec3::Z, Vec3::Y),
            ..default()
        },
    );

    let camera_bundle = (
        CameraMode::ThirdPersonForward,
        visibility_bundle(),
        Camera3dBundle {
            projection: bevy::render::camera::Projection::Perspective(PerspectiveProjection {
                fov: PI / 2.,
                far: 2048.0,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
                .looking_to(Vec3::Z, Vec3::Y),
            ..default()
        },
        Fxaa::default(),
        bevy_atmosphere::plugin::AtmosphereCamera::default(),
    );

    cmds.spawn(player_bundle).with_children(|player| {
        player.spawn(body_bundle);

        player.spawn(head_bundle).with_children(|head| {
            // spawn camera as a child of head
            head.spawn(camera_bundle);
        });
    });

    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
}
