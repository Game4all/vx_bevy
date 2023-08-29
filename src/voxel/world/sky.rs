use bevy::prelude::{
    Color, Commands, Deref, DirectionalLight, DirectionalLightBundle, Entity, ParamSet, Plugin,
    Query, Res, Resource, Transform, Vec3, With,
};

use super::player::CameraMode;

#[derive(Resource, Deref)]
struct SkyLightEntity(Entity);

fn setup_sky_lighting(mut cmds: Commands) {
    const _SIZE: f32 = 200.0; //make this dynamic according to view distance???

    let sky_light_entity = cmds
        .spawn(DirectionalLightBundle {
            transform: Transform::IDENTITY.looking_to(Vec3::new(-1.0, -0.6, -1.0), Vec3::Y),
            directional_light: DirectionalLight {
                color: Color::WHITE,
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    cmds.insert_resource(SkyLightEntity(sky_light_entity));
}

fn update_light_position(
    sky_light_entity: Res<SkyLightEntity>,
    mut queries: ParamSet<(Query<&mut Transform>, Query<&Transform, With<CameraMode>>)>,
) {
    let sky_light_entity = **sky_light_entity;
    let player_translation = queries
        .p1()
        .get_single()
        .map_or_else(|_| Default::default(), |ply| ply.translation);

    {
        let mut binding = queries.p0();
        let mut sky_light_transform = binding.get_mut(sky_light_entity).unwrap();
        sky_light_transform.translation = player_translation;
    }
}

pub struct InteractiveSkyboxPlugin;

impl Plugin for InteractiveSkyboxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup_sky_lighting)
            .add_system(update_light_position);
    }
}
