use bevy::prelude::*;

const ILLUMINANCE_BASE: f32 = 4_000.0;

//todo: have dynamic directional sunlight.
fn setup_sun_lighting(mut commands: Commands) {
    commands
        .spawn()
        .insert(DirectionalLight::new(
            Color::WHITE,
            ILLUMINANCE_BASE,
            -Vec3::Y,
        ))
        .insert(Transform::default())
        .insert(GlobalTransform::default());
}

#[allow(dead_code)]
fn update_sun_lighting_intensity(mut query: Query<&mut DirectionalLight>, time: Res<Time>) {
    let mut light = query.single_mut();
    let illuminance = (time.time_since_startup().as_secs_f32() * 0.1).cos() * ILLUMINANCE_BASE;
    if illuminance > 0f32 {
        light.illuminance = illuminance;
    }
}

pub struct EnvLightingPlugin;

impl Plugin for EnvLightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_sun_lighting.system());
        //.add_system(update_sun_lighting_intensity.system());
    }
}
