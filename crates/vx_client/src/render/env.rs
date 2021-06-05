use bevy::prelude::*;

//todo: have dynamic directional sunlight.
fn setup_sun_lighting(mut commands: Commands) {
    commands
        .spawn()
        .insert(DirectionalLight::new(Color::WHITE, 4_000.0, -Vec3::Y))
        .insert(Transform::default())
        .insert(GlobalTransform::default());
}

pub struct EnvLightingPlugin;

impl Plugin for EnvLightingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_sun_lighting.system());
    }
}
