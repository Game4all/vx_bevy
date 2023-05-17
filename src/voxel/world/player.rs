use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_egui::EguiContexts;
use std::f32::consts::FRAC_PI_2;

use crate::debug::DebugUISet;

use super::chunks::LoadChunksAround;

const BODY_ROTATION_SLERP: f32 = 0.5;

#[derive(Component)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct BodyBundle<M: Material> {
    pub material_mesh_bundle: MaterialMeshBundle<M>,

    // defaults are fine for these:
    pub body: Body,
    pub load_chunks_around: LoadChunksAround,
}

/// Marker component for player body.
#[derive(Component, Default)]
pub struct Body;

#[derive(Component)]
pub struct Head;

#[derive(Component, Debug, Clone, Copy)]
pub enum CameraMode {
    FirstPerson,
    ThirdPersonForward,
}

impl CameraMode {
    const fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPersonForward,
            Self::ThirdPersonForward => Self::FirstPerson,
        }
    }

    fn translation(self) -> Vec3 {
        match self {
            Self::FirstPerson => Vec3::ZERO,
            Self::ThirdPersonForward => Vec3::Z * -5.0,
        }
    }
}

// Reusing the player controller impl for now.

pub const DEFAULT_CAMERA_SENS: f32 = 0.005;

fn handle_player_mouse_move(
    mut head: Query<&mut Transform, With<Head>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let mut head_transform = head.single_mut();
    let mut delta = Vec2::ZERO;

    for mouse_move in mouse_motion_event_reader.iter() {
        delta -= mouse_move.delta;
    }

    if !matches!(window.cursor.grab_mode, CursorGrabMode::Locked) {
        return;
    }

    let (yaw, pitch, _roll) = head_transform.rotation.to_euler(EulerRot::YXZ);
    let yaw = delta.x.mul_add(DEFAULT_CAMERA_SENS, yaw);
    let pitch = delta
        .y
        .mul_add(-DEFAULT_CAMERA_SENS, pitch)
        // ensure that the look direction always has a component in the xz plane:
        .clamp(-FRAC_PI_2 + 1e-5, FRAC_PI_2 - 1e-5);
    head_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}

fn handle_player_keyboard_input(
    mut egui: EguiContexts,
    // mut queries: ParamSet<Query<&mut Transform, With<Body>>>,
    mut queries: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<&Transform, With<Body>>,
    )>,
    keys: Res<Input<KeyCode>>,
    btns: Res<Input<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    // cursor grabbing
    if btns.just_pressed(MouseButton::Left) && !egui.ctx_mut().wants_pointer_input() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    // cursor ungrabbing
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }

    let (forward, right) = {
        let body = queries.p1();
        let body_transform = body.single();
        let forward = body_transform.rotation.mul_vec3(Vec3::Z).normalize();
        let right = Vec3::Y.cross(forward); // @todo(meyerzinn): not sure why this is the correct orientation
        (forward, right)
    };

    let mut body = queries.p0();
    let mut body_transform = body.single_mut();

    let mut direction = Vec3::ZERO;
    let mut acceleration = 1.0f32;

    if keys.pressed(KeyCode::W) {
        direction.z -= 1.0;
    }

    if keys.pressed(KeyCode::S) {
        direction.z += 1.0;
    }

    if keys.pressed(KeyCode::D) {
        direction.x += 1.0;
    }

    if keys.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }

    if keys.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }

    if keys.pressed(KeyCode::LShift) {
        direction.y -= 1.0;
    }

    if keys.pressed(KeyCode::LControl) {
        acceleration *= 8.0;
    }

    if direction == Vec3::ZERO {
        return;
    }

    // hardcoding 0.10 as a factor for now to not go zoomin across the world.
    body_transform.translation += direction.x * right * acceleration
        + direction.z * forward * acceleration
        + direction.y * Vec3::Y * acceleration;
}

fn handle_player_change_camera_mode(
    keys: Res<Input<KeyCode>>,
    mut cameras: Query<(&mut CameraMode, &mut Transform)>,
) {
    if keys.just_pressed(KeyCode::F5) {
        let (mut mode, mut transform) = cameras.single_mut();
        *mode = mode.next();
        transform.translation = mode.translation();
    }
}

fn update_player_body_rotation(
    mut queries: ParamSet<(
        Query<&mut Transform, With<Body>>,
        Query<&Transform, With<Head>>,
    )>,
) {
    let yaw = {
        let head = queries.p1();
        let (yaw, _pitch, _roll) = head.single().rotation.to_euler(EulerRot::YXZ);
        yaw
    };
    let mut body = queries.p0();
    let mut body_transform = body.single_mut();
    let desired = Quat::from_euler(EulerRot::YXZ, yaw, 0., 0.);
    body_transform.rotation = body_transform.rotation.slerp(desired, BODY_ROTATION_SLERP);
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug, SystemSet)]
/// Systems related to player controls.
pub struct PlayerControllerSet;

pub struct VoxelWorldPlayerControllerPlugin;

impl Plugin for VoxelWorldPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                handle_player_mouse_move,
                update_player_body_rotation,
                handle_player_keyboard_input,
                handle_player_change_camera_mode,
            )
                .chain()
                .in_base_set(CoreSet::Update)
                .after(DebugUISet::Display),
        );
    }
}
