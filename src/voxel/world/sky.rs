use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_fixed_timer;
use bevy::time::Time;
use bevy_atmosphere::prelude::*;

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    time: Res<Time>,
) {
    let t = time.elapsed_seconds_wrapped() as f32 / 2.0;
    atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Quat::from_rotation_x(-t);
        directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }
}

// Simple environment
fn setup_environment(mut commands: Commands) {
    // Our Sun
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                illuminance: 100000.0,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
        Sun, // Marks the light as Sun
    ));
}

pub struct InteractiveSkyboxPlugin;

impl Plugin for InteractiveSkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AtmosphereModel::default())
            .add_startup_system(setup_environment)
            .add_system(
                daylight_cycle
                    .in_schedule(CoreSchedule::FixedUpdate)
                    .run_if(on_fixed_timer(Duration::from_millis(50))),
            );
    }
}
