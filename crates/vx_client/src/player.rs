use bevy::{input::mouse::MouseMotion, prelude::*};

use std::f32::consts::FRAC_PI_2;

pub const CAMERA_SENS: f32 = 0.0005;

#[derive(Default)]
pub struct PlayerController {
    yaw: f32,
    pitch: f32,
    cursor_locked: bool,
}

pub fn handle_player_mouse_move(
    mut query: Query<(&mut PlayerController, &mut Transform)>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut window: ResMut<Windows>,
) {
    for (mut controller, mut transform) in query.single_mut() {
        let mut delta = Vec2::ZERO;

        if controller.cursor_locked {
            for mouse_move in mouse_motion_event_reader.iter() {
                delta += mouse_move.delta;
            }
        }

        let first_win = window.get_primary_mut().unwrap();
        first_win.set_cursor_visibility(!controller.cursor_locked);
        first_win.set_cursor_lock_mode(controller.cursor_locked);
        if controller.cursor_locked {
            first_win.set_cursor_position((first_win.width() / 2., first_win.height() / 2.).into());
        }

        let mut new_pitch = controller.pitch + delta.y * CAMERA_SENS;
        let new_yaw = controller.yaw - delta.x * CAMERA_SENS;

        new_pitch = new_pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        controller.yaw = new_yaw;
        controller.pitch = new_pitch;

        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, new_yaw) * Quat::from_axis_angle(-Vec3::X, new_pitch);
    }
}

pub fn handle_player_input(
    mut query: Query<(&mut PlayerController, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (mut controller, mut transform) in query.single_mut() {
        if keyboard.just_pressed(KeyCode::Escape) {
            controller.cursor_locked = !controller.cursor_locked;
        }

        let mut direction = Vec3::ZERO;

        let forward = transform.rotation.mul_vec3(Vec3::Z).normalize();
        let right = transform.rotation.mul_vec3(Vec3::X).normalize();

        if keyboard.pressed(KeyCode::Z) {
            direction.z -= 1.0;
        }

        if keyboard.pressed(KeyCode::S) {
            direction.z += 1.0;
        }

        if keyboard.pressed(KeyCode::D) {
            direction.x += 1.0;
        }

        if keyboard.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }

        transform.translation += direction.x * right + direction.z * forward;
    }
}

//todo: move to client-side plugins whens 'the split' happens
pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(handle_player_mouse_move.system())
            .add_system(handle_player_input.system())
            .add_system(handle_player_mouse_move.system());
    }
}
